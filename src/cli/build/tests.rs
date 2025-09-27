use crate::types::*;
use super::*;

// Ensures a plain cargo test emits the expected step with quoted fields and no -- --exact
#[test]
fn yaml_includes_basic_cargo_test_step() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let root = tmp.path();

    // Arrange: write config with a single Cargo test
    let tests = vec![AutoTest {
        meta: TestMeta {
            name: "basic_add_small_numbers".into(),
            timeout: 10,
            points: 1,
            description: "".into(),
        },
        kind: TestKind::CargoTest { manifest_path: None },
    }];
    // helper identical to your other tests
    let autograder = root.join(".autograder");
    std::fs::create_dir_all(&autograder)?;
    std::fs::write(autograder.join("autograder.json"), serde_json::to_string_pretty(&tests)?)?;

    // Act
    run(root, true)?;
    let yaml = std::fs::read_to_string(root.join(".github/workflows/classroom.yml"))?;

    // Assert (quoted fields, no -- --exact)
    assert!(yaml.contains(r#"- name: "basic_add_small_numbers""#));
    assert!(yaml.contains(r#"id: "basic-add-small-numbers""#));
    assert!(yaml.contains(r#"uses: "classroom-resources/autograding-command-grader@v1""#));
    assert!(yaml.contains(r#"test-name: "basic_add_small_numbers""#));
    assert!(yaml.contains(r#"command: "cargo test basic_add_small_numbers""#));
    assert!(!yaml.contains(r#"-- --exact"#));
    assert!(yaml.contains(r#"timeout: 10"#));
    assert!(yaml.contains(r#"max-score: 1"#));
    Ok(())
}

// Ensures manifest-path appears AFTER the test name and is quoted as part of the command
#[test]
fn yaml_orders_manifest_flag_after_name() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let root = tmp.path();

    let tests = vec![AutoTest {
        meta: TestMeta {
            name: "unit_adds".into(),
            timeout: 10,
            points: 1,
            description: "".into(),
        },
        kind: TestKind::CargoTest { manifest_path: Some("member/Cargo.toml".into()) },
    }];
    let autograder = root.join(".autograder");
    std::fs::create_dir_all(&autograder)?;
    std::fs::write(autograder.join("autograder.json"), serde_json::to_string_pretty(&tests)?)?;

    run(root, true)?;
    let yaml = std::fs::read_to_string(root.join(".github/workflows/classroom.yml"))?;

    // Exact order we want
    assert!(yaml.contains(r#"command: "cargo test unit_adds --manifest-path member/Cargo.toml""#));
    Ok(())
}

// Ensures clippy and commit-count steps render, and reporter uses double-curly expressions
#[test]
fn yaml_includes_clippy_and_commit_count_and_reporter_env() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let root = tmp.path();

    let tests = vec![
        AutoTest {
            meta: TestMeta { name: "CLIPPY_STYLE_CHECK".into(), timeout: 10, points: 1, description: "".into() },
            kind: TestKind::Clippy { manifest_path: None },
        },
        AutoTest {
            meta: TestMeta { name: "COMMIT_COUNT_1".into(), timeout: 10, points: 1, description: "".into() },
            kind: TestKind::CommitCount { min_commits: 1 },
        },
    ];
    let autograder = root.join(".autograder");
    std::fs::create_dir_all(&autograder)?;
    std::fs::write(autograder.join("autograder.json"), serde_json::to_string_pretty(&tests)?)?;

    run(root, true)?;
    let yaml = std::fs::read_to_string(root.join(".github/workflows/classroom.yml"))?;

    // Clippy
    assert!(yaml.contains(r#"- name: "CLIPPY_STYLE_CHECK""#));
    assert!(yaml.contains(r#"id: "clippy-style-check""#));
    assert!(yaml.contains(r#"command: "cargo clippy -- -D warnings""#));

    // Commit count (script path name depends on your helper; adjust if needed)
    assert!(yaml.contains(r#"- name: "COMMIT_COUNT_1""#));
    assert!(yaml.contains(r#"id: "commit-count-1""#));
    assert!(yaml.contains(r#"command: "bash ./.autograder/"#)); // broad match
    assert!(yaml.contains(r#"max-score: 1"#));

    // Reporter: double-curly GitHub Actions expressions + runners CSV
    assert!(yaml.contains(
        r#"CLIPPY-STYLE-CHECK_RESULTS: "${{ steps.clippy-style-check.outputs.result }}""#
    ));
    assert!(yaml.contains(
        r#"COMMIT-COUNT-1_RESULTS: "${{ steps.commit-count-1.outputs.result }}""#
    ));
    assert!(yaml.contains("runners: clippy-style-check,commit-count-1"));
    Ok(())
}

// Guards against accidentally adding -- --exact later
#[test]
fn cargo_test_cmd_does_not_append_exact() {
    let out_root = crate::types::command_makers::cargo_test_cmd("foo", None);
    let out_mp   = crate::types::command_makers::cargo_test_cmd("foo", Some("x/Cargo.toml"));
    assert_eq!(out_root, "cargo test foo");
    assert_eq!(out_mp,   "cargo test foo --manifest-path x/Cargo.toml");
    assert!(!out_root.contains("-- --exact"));
    assert!(!out_mp.contains("-- --exact"));
}
