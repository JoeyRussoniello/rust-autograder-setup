use super::*;
use crate::types::{AutoTest, TestKind, TestMeta};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

use super::build_functions::get_yaml_preamble;

// Small helper: write a JSON array of AutoTest to .autograder/autograder.json
fn write_autograder_json(root: &Path, tests: &[AutoTest]) -> anyhow::Result<()> {
    let tests_dir = root.join(".autograder");
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
        AutoTest {
            meta: TestMeta {
                name: "test_one".into(),
                timeout: 30,
                points: 2,
                description: "".into(),
            },
            kind: TestKind::CargoTest {
                manifest_path: None,
            },
        },
        AutoTest {
            meta: TestMeta {
                name: "CLIPPY_STYLE_CHECK".into(),
                timeout: 45,
                points: 0,
                description: "".into(),
            },
            kind: TestKind::Clippy {
                manifest_path: None,
            },
        },
        AutoTest {
            meta: TestMeta {
                name: "tokio_async_test".into(),
                timeout: 40,
                points: 3,
                description: "".into(),
            },
            kind: TestKind::CargoTest {
                manifest_path: None,
            },
        },
    ];
    write_autograder_json(root, &tests)?;

    // Act
    run(root, true)?; // writes .github/workflows/classroom.yml

    // Assert
    let yaml = read_workflow(root)?;
    assert!(yaml.starts_with(&get_yaml_preamble(true)));
    assert!(yaml.contains(r#"- name: test_one"#));
    assert!(yaml.contains(r#"test-name: "test_one""#));
    assert!(yaml.contains(r#"command: "cargo test test_one""#));
    assert!(yaml.contains(r#"max-score: 2"#));

    assert!(yaml.contains(r#"- name: tokio_async_test"#));
    assert!(yaml.contains(r#"test-name: "tokio_async_test""#));
    assert!(yaml.contains(r#"command: "cargo test tokio_async_test""#));
    assert!(yaml.contains(r#"max-score: 3"#));

    // pruned 0-pt clippy
    assert!(!yaml.contains("CLIPPY_STYLE_CHECK"));
    assert!(!yaml.contains("cargo clippy -- -D warnings"));

    // Reporter env/runners
    assert!(yaml.contains(r#"TEST-ONE_RESULTS: "${{steps.test-one.outputs.result}}""#));
    assert!(
        yaml.contains(r#"TOKIO-ASYNC-TEST_RESULTS: "${{steps.tokio-async-test.outputs.result}}""#)
    );
    assert!(yaml.contains("runners: test-one,tokio-async-test"));
    Ok(())
}

#[test]
fn compile_includes_clippy_command_when_points_positive() {
    let mut ya = YAMLAutograder::new(PathBuf::from("."));
    ya.set_preamble(String::new());
    ya.set_tests(vec![AutoTest {
        meta: TestMeta {
            name: "CLIPPY_STYLE_CHECK".into(),
            timeout: 5,
            points: 1,
            description: "".into(),
        },
        kind: TestKind::Clippy {
            manifest_path: None,
        },
    }]);

    let out = ya.compile().expect("Unable to compile YAML");
    assert!(out.contains(r#"- name: CLIPPY_STYLE_CHECK"#));
    assert!(out.contains(r#"command: "cargo clippy -- -D warnings""#));
    assert!(out.contains(r#"max-score: 1"#));
    assert!(
        out.contains(
            r#"CLIPPY-STYLE-CHECK_RESULTS: "${{steps.clippy-style-check.outputs.result}}""#
        )
    );
    assert!(out.contains("runners: clippy-style-check"));
}

#[test]
fn read_autograder_config_parses_valid_json_and_errors_on_invalid() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let root = tmp.path();
    let tests_dir = root.join(".autograder");
    fs::create_dir_all(&tests_dir)?;

    // Valid JSON in NEW schema
    fs::write(
        tests_dir.join("autograder.json"),
        r#"[{
              "meta":{"name":"a","timeout":10,"points":1,"description":"test a"},
              "type":"cargo_test"
            },{
              "meta":{"name":"b","timeout":20,"points":0,"description":""},
              "type":"clippy"
            }]"#,
    )?;
    let v = super::read_autograder_config(root)?;
    assert_eq!(v.len(), 2);
    assert_eq!(v[0].meta.name, "a");
    assert_eq!(v[1].meta.points, 0);

    // Invalid JSON
    fs::write(tests_dir.join("autograder.json"), "not json")?;
    let err = super::read_autograder_config(root).unwrap_err();
    let msg = err.to_string();
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
fn run_includes_manifest_path_when_present() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let root = tmp.path();

    let tests = vec![
        AutoTest {
            meta: TestMeta {
                name: "unit_adds".into(),
                timeout: 10,
                points: 2,
                description: "".into(),
            },
            kind: TestKind::CargoTest {
                manifest_path: Some("questions/q1/Cargo.toml".into()),
            },
        },
        AutoTest {
            meta: TestMeta {
                name: "root_case".into(),
                timeout: 10,
                points: 1,
                description: "".into(),
            },
            kind: TestKind::CargoTest {
                manifest_path: None,
            },
        },
    ];
    write_autograder_json(root, &tests)?;

    // Act
    run(root, true)?;

    // Assert
    let yaml = read_workflow(root)?;
    assert!(yaml.starts_with(&get_yaml_preamble(true)));

    // With manifest_path
    assert!(yaml.contains(r#"- name: unit_adds"#));
    assert!(yaml.contains(r#"test-name: "unit_adds""#));
    assert!(
        yaml.contains(r#"command: "cargo test unit_adds --manifest-path questions/q1/Cargo.toml""#)
    );
    assert!(yaml.contains(r#"max-score: 2"#));

    // Without manifest_path
    assert!(yaml.contains(r#"- name: root_case"#));
    assert!(yaml.contains(r#"test-name: "root_case""#));
    assert!(yaml.contains(r#"command: "cargo test root_case""#));
    assert!(!yaml.contains("root_case --manifest-path"));
    assert!(yaml.contains(r#"max-score: 1"#));

    // Reporter wiring
    assert!(yaml.contains(r#"UNIT-ADDS_RESULTS: "${{steps.unit-adds.outputs.result}}""#));
    assert!(yaml.contains(r#"ROOT-CASE_RESULTS: "${{steps.root-case.outputs.result}}""#));
    assert!(yaml.contains("runners: unit-adds,root-case"));

    Ok(())
}
