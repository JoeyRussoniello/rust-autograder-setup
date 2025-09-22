use crate::types::AutoTest;
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
        name,
        timeout: 10,
        points,
        docstring: doc,
        // NOTE: your README calls this field `num_commits`; consider renaming for consistency.
        min_commits: None,
        manifest_path: manifest_path_opt,
    }
}

pub fn commit_count_autotests(n: u32, points: u32) -> Vec<AutoTest> {
    (1..=n)
        .map(|i| AutoTest {
            name: format!("COMMIT_COUNT_{}", i),
            timeout: 10,
            points,
            // `table` replaces the "##" later—leave as-is.
            docstring: "Ensures at least ## commits.".to_string(),
            min_commits: Some(i),
            manifest_path: None,
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
    let stripped_path = manifest_path
        .strip_suffix("/Cargo.toml")
        .unwrap_or(manifest_path);

    let docstring = if stripped_path == "." || stripped_path == "Cargo.toml" {
        format!("Submission has at least {} tests", required_tests)
    } else {
        format!(
            "{} submission has at least {} tests",
            stripped_path, required_tests
        )
    };

    AutoTest {
        name: format!("TEST_COUNT_{}", manifest_path.to_uppercase()),
        timeout: 10,
        points,
        docstring,
        min_commits: Some(required_tests),
        manifest_path: Some(manifest_path.to_string()),
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
