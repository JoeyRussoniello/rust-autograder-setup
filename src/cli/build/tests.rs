use super::*;
use crate::types::AutoTest;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

// Small helper: write a JSON array of AutoTest to tests/autograder.json
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
            name: "test_one".into(),
            timeout: 30,
            points: 2,
            docstring: "".into(),
            min_commits: None,
        },
        AutoTest {
            name: "CLIPPY_STYLE_CHECK".into(),
            timeout: 45,
            points: 0,
            docstring: "".into(),
            min_commits: None,
        },
        AutoTest {
            name: "tokio_async_test".into(),
            timeout: 40,
            points: 3,
            docstring: "".into(),
            min_commits: None,
        },
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
    assert!(
        yaml.contains(r#"TOKIO-ASYNC-TEST_RESULTS: "${{steps.tokio-async-test.outputs.result}}""#)
    );
    // Runners list preserves input order (after pruning)
    assert!(yaml.contains("runners: test-one,tokio-async-test"));

    Ok(())
}

#[test]
fn compile_includes_clippy_command_when_points_positive() {
    // Directly exercise YAMLAutograder internals:
    // if CLIPPY has >0 points, it should be included with cargo clippy command.
    let mut ya = YAMLAutograder::new(PathBuf::from("."));
    ya.set_preamble(String::new());
    ya.set_tests(vec![AutoTest {
        name: "CLIPPY_STYLE_CHECK".into(),
        timeout: 5,
        points: 1,
        docstring: "".into(),
        min_commits: None,
    }]);
    let out = ya.compile().expect("Unable to compile YAML");

    assert!(out.contains(r#"- name: CLIPPY_STYLE_CHECK"#));
    assert!(out.contains(r#"command: "cargo clippy -- -D warnings""#));
    assert!(out.contains(r#"max-score: 1"#));
    // Reporter wiring should reference the slug id "clippy-style-check"
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

    // Valid JSON
    fs::write(
        tests_dir.join("autograder.json"),
        r#"[{"name":"a","timeout":10,"points":1,"docstring":"test a"},
            {"name":"b","timeout":20,"points":0,"docstring":""}]"#,
    )?;
    let v = super::read_autograder_config(root)?; // <-- pass root
    assert_eq!(v.len(), 2);
    assert_eq!(v[0].name, "a");
    assert_eq!(v[1].points, 0);

    // Invalid JSON (overwrite the same file with bad contents)
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
fn insert_autograder_string_respects_indentation_and_line_splitting() {
    let mut ya = YAMLAutograder::new(PathBuf::from("."));
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
