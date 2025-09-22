use crate::types::AutoTest;
use crate::utils::{collect_rs_files_with_manifest, ensure_exists, get_tests_dir};
use anyhow::{Context, Result};
use std::{fs, io::Write, path::Path};

use functions::{clippy_autotests, commit_count_autotests, test_count_autotests};
use scan::{TestWithManifest, find_all_tests};
mod functions;
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
        items.extend(test_count_autotests(
            &manifest_paths,
            num_points,
            require_tests,
        ));
    }

    let json = serde_json::to_string_pretty(&items)?;
    let mut f = fs::File::create(&out_path)
        .with_context(|| format!("Failed to create {}", out_path.to_string_lossy()))?;
    f.write_all(json.as_bytes())?;

    println!("Wrote {}", out_path.to_string_lossy());
    Ok(())
}
