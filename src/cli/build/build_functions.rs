use anyhow::{Context, Result};
use std::fs::{create_dir_all, write};
use std::path::Path;
// Import all utility script constants
use crate::utils::scripts::*;
/// Generates the YAML preamble for the GitHub Actions workflow file.
pub fn get_yaml_preamble(on_push: bool) -> String {
    let mut triggers = vec!["repository_dispatch"];

    if on_push {
        triggers.push("push");
    }
    let triggers_joined = triggers.join(", ");

    let preamble = format!(
        r#"name: Autograding Tests
on: [{}]

permissions:
  checks: write
  actions: read
  contents: read

jobs:
  run-autograding-tests:
    runs-on: ubuntu-latest
    if: github.actor != 'github-classroom[bot]'
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          # Checkout with fetch depth 0 to get a full git history for commit counting
          fetch-depth: 0
    
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy,rustfmt

"#,
        triggers_joined
    );

    preamble
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
