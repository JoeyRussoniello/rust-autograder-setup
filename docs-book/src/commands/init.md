# Command: `init`

Scans the project (recursively), finds test functions, and writes `.autograder/autograder.json`. Supports nested Rust directories.

## Options

```bash
-r, --root <ROOT>
        Root of the Rust project (defaults to current directory)

        [default: .]

-t, --tests-dir <TESTS_DIR>
        Location of all test cases (defaults to <root>)

        [default: .]

    --default-points <DEFAULT_POINTS>
        Default number of points per test

        [default: 1]

    --no-style-check
        Disable the Clippy style check (enabled by default)

    --require-commits <REQUIRE_COMMITS>...
        Require specific commit thresholds (e.g. --require-commits 5 10 15 20)

        [default: 1]

    --require-branches <REQUIRE_BRANCHES>...
        Require specific branch tresholds (e.g --require-branches 2 4 6)

    --require-tests <REQUIRE_TESTS>...
        Require specific student-written test thresholds (e.g --require-tests 2 4 6)

-h, --help
        Print help (see a summary with '-h')
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

# Award 1 point for reaching 5, 10, and 20 commits
autograder-setup init --require-commits 5 10 20

# Award points for 10 and 20 commits, and for having 2 and 3 branches
autograder-setup init --require-commits 10 20 --require-branches 2 3
```

## Counting checks (commits, branches, tests)

The `init` command can emit simple threshold checks that award 1 point each when a submission meets a given threshold. The three related flags behave the same way: each value supplied becomes an independent 1‑point check.

- `--require-commits <N>...`
  - Each value produces a check that the submission has at least N commits.
  - Example: `--require-commits 5 10 20` → three checks (5, 10, 20 commits).

- `--require-branches <N>...`
  - Each value produces a check that the repository has at least N distinct branches.
  - Example: `--require-branches 2 4` → two checks (2 branches, 4 branches).

- `--require-tests <N>...`
  - Each value produces a check that the student-written test count for a crate reaches at least N tests.
  - **IMPORTANT:** `--require-tests` applies per manifest path. For a workspace, a threshold value produces a separate check for the root crate and for each member crate (i.e., each manifest path gets its own check).
  - Example: in a workspace with `member/` and a root crate, `--require-tests 3` yields a `TEST_COUNT` check for the root (if present) and a `TEST_COUNT` check for `member` (each requiring 3 tests).
    - This behavior can be refined by changing/removing the entries in `.autograder.json`

Examples

```bash
# Award 1 point for reaching 5, 10, and 20 commits
autograder-setup init --require-commits 5 10 20

# Award 1 point for having 2 and 4 branches
autograder-setup init --require-branches 2 4

# Require at least 3 tests for each manifest (root and each workspace member)
autograder-setup init --require-tests 3
```

Notes

- Each supplied value becomes an independent 1‑point check (not cumulative).
- Deprecated: `--num-commit-checks N` expands to thresholds `1..=N` (e.g., `--num-commit-checks 3` → `1 2 3`). Prefer `--require-commits` for explicit thresholds.
