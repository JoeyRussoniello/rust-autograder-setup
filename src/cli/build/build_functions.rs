use anyhow::{Context, Result};
use std::fs::{create_dir_all, write};
use std::path::Path;
// Import all utility script constants
use super::preamble::*;
use crate::cli::helpers::build::{Preset, Trigger};
use crate::utils::scripts::*;

/// Generates the YAML preamble for the GitHub Actions workflow file.
pub fn get_yaml_preamble(preset: Option<Preset>, explicit_triggers: &[Trigger]) -> String {
    let on_cfg = OnConfig::from_args(preset, explicit_triggers);

    let mut lines: Vec<String> = vec!["name: Autograding Tests".into()];

    // Render the `on:` block
    lines.extend(on_cfg.to_yaml_lines());

    // static permissions + job header
    lines.extend([
        "".into(),
        "permissions:".into(),
        "  checks: write".into(),
        "  actions: read".into(),
        "  contents: read".into(),
        "".into(),
        "jobs:".into(),
        "  run-autograding-tests:".into(),
        "    runs-on: ubuntu-latest".into(),
        "    if: github.actor != 'github-classroom[bot]'".into(),
        "    steps:".into(),
        "      - name: Checkout code".into(),
        "        uses: actions/checkout@v4".into(),
        "        with:".into(),
        "          # Checkout with fetch depth 0 to get a full git history for commit counting"
            .into(),
        "          fetch-depth: 0".into(),
        "".into(),
        "      - name: Install Rust toolchain".into(),
        "        uses: dtolnay/rust-toolchain@stable".into(),
        "        with:".into(),
        "          components: clippy,rustfmt".into(),
        "".into(),
    ]);

    lines.join("\n")
}

/// Shared function to write any script to the .autograder directory
fn write_script(root: &Path, script_name: &str, contents: &str) -> Result<()> {
    let script_path = root.join(".autograder").join(script_name);

    // Bail early if the script already exists
    if script_path.exists() {
        return Ok(());
    }

    create_dir_all(script_path.parent().unwrap()).with_context(|| {
        format!(
            "Failed to create directory for {}",
            script_path.to_string_lossy()
        )
    })?;
    write(&script_path, contents)
        .with_context(|| format!("Failed to write to {}", script_path.to_string_lossy()))?;
    Ok(())
}

/// Write a single parameterized shell script to count commits in a git repository.
pub fn write_commit_count_shell(root: &Path) -> Result<()> {
    write_script(
        root,
        SCRIPT_NAMES.commit_count,
        COMMIT_COUNT_SCRIPT_CONTENTS,
    )?;
    Ok(())
}

/// Write a single parameterized shell script to count branches in a git repository.
pub fn write_branch_count_shell(root: &Path) -> Result<()> {
    write_script(
        root,
        SCRIPT_NAMES.branch_count,
        BRANCH_COUNT_SCRIPT_CONTENTS,
    )?;
    Ok(())
}
