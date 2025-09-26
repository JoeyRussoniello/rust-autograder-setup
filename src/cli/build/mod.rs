use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::types::{AutoTest, StepCmd};
use crate::utils::{
    YAML_INDENT, get_commit_count_file_name_from_str, read_autograder_config,
    replace_double_hashtag, slug_id, yaml_quote,
};
use std::collections::HashMap;
use std::fs::{File, create_dir_all};

mod build_functions;

pub fn run(root: &Path, grade_on_push: bool) -> Result<()> {
    let tests = read_autograder_config(root)?;

    let workflows_dir = root.join(".github").join("workflows");
    create_dir_all(&workflows_dir)
        .with_context(|| format!("Failed to create {}", workflows_dir.to_string_lossy()))?;

    //.yml used instead of .YAML for github classroom compatibility
    let workflow_path = workflows_dir.join("classroom.yml");

    let mut yaml_compiler = YAMLAutograder::new(root.to_path_buf());
    yaml_compiler.set_preamble(build_functions::get_yaml_preamble(grade_on_push));
    yaml_compiler.set_tests(tests);
    let workflow_content = yaml_compiler.compile();

    write_workflow(
        &workflow_path,
        &workflow_content.expect("Unable to compile YAML"),
    )?;
    println!(
        "Wrote Configured autograder YAML to {}",
        workflow_path.to_string_lossy()
    );
    Ok(())
}

fn write_workflow(path: &Path, content: &str) -> Result<()> {
    let mut f = File::create(path)
        .with_context(|| format!("Failed to create {}", path.to_string_lossy()))?;
    use std::io::Write;
    f.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write {}", path.to_string_lossy()))?;
    Ok(())
}

pub struct YAMLAutograder {
    pub preamble: String,
    pub autograder_content: String,
    tests: Vec<AutoTest>,
    ids: Vec<String>,
    root: PathBuf,
}
impl YAMLAutograder {
    fn new(root: PathBuf) -> Self {
        Self {
            preamble: String::new(),
            autograder_content: String::new(),
            tests: Vec::new(),
            ids: Vec::new(),
            root,
        }
    }

    fn set_preamble(&mut self, preamble: String) {
        self.preamble = preamble;
    }

    fn set_tests(&mut self, tests: Vec<AutoTest>) {
        self.tests = tests.into_iter().filter(|t| t.points > 0).collect();
        self.ids = Vec::with_capacity(self.tests.len());
    }

    fn compile_test_step(&mut self, test: &AutoTest, cmd: &str) {
        let name = test.name.trim();
        let id = slug_id(name);
        let indent_level = 3;
        self.ids.push(id.clone());

        self.insert_autograder_string(format!("- name: {}", name), indent_level);
        self.insert_autograder_string(
            format!(
                "id: {}\nuses: classroom-resources/autograding-command-grader@v1\nwith:",
                id
            ),
            indent_level + 1,
        );

        let full_command = if cmd == "cargo test" {
            format!("{} {} -- --exact", cmd, name)
        } else {
            cmd.to_string()
        };

        self.insert_autograder_string(
            format!(
                "test-name: {}\nsetup-command: {}\ncommand: {}\ntimeout: {}\nmax-score: {}\n",
                yaml_quote(name),
                yaml_quote(""),
                yaml_quote(&full_command),
                test.timeout,
                test.points
            ),
            indent_level + 2,
        );
    }

    fn compile_test_steps(&mut self) -> Result<()> {
        //Clone tests to avoid an immutable borrow on self
        let tests = self.tests.clone();

        // Count the number of `cargo tests` present in each manifest path
        let counts_by_manifest: HashMap<Option<String>, u32> = tests
            .iter()
            .map(infer_step_cmd)
            .filter_map(|s| {
                if let StepCmd::CargoTest { manifest_path, .. } = s {
                    // manifest_path is already an Option<String>, just return it
                    Some(manifest_path)
                } else {
                    None
                }
            })
            .fold(HashMap::new(), |mut acc, path| {
                *acc.entry(path).or_insert(0) += 1;
                acc
            });

        for test in tests.iter() {
            let step = infer_step_cmd(test);

            match &step {
                StepCmd::TestCount { .. } => {
                    let base_command = step.command();
                    let mp = &test.manifest_path;
                    // ? Maybe revisit defaulting to zero
                    let num_cargo_tests = counts_by_manifest.get(mp).unwrap_or(&0);
                    self.compile_test_step(
                        test,
                        &replace_double_hashtag(base_command, *num_cargo_tests),
                    )
                }
                StepCmd::CommitCount { min, name } => {
                    write_commit_count_shell(
                        &self.root,
                        *min,
                        &get_commit_count_file_name_from_str(name),
                    )?;

                    self.compile_test_step(test, &step.command());
                }
                _ => self.compile_test_step(test, &step.command()),
            }
            self.autograder_content.push('\n');
        }

        Ok(())
    }

    fn compile_test_reporter(&mut self) {
        let indent_level = 3;
        self.insert_autograder_string("- name: Autograding Reporter".to_string(), indent_level);
        self.insert_autograder_string(
            "uses: classroom-resources/autograding-grading-reporter@v1\nenv:".to_string(),
            indent_level + 1,
        );

        let ids = self.ids.clone();
        for id in ids.iter() {
            let env_key = format!("{}_RESULTS", id.to_uppercase());
            self.insert_autograder_string(
                format!("{}: \"${{{{steps.{}.outputs.result}}}}\"", env_key, id),
                indent_level + 2,
            );
        }

        self.insert_autograder_string("with:".to_string(), indent_level + 1);
        self.insert_autograder_string(format!("runners: {}", self.ids.join(",")), indent_level + 2);
    }

    fn insert_autograder_string(&mut self, s: String, indent_level: usize) {
        let indent = YAML_INDENT.repeat(indent_level);
        //? Could raise error on multi-lines to avoid undetermined behavior
        for line in s.lines() {
            self.autograder_content
                .push_str(&format!("{}{}\n", indent, line));
        }
    }

    fn compile(&mut self) -> Result<String> {
        self.autograder_content.clear();
        self.autograder_content.push_str(&self.preamble);
        self.compile_test_steps()?;
        self.compile_test_reporter();
        Ok(self.autograder_content.to_string())
    }
}

fn infer_step_cmd(test: &AutoTest) -> StepCmd {
    let n = test.name.trim();
    let manifest_path = test.manifest_path.clone();
    // Style check
    if n.to_ascii_lowercase().contains("clippy_style_check") {
        return StepCmd::ClippyCheck { manifest_path };
    }

    // Commit count
    if n.starts_with("COMMIT_COUNT") {
        // Priority: explicit field > number in name > default
        return StepCmd::CommitCount {
            name: n.to_string(),
            min: test.min_commits.unwrap(),
        };
    }

    if n.starts_with("TEST_COUNT") {
        return StepCmd::TestCount {
            min: test.min_tests.unwrap(),
            manifest_path,
        };
    }

    // Default: cargo test by function name
    StepCmd::CargoTest {
        function_name: n.to_string(),
        manifest_path,
    }
}

fn write_commit_count_shell(root: &Path, num_commits: u32, name: &str) -> Result<()> {
    let script_path = root.join(".autograder").join(name);
    // Shell script content
    let script = format!(
        r#"#!/usr/bin/env bash
# tests/commit_count.sh
set -euo pipefail

# Usage:
#   MIN=3 bash tests/commit_count.sh
#   bash tests/commit_count.sh -m 3

MIN={min}

# Validate MIN
if ! [[ "$MIN" =~ ^[0-9]+$ ]]; then
  echo "MIN must be a non-negative integer; got: '$MIN'" >&2
  exit 2
fi

# Ensure we're in a git repo
if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "Not a git repository (are you running inside the checkout?)" >&2
  exit 1
fi

# Warn if shallow (runner must checkout with fetch-depth: 0 for full history)
if [ -f "$(git rev-parse --git-dir)/shallow" ]; then
  echo "Warning: shallow clone detected; commit count may be incomplete." >&2
fi

# Count commits
COUNT=$(git rev-list --count HEAD 2>/dev/null || echo 0)

if [ "$COUNT" -ge "$MIN" ]; then
  echo "✅ Found $COUNT commits (min $MIN) — PASS"
  exit 0
else
  echo "❌ Found $COUNT commits (min $MIN) — FAIL"
  exit 1
fi
"#,
        min = num_commits
    );

    // Write the file
    write_workflow(&script_path, &script)?;

    println!(
        "Wrote commit count shell to {}",
        script_path.to_string_lossy()
    );
    Ok(())
}

#[cfg(test)]
pub mod tests;
