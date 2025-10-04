use anyhow::{Context, Result};
use std::fs::{create_dir_all, write};
use std::path::Path;
// Import all utility script constants
use crate::cli::helpers::build::{Preset, Trigger, resolve_triggers};
use crate::utils::scripts::*;

// TODO: Rewrite this using a struct to hold `on` options and
/// Generates the YAML preamble for the GitHub Actions workflow file.
pub fn get_yaml_preamble(preset: Option<Preset>, explicit_triggers: &[Trigger]) -> String {
    use Trigger::*;

    let chosen = resolve_triggers(preset, explicit_triggers);

    // Always-on baseline:
    let mut lines: Vec<String> = vec![
        "name: Autograding Tests".into(),
        "on:".into(),
        "  repository_dispatch:".into(),
    ];

    if chosen.contains(&WorkflowDispatch) {
        lines.push("  workflow_dispatch:".into());
    }
    if chosen.contains(&Push) {
        // Keep as a block so we can drop in branches/paths later
        lines.push("  push:".into());
        // (future) lines.push("    branches:"); etc.
    }
    if chosen.contains(&PullRequest) {
        lines.push("  pull_request:".into());
        // (future) lines.push("    types:"); etc.
    }

    // static permissions + job header
    lines.extend(
        [
            "".to_string(),
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
        ]
        .map(String::from),
    );

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
