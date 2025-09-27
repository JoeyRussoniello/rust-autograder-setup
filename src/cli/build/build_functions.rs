use super::create_and_write;
use anyhow::Result;
use std::path::Path;
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

//TODO: refactor to write a single shell script that takes an input
/// Write a shell script to check for a minimum number of commits.
pub fn write_commit_count_shell(root: &Path, num_commits: u32, name: &str) -> Result<()> {
    let script_path = root.join(".autograder").join(name);
    // Shell script content
    let script = format!(
        r#"#!/usr/bin/env bash
# tests/commit_count.sh
set -euo pipefail

# Usage:
#   MIN=3 bash tests/commit_count.sh
#   bash tests/commit_count.sh -m 3

MIN={min}

# Validate MIN
if ! [[ "$MIN" =~ ^[0-9]+$ ]]; then
  echo "MIN must be a non-negative integer; got: '$MIN'" >&2
  exit 2
fi

# Ensure we're in a git repo
if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "Not a git repository (are you running inside the checkout?)" >&2
  exit 1
fi

# Warn if shallow (runner must checkout with fetch-depth: 0 for full history)
if [ -f "$(git rev-parse --git-dir)/shallow" ]; then
  echo "Warning: shallow clone detected; commit count may be incomplete." >&2
fi

# Count commits
COUNT=$(git rev-list --count HEAD 2>/dev/null || echo 0)

if [ "$COUNT" -ge "$MIN" ]; then
  echo "✅ Found $COUNT commits (min $MIN) — PASS"
  exit 0
else
  echo "❌ Found $COUNT commits (min $MIN) — FAIL"
  exit 1
fi
"#,
        min = num_commits
    );

    // Write the file
    create_and_write(&script_path, &script)?;

    println!(
        "Wrote commit count shell to {}",
        script_path.to_string_lossy()
    );
    Ok(())
}
