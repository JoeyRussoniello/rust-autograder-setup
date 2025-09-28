use super::RunConfig;
use crate::types::AutoTest;
use crate::utils::{collect_rs_files_with_manifest, ensure_exists, get_tests_dir};
use anyhow::{Context, Result};
use std::{fs, io::Write};

use functions::*;
use scan::{TestWithManifest, find_all_tests};
mod functions;
mod scan;
#[cfg(test)]
mod tests;

// This many function args are necessary for flag parsing
#[allow(clippy::too_many_arguments)]
pub fn run(cfg: &RunConfig) -> Result<()> {
    // ---- Discover tests ------------------------------------------------------
    let tests_dir = get_tests_dir(&cfg.root, &cfg.tests_dir_name);
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
    let out_dir = &cfg.root.join(".autograder");
    fs::create_dir_all(out_dir)
        .with_context(|| format!("Failed to create {}", out_dir.to_string_lossy()))?;
    let out_path = out_dir.join("autograder.json");

    // ---- Convert discovered tests to AutoTests -------------------------------
    let manifest_paths = TestWithManifest::get_distinct_manifest_paths(&tests, &cfg.root);

    let mut items: Vec<AutoTest> = tests
        .into_iter()
        .map(|t| t.to_autotest(&cfg.root, cfg.num_points))
        .collect();

    if cfg.style_check {
        items.extend(clippy_autotests(&manifest_paths, cfg.num_points));
    }

    let commit_thresholds = cfg.resolve_commit_thresholds();
    if !commit_thresholds.is_empty() {
        items.extend(commit_count_autotests(
            commit_thresholds.into_iter(),
            cfg.num_points,
        ))
    }

    // ---- Branch counting logic ----------------------------------------------
    if !cfg.require_branches.is_empty() {
        items.extend(branch_count_autotests(
            cfg.require_branches.iter().copied(),
            cfg.num_points,
        ));
    }
    // ---- Test count steps ----------------------------------------------------
    if cfg.require_tests > 0 {
        items.extend(test_count_autotests(
            &manifest_paths,
            cfg.num_points,
            cfg.require_tests,
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
