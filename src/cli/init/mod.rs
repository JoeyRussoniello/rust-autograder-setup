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

// This many function args are necessary for flag parsing
#[allow(clippy::too_many_arguments)]
pub fn run(
    root: &Path,
    tests_dir_name: &Path,
    num_points: u32,
    style_check: bool,
    commit_counts: bool,
    num_commit_checks: Option<u32>, // DEPRECATED: present (Some) or absent (None)
    require_tests: u32,
    require_commits: &[u32], // New preferred thresholds
) -> Result<()> {
    // ---- Discover tests ------------------------------------------------------
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

    // ---- Prepare output location --------------------------------------------
    let out_dir = root.join(".autograder");
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("Failed to create {}", out_dir.to_string_lossy()))?;
    let out_path = out_dir.join("autograder.json");

    // ---- Convert discovered tests to AutoTests -------------------------------
    let manifest_paths = TestWithManifest::get_distinct_manifest_paths(&tests, root);

    let mut items: Vec<AutoTest> = tests
        .into_iter()
        .map(|t| t.to_autotest(root, num_points))
        .collect();

    if style_check {
        items.extend(clippy_autotests(&manifest_paths, num_points));
    }

    // ---- Commit counting logic ----------------------------------------------
    //
    // Precedence:
    //   1) If commit_counts=false, ignore both flags and emit nothing.
    //   2) If commit_counts=true and require_commits is non-empty, use those thresholds.
    //   3) Else if commit_counts=true and num_commit_checks=Some(n):
    //        - n==0 => emit nothing
    //        - n>=1 => thresholds 1..=n (deprecated expansion)
    //   4) Else (no thresholds) => emit nothing.
    //
    if commit_counts {
        let thresholds: Vec<u32> = if !require_commits.is_empty() {
            require_commits.to_vec()
        } else if let Some(n) = num_commit_checks {
            if n == 0 {
                Vec::new()
            } else {
                (1..=n).collect()
            }
        } else {
            Vec::new()
        };

        if !thresholds.is_empty() {
            items.extend(commit_count_autotests(thresholds.into_iter(), num_points));
        }
    }

    // ---- Test count steps ----------------------------------------------------
    if require_tests > 0 {
        items.extend(test_count_autotests(
            &manifest_paths,
            num_points,
            require_tests,
        ));
    }

    // ---- Write config --------------------------------------------------------
    let json = serde_json::to_string_pretty(&items)?;
    let mut f = fs::File::create(&out_path)
        .with_context(|| format!("Failed to create {}", out_path.to_string_lossy()))?;
    f.write_all(json.as_bytes())?;

    println!("Wrote {}", out_path.to_string_lossy());
    Ok(())
}
