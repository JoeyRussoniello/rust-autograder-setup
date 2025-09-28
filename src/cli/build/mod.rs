use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::types::{AutoTest, TestKind};
use crate::utils::{read_autograder_config, replace_double_hashtag, slug_id};

use build_functions::{get_yaml_preamble, write_branch_count_shell, write_commit_count_shell};
use std::collections::{BTreeMap, HashMap};
use std::fs::{File, create_dir_all};
use steps::{CommandStep, CommandWith, ReporterStep};

mod build_functions;
mod steps;

pub fn run(root: &Path, grade_on_push: bool) -> Result<()> {
    let tests = read_autograder_config(root)?;
    let workflows_dir = root.join(".github").join("workflows");
    create_dir_all(&workflows_dir)
        .with_context(|| format!("Failed to create {}", workflows_dir.to_string_lossy()))?;

    //.yml used instead of .YAML for github classroom compatibility
    let workflow_path = workflows_dir.join("classroom.yml");

    let mut yaml_compiler = YAMLAutograder::new(root.to_path_buf());
    yaml_compiler.set_preamble(get_yaml_preamble(grade_on_push));
    yaml_compiler.set_tests(tests);
    let workflow_content = yaml_compiler.compile();

    create_and_write(
        &workflow_path,
        &workflow_content.expect("Unable to compile YAML"),
    )?;
    println!(
        "Wrote Configured autograder YAML to {}",
        workflow_path.to_string_lossy()
    );
    Ok(())
}

pub fn create_and_write(path: &Path, content: &str) -> Result<()> {
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
        self.tests = tests.into_iter().filter(|t| t.meta.points > 0).collect();
        self.ids = Vec::with_capacity(self.tests.len());
    }

    fn compile_test_step(&mut self, test: &AutoTest, cmd: &str) {
        let name = test.meta.name.trim().to_string();
        let id = slug_id(&name);
        self.ids.push(id.clone());

        let step = CommandStep {
            name: name.clone(),
            id,
            uses: "classroom-resources/autograding-command-grader@v1".into(),
            with: CommandWith {
                test_name: name,
                setup_command: "".into(),
                command: cmd.into(),
                timeout: test.meta.timeout,
                max_score: test.meta.points,
            },
        };

        // write it at the same indent (3) as before
        step.write_to(&mut self.autograder_content, 3);
        self.autograder_content.push('\n');
    }

    fn compile_test_steps(&mut self) -> anyhow::Result<()> {
        let tests = self.tests.clone();

        // Count cargo tests per manifest
        let mut counts_by_manifest: HashMap<Option<String>, u32> = HashMap::new();
        for t in &tests {
            if let TestKind::CargoTest { manifest_path } = &t.kind {
                *counts_by_manifest.entry(manifest_path.clone()).or_insert(0) += 1;
            }
        }

        for test in &tests {
            match &test.kind {
                TestKind::TestCount { manifest_path, .. } => {
                    let base = test.command();
                    let n = *counts_by_manifest.get(manifest_path).unwrap_or(&0);
                    self.compile_test_step(test, &replace_double_hashtag(&base, n));
                }
                TestKind::CommitCount { .. } => {
                    write_commit_count_shell(&self.root)?;
                    self.compile_test_step(test, &test.command());
                }
                TestKind::BranchCount { .. } => {
                    write_branch_count_shell(&self.root)?;
                    self.compile_test_step(test, &test.command());
                }
                _ => self.compile_test_step(test, &test.command()),
            }
        }
        Ok(())
    }

    fn compile_test_reporter(&mut self) {
        let mut env = BTreeMap::new(); // stable order for clean diffs
        for id in &self.ids {
            env.insert(
                format!("{}_RESULTS", id.to_uppercase()),
                format!("${{{{steps.{id}.outputs.result}}}}"),
            );
        }
        let runners_csv = self.ids.join(",");

        let reporter = ReporterStep {
            name: "Autograding Reporter".into(),
            uses: "classroom-resources/autograding-grading-reporter@v1".into(),
            env,
            runners_csv,
        };

        reporter.write_to(&mut self.autograder_content, 3);
    }

    fn compile(&mut self) -> Result<String> {
        self.autograder_content.clear();
        self.autograder_content.push_str(&self.preamble);
        self.compile_test_steps()?;
        self.compile_test_reporter();
        Ok(self.autograder_content.to_string())
    }
}

#[cfg(test)]
pub mod tests;
