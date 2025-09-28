// Contents of utility bash scripts.

// Singelton struct to hold script names as constants
pub struct ScriptNames {
    pub commit_count: &'static str,
    pub branch_count: &'static str,
}
pub const SCRIPT_NAMES: ScriptNames = ScriptNames {
    commit_count: "commit_count.sh",
    branch_count: "branch_count.sh",
};

// A shell script that ensures at least `n` commits exist in the git history
pub const COMMIT_COUNT_SCRIPT_CONTENTS: &str = r#"#!/usr/bin/env bash
# .autograder/commit_count.sh
set -euo pipefail

# Usage:
#   bash .autograder/commit_count.sh 3
#   MIN=3 bash .autograder/commit_count.sh

MIN="${1:-${MIN:-0}}"

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
"#;

// A shell script that ensures at least `n` branches with commits exist in the git history
pub const BRANCH_COUNT_SCRIPT_CONTENTS: &str = r#"#!/bin/bash

# Git Branch Analysis Script
# Usage: ./analyze_branches.sh [n_branches]

show_usage() {
    echo "Usage: $0 [n_branches]"
    echo ""
    echo "Checks how many unique branches have commits and prints results."
    echo "Exits 0 if at least n_branches exist, otherwise exits 1."
    echo ""
    echo "Example:"
    echo "  $0 5   # Require at least 5 branches with commits"
}

# Get all unique branch names from commit history that had at least one commit
get_unique_branches() {
    git log --all --pretty=format:"%D" \
      | grep -o '[^,)]*' \
      | grep -v '^$' \
      | grep -v 'HEAD' \
      | sed 's/origin\///' \
      | sort -u
}

# Parse argument
if [ $# -ne 1 ]; then
    show_usage
    exit 1
fi

N_BRANCHES=$1
if ! [[ $N_BRANCHES =~ ^[0-9]+$ ]]; then
    echo "Error: Argument must be a number"
    show_usage
    exit 1
fi

# Count branches
UNIQUE_BRANCHES=$(get_unique_branches)
BRANCH_COUNT=$(echo "$UNIQUE_BRANCHES" | wc -l)

# Always print status
echo "🔎 Found $BRANCH_COUNT branches with commits"
echo "$UNIQUE_BRANCHES"

# Decide success/failure
if [ $BRANCH_COUNT -ge $N_BRANCHES ]; then
    echo "✅ Success: At least $N_BRANCHES branches with commits"
    exit 0
else
    echo "❌ Failure: Only $BRANCH_COUNT branches found, need at least $N_BRANCHES"
    exit 1
fi
"#;
