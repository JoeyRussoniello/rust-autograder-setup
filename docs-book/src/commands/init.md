# Command: `init`

Scans the project (recursively), finds test functions, and writes `.autograder/autograder.json`. Supports nested Rust directories.

## Options

```bash
-r, --root <ROOT>            Root of the Rust project [default: .]
-t, --tests-dir <TESTS_DIR>  Location of tests (defaults to <root>) [default: .]
    --default-points <N>     Default points per test [default: 1]
    --no-style-check         Disable the Clippy style check (enabled by default)
    --no-commit-count        Disable Commit Counting (enabled by default)
    --require-commits ...    Require specific commit thresholds (e.g. 5 10 15 20) [default: 1]
    --require-branches ...   Require specific branch thresholds (e.g. 2 4 6) [default: 0]
    --require-tests [N]      Require a minimum number of tests (default: 0; set to 1 if flag is passed without a value)
-h, --help                   Print help
```

## Examples

```bash
# Initialize for a sibling repo
autograder-setup init --root ../student-assignment

# Only search ./tests recursively
autograder-setup init --tests-dir tests

# Default 5 points per test
autograder-setup init --default-points 5

# Omit style or commit checks
autograder-setup init --no-style-check
autograder-setup init --no-commit-count

# Require at least 5 tests
autograder-setup init --require-tests 5

# Shortcut: require at least 1 test
autograder-setup init --require-tests

# Award 1 point for reaching 5, 10, and 20 commits
autograder-setup init --require-commits 5 10 20

# Award points for 10 and 20 commits, and for having 2 and 3 branches
autograder-setup init --require-commits 10 20 --require-branches 2 3
```

## Commit counting

By default, the generator creates a single commit check requiring **at least 1 commit**.  
Override with `--require-commits` to specify multiple thresholds:

```bash
--require-commits 5 10 15
```

Produces three independent checks:

- ✅ 1 point for reaching 5 commits
- ✅ 1 point for reaching 10 commits
- ✅ 1 point for reaching 15 commits

Each threshold can also be fine-tuned directly in `.autograder/autograder.json`.

> **Deprecated:** `--num-commit-checks N` expands to thresholds `1..=N` (e.g., `3` → `1 2 3`). Prefer `--require-commits`.