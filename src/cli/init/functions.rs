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
        doc = format!("`cargo clippy` style check for `{}`", dir);
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

/// A generic helper to create threshold-based autotests
fn threshold_autotests<I>(
    iterator: I,
    points: u32,
    prefix: &str,
    description: &str,
    mk_kind: impl Fn(u32) -> TestKind,
) -> Vec<AutoTest>
where
    I: Iterator<Item = u32>,
{
    iterator
        .map(|i| AutoTest {
            meta: TestMeta {
                name: format!("{}_{}", prefix, i),
                timeout: 10,
                points,
                description: format!("Ensures at least {} {}.", i, description),
            },
            kind: mk_kind(i),
        })
        .collect()
}

pub fn commit_count_autotests<I>(iterator: I, points: u32) -> Vec<AutoTest>
where
    I: Iterator<Item = u32>,
{
    threshold_autotests(iterator, points, "COMMIT_COUNT", "commits", |i| {
        TestKind::CommitCount { min_commits: i }
    })
}

pub fn branch_count_autotests<I>(iterator: I, points: u32) -> Vec<AutoTest>
where
    I: Iterator<Item = u32>,
{
    threshold_autotests(iterator, points, "BRANCH_COUNT", "branches", |i| {
        TestKind::BranchCount { min_branches: i }
    })
}

pub fn test_count_autotests<I>(
    iterator: I,
    points: u32,
    manifest_paths: &HashSet<String>,
) -> Vec<AutoTest>
where
    I: Iterator<Item = u32>,
{
    // Collect required test thresholds so we can iterate them multiple times.
    let required: Vec<u32> = iterator.collect();

    // Stable order for deterministic output: sort manifest paths.
    let mut mps: Vec<String> = manifest_paths.iter().cloned().collect();
    mps.sort();

    let mut out = Vec::new();
    for mp in mps {
        for req in &required {
            out.push(test_count_autotest_for(&mp, points, *req));
        }
    }
    out
}

/// Likely needs to be refactored for iterator based solution
fn test_count_autotest_for(manifest_path: &str, points: u32, required_tests: u32) -> AutoTest {
    let dir = manifest_dir_label(manifest_path);

    let name: String;
    let docstring: String;
    let manifest_path_opt: Option<String>;

    // THis can go in a function
    if matches!(dir.as_str(), "." | "Cargo.toml") {
        docstring = format!("Submission has at least {} tests", required_tests);
        manifest_path_opt = None;
        name = format!("TEST_COUNT_{}", required_tests);
    } else {
        docstring = format!("{} submission has at least {} tests", dir, required_tests);
        manifest_path_opt = Some(manifest_path.to_string());
        name = format!(
            "TEST_COUNT_{}_{}",
            manifest_path.to_uppercase(),
            required_tests
        );
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
