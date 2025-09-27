use crate::types::{AutoTest, TestKind, TestMeta};
/// A helper module for individual AutoTest Creation
use std::collections::HashSet;

/// Collects manifest_paths into different clippy checks
pub fn clippy_autotests(manifest_paths: &HashSet<String>, points: u32) -> Vec<AutoTest> {
    manifest_paths
        .iter()
        .map(|mp| clippy_autotest_for(mp, points))
        .collect()
}

fn clippy_autotest_for(manifest_path: &str, points: u32) -> AutoTest {
    let dir = manifest_dir_label(manifest_path); // "." | "Cargo.toml" | "member"

    let name: String;
    let doc: String;
    let manifest_path_opt: Option<String>;

    if matches!(dir.as_str(), "." | "Cargo.toml") {
        name = "CLIPPY_STYLE_CHECK".to_string();
        doc = "`cargo clippy` style check".to_string();
        manifest_path_opt = None;
    } else {
        name = format!("CLIPPY_STYLE_CHECK_{}", dir);
        doc = format!("`cargo clippy style check for `{}`", dir);
        manifest_path_opt = Some(manifest_path.to_string());
    }

    AutoTest {
        meta: TestMeta {
            name,
            points,
            timeout: 10,
            description: doc.clone(),
        },
        kind: TestKind::Clippy {
            manifest_path: manifest_path_opt,
        },
    }
}

pub fn commit_count_autotests(n: u32, points: u32) -> Vec<AutoTest> {
    (1..=n)
        .map(|i| AutoTest {
            meta: TestMeta {
                name: format!("COMMIT_COUNT_{}", i),
                timeout: 10,
                points,
                // ## Intentionally left to allow flexibility when reading autograder.json
                description: "Ensures at least ## commits.".to_string(),
            },
            kind: TestKind::CommitCount { min_commits: i },
        })
        .collect()
}

/// Count test cases per manifest path
pub fn test_count_autotests(
    manifest_paths: &HashSet<String>,
    points: u32,
    required_tests: u32,
) -> Vec<AutoTest> {
    manifest_paths
        .iter()
        .map(|mp| test_count_autotest_for(mp, points, required_tests))
        .collect()
}
fn test_count_autotest_for(manifest_path: &str, points: u32, required_tests: u32) -> AutoTest {
    let dir = manifest_dir_label(manifest_path);

    let name: String;
    let docstring: String;
    let manifest_path_opt: Option<String>;

    if matches!(dir.as_str(), "." | "Cargo.toml") {
        docstring = format!("Submission has at least {} tests", required_tests);
        manifest_path_opt = None;
        name = "TEST_COUNT".to_string();
    } else {
        docstring = format!("{} submission has at least {} tests", dir, required_tests);
        manifest_path_opt = Some(manifest_path.to_string());
        name = format!("TEST_COUNT_{}", manifest_path.to_uppercase());
    }

    AutoTest {
        meta: TestMeta {
            name,
            points,
            timeout: 10,
            description: docstring.clone(),
        },
        kind: TestKind::TestCount {
            min_tests: required_tests,
            manifest_path: manifest_path_opt,
        },
    }
}

/// Turn ".../Cargo.toml" into "member" or "." for workspace root.
fn manifest_dir_label(path: &str) -> String {
    if path == "Cargo.toml" {
        return ".".into();
    }
    std::path::Path::new(path)
        .parent()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".into())
}
