use anyhow::Result;
use std::path::Path;

use crate::utils::scripts::COMMIT_COUNT_SCRIPT_CONTENTS;
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

/// Write a single parameterized shell script to count commits in a git repository.
pub fn write_commit_count_shell(root: &Path) -> Result<()> {
    let script_path = root.join(".autograder").join("commit_count.sh");

    // Bail early if the script already exists
    if script_path.exists() {
        return Ok(());
    }

    std::fs::create_dir_all(script_path.parent().unwrap())?;
    std::fs::write(&script_path, COMMIT_COUNT_SCRIPT_CONTENTS)?;

    Ok(())
}
