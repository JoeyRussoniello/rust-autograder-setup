use anyhow::{Context, Result};
use std::path::Path;

use serde_json;
use std::fs::{File, create_dir_all};
use std::io::BufReader;

use crate::utils::{YAML_INDENT, YAML_PREAMBLE, ensure_exists, slug_id, yaml_quote};

use crate::types::AutoTest;

pub fn run(root: &Path) -> Result<()> {
    let autograder_config = root.join("tests").join("autograder.json");
    ensure_exists(&autograder_config)?;
    let tests = read_autograder_config(&autograder_config)?;

    if tests.is_empty() {
        anyhow::bail!("Autograder.json config not configured. Add tests using `auto-setup init`");
    }

    let workflows_dir = root.join(".github").join("workflows");
    create_dir_all(&workflows_dir)
        .with_context(|| format!("Failed to create {}", workflows_dir.to_string_lossy()))?;

    //.yml used instead of .YAML for github classroom compatibility
    let workflow_path = workflows_dir.join("classroom.yml");

    let mut yaml_compiler = YAMLAutograder::new();
    yaml_compiler.set_preamble(YAML_PREAMBLE.to_string());
    yaml_compiler.set_tests(tests);
    let workflow_content = yaml_compiler.compile();

    write_workflow(&workflow_path, &workflow_content)?;
    println!(
        "Wrote Configured autograder YAML to {}",
        workflow_path.to_string_lossy()
    );
    Ok(())
}

fn read_autograder_config(path: &Path) -> Result<Vec<AutoTest>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tests = serde_json::from_reader(reader)?;
    Ok(tests)
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
}
impl YAMLAutograder {
    fn new() -> Self {
        Self {
            preamble: String::new(),
            autograder_content: String::new(),
            tests: Vec::new(),
            ids: Vec::new(),
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

    fn compile_test_steps(&mut self) {
        //Clone tests to avoid an immutable borrow on self
        let tests = self.tests.clone();
        let clippy_string = String::from("CLIPPY_STYLE_CHECK");
        for test in tests.iter() {
            //? Could move the clippy check into the compile_test_step function, but this is clearer
            if test.name != clippy_string {
                self.compile_test_step(test, "cargo test");
            } else {
                self.compile_test_step(test, "cargo clippy -- -D warnings");
            }
            self.autograder_content.push('\n');
        }
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

    fn compile(&mut self) -> String {
        self.autograder_content.clear();
        self.autograder_content.push_str(&self.preamble);
        self.compile_test_steps();
        self.compile_test_reporter();
        self.autograder_content.to_string()
    }
}


// src/build.rs (or wherever your build code lives)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AutoTest;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    // Small helper: write a JSON array of AutoTest to tests/autograder.json
    fn write_autograder_json(root: &Path, tests: &[AutoTest]) -> anyhow::Result<()> {
        let tests_dir = root.join("tests");
        fs::create_dir_all(&tests_dir)?;
        let path = tests_dir.join("autograder.json");
        let mut f = File::create(path)?;
        let s = serde_json::to_string_pretty(tests)?;
        f.write_all(s.as_bytes())?;
        Ok(())
    }

    fn read_workflow(root: &Path) -> anyhow::Result<String> {
        let p = root.join(".github/workflows/classroom.yml");
        Ok(fs::read_to_string(p)?)
    }

    #[test]
    fn run_generates_yaml_pruning_zero_point_and_using_exact_commands() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let root = tmp.path();

        // 3 tests: two graded, one 0-point clippy which must be pruned
        let tests = vec![
            AutoTest { name: "test_one".into(), timeout: 30, points: 2 },
            AutoTest { name: "CLIPPY_STYLE_CHECK".into(), timeout: 45, points: 0 },
            AutoTest { name: "tokio_async_test".into(), timeout: 40, points: 3 },
        ];
        write_autograder_json(root, &tests)?;

        // Act
        run(root)?; // should write .github/workflows/classroom.yml

        // Assert
        let yaml = read_workflow(root)?;
        // 1) Preamble is at the top
        assert!(yaml.starts_with(YAML_PREAMBLE));

        // 2) Steps for graded tests exist with quoted command and -- --exact
        assert!(yaml.contains(r#"- name: test_one"#));
        assert!(yaml.contains(r#"test-name: "test_one""#));
        assert!(yaml.contains(r#"command: "cargo test test_one -- --exact""#));
        assert!(yaml.contains(r#"max-score: 2"#));

        assert!(yaml.contains(r#"- name: tokio_async_test"#));
        assert!(yaml.contains(r#"test-name: "tokio_async_test""#));
        assert!(yaml.contains(r#"command: "cargo test tokio_async_test -- --exact""#));
        assert!(yaml.contains(r#"max-score: 3"#));

        // 3) 0-point clippy is pruned from steps & env
        assert!(!yaml.contains("CLIPPY_STYLE_CHECK"));
        assert!(!yaml.contains("cargo clippy -- -D warnings"));

        // 4) Reporter env/runners: IDs are slugged from names and uppercased in *_RESULTS
        // slug("test_one") => "test-one"; slug("tokio_async_test") => "tokio-async-test"
        assert!(yaml.contains(r#"TEST-ONE_RESULTS: "${{steps.test-one.outputs.result}}""#));
        assert!(yaml.contains(
            r#"TOKIO-ASYNC-TEST_RESULTS: "${{steps.tokio-async-test.outputs.result}}""#
        ));
        // Runners list preserves input order (after pruning)
        assert!(yaml.contains("runners: test-one,tokio-async-test"));

        Ok(())
    }

    #[test]
    fn compile_includes_clippy_command_when_points_positive() {
        // Directly exercise YAMLAutograder internals:
        // if CLIPPY has >0 points, it should be included with cargo clippy command.
        let mut ya = YAMLAutograder::new();
        ya.set_preamble(String::new());
        ya.set_tests(vec![
            AutoTest { name: "CLIPPY_STYLE_CHECK".into(), timeout: 5, points: 1 }
        ]);
        let out = ya.compile();

        assert!(out.contains(r#"- name: CLIPPY_STYLE_CHECK"#));
        assert!(out.contains(r#"command: "cargo clippy -- -D warnings""#));
        assert!(out.contains(r#"max-score: 1"#));
        // Reporter wiring should reference the slug id "clippy-style-check"
        assert!(out.contains(
            r#"CLIPPY-STYLE-CHECK_RESULTS: "${{steps.clippy-style-check.outputs.result}}""#
        ));
        assert!(out.contains("runners: clippy-style-check"));
    }

    #[test]
    fn read_autograder_config_parses_valid_json_and_errors_on_invalid() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let root = tmp.path();
        let tests_dir = root.join("tests");
        fs::create_dir_all(&tests_dir)?;

        // Valid JSON
        let valid_path = tests_dir.join("autograder.json");
        let valid = r#"
        [
          {"name":"a","timeout":10,"points":1},
          {"name":"b","timeout":20,"points":0}
        ]
        "#;
        fs::write(&valid_path, valid)?;
        let v = super::read_autograder_config(&valid_path)?;
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].name, "a");
        assert_eq!(v[1].points, 0);

        // Invalid JSON
        let invalid_path = tests_dir.join("autograder_bad.json");
        fs::write(&invalid_path, "not json")?;
        let err = super::read_autograder_config(&invalid_path).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("expected value") || msg.contains("EOF") || msg.contains("at line"),
            "unexpected error: {msg}"
        );

        Ok(())
    }

    #[test]
    fn write_workflow_creates_file_and_is_recoverable() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let root = tmp.path();
        let workflows = root.join(".github/workflows");
        fs::create_dir_all(&workflows)?;
        let p = workflows.join("classroom.yml");

        super::write_workflow(&p, "hello")?;
        assert_eq!(fs::read_to_string(&p)?, "hello");

        Ok(())
    }

    #[test]
    fn insert_autograder_string_respects_indentation_and_line_splitting() {
        let mut ya = YAMLAutograder::new();
        ya.set_preamble(String::new());
        ya.insert_autograder_string("foo".into(), 0);
        ya.insert_autograder_string("bar\nbaz".into(), 2);

        let out = ya.autograder_content.clone();
        // "foo" (no indent), then "bar"/"baz" each with two YAML_INDENTs
        let expected_bar = format!("\n{}{}\n", YAML_INDENT.repeat(2), "bar");
        let expected_baz = format!("{}{}\n", YAML_INDENT.repeat(2), "baz");
        assert!(out.starts_with("foo\n"));
        assert!(out.contains(&expected_bar));
        assert!(out.ends_with(&expected_baz));
    }
}
