use anyhow::{Context, Result};
use std::{fs, io::Write, path::Path};
use std::collections::HashSet;
use crate::types::AutoTest;
use crate::utils::{collect_rs_files_with_manifest, ensure_exists, get_tests_dir};

use scan::{TestWithManifest, find_all_tests};

mod scan;
#[cfg(test)]
mod tests;

pub fn run(
    root: &Path,
    tests_dir_name: &Path,
    num_points: u32,
    style_check: bool,
    commit_counts: bool,
    num_commit_checks: u32,
    require_tests: u32,
) -> Result<()> {
    let tests_dir = get_tests_dir(root, tests_dir_name);
    ensure_exists(&tests_dir)?;
    println!("Scanning {} for tests...", tests_dir.to_string_lossy());

    let files = collect_rs_files_with_manifest(&tests_dir)
        .with_context(|| format!("While scanning {}", tests_dir.to_string_lossy()))?;
    if files.is_empty() {
        anyhow::bail!("No `.rs` files found under {}", tests_dir.to_string_lossy());
    }

    let tests = find_all_tests(&files)
        .with_context(|| "Error converting test cases to test cases with manifest paths")?;

    if tests.is_empty() {
        anyhow::bail!("Found no test functions (looked for #[test]/#[...::test])");
    }

    let out_dir = root.join(".autograder");
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("Failed to create {}", out_dir.to_string_lossy()))?;
    let out_path = out_dir.join("autograder.json");

    // * Get the distinct list of manifest paths BEFORE consuming the test objects
    let manifest_paths = TestWithManifest::get_distinct_manifest_paths(&tests, root);
    // Build AutoTests, attaching manifest_path when present
    let mut items: Vec<AutoTest> = tests
        .into_iter()
        .map(|t| t.to_autotest(root, num_points))
        .collect();

    if style_check {
        //Create autotests for each distinct manifest path we found
        items.extend(clippy_autotests(&manifest_paths, num_points));
    }

    if commit_counts {
        items.extend(commit_count_autotests(num_commit_checks, num_points));
    }

    if require_tests > 0 {
        items.extend(test_count_autotests(&manifest_paths, num_points, require_tests));
    }

    let json = serde_json::to_string_pretty(&items)?;
    let mut f = fs::File::create(&out_path)
        .with_context(|| format!("Failed to create {}", out_path.to_string_lossy()))?;
    f.write_all(json.as_bytes())?;

    println!("Wrote {}", out_path.to_string_lossy());
    Ok(())
}

/// ! Consumes manifest_paths. Can be reworked if manifest paths are needed for something else
fn clippy_autotests(
    manifest_paths: &HashSet<String>,
    points: u32,
) -> Vec<AutoTest> {
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

fn commit_count_autotests(n: u32, points: u32) -> Vec<AutoTest> {
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
fn test_count_autotests(manifest_paths: &HashSet<String>, points:u32, required_tests: u32) -> Vec<AutoTest>{
    manifest_paths
        .iter()
        .map(|mp| test_count_autotest_for(mp, points, required_tests))
        .collect()
}
fn test_count_autotest_for(manifest_path: &str, points: u32, required_tests: u32) -> AutoTest {
    AutoTest{
        name: format!("TEST_COUNT"),
        timeout: 10,
        points,
        docstring: format!("{} submission has at least {} tests", manifest_path, required_tests),
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
