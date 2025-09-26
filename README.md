# autograder-setup

<div align="center">

[![Latest release](https://img.shields.io/github/v/release/JoeyRussoniello/rust-autograder-setup?display_name=tag&sort=semver)](https://github.com/JoeyRussoniello/rust-autograder-setup/releases/latest)&nbsp;&nbsp;
[![Downloads](https://img.shields.io/github/downloads/JoeyRussoniello/rust-autograder-setup/total)](https://github.com/JoeyRussoniello/rust-autograder-setup/releases)&nbsp;&nbsp;
[![Release status](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/release.yaml/badge.svg)](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/release.yaml)&nbsp;&nbsp;
[![Build](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/ci.yaml/badge.svg)](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/ci.yaml)

</div>

A tiny Rust CLI that bootstraps GitHub Classroom autograding for Rust projects.

- `autograder-setup init` scans for test cases across multiple rust projects and builds a `.autograder/autograder.json` config file, making it quick, easy, and consistent to set up assignments.
- `autograder-setup build` turns that config into a ready-to-run GitHub Actions workflow at `.github/workflows/classroom.yaml`, removing the need to hand-edit YAML for every homework.  
- `autograder-setup table` reads `.autograder/autograder.json` and generates a Markdown table for assignment READMEs, giving students a transparent overview of each test, its purpose, and its point value.  

Keeps autograding setup **simple for instructors** while making grading criteria **transparent for students**.

---

## Table of Contents

- [Releases](#-releases)
  - [Prebuilt binaries](#prebuilt-binaries)
- [Installation](#installation)
  - [Option A — Install from release](#option-a--install-from-release-recommended)
    - [macOS](#macos)
    - [Windows (PowerShell)](#windows-powershell)
  - [Option B — Build from source](#option-b---build-from-source)
- [Usage](#usage)
  - [Quickstart](#quickstart)
  - [Command Reference](#command-reference)
    - [init](#init)
    - [build](#build)
    - [table](#table)
    - [reset](#reset)
- [Repository Structure](#repository-structure)
- [Upcoming Features](#upcoming-features)

---

## 📦 Releases

- **Latest:** [https://github.com/JoeyRussoniello/rust-autograder-setup/releases/latest](https://github.com/JoeyRussoniello/rust-autograder-setup/releases/latest)
- **All releases:** [https://github.com/JoeyRussoniello/rust-autograder-setup/releases](https://github.com/JoeyRussoniello/rust-autograder-setup/releases)

### Prebuilt binaries

| OS / Target | Download |
|---|---|
| macOS (x86_64-apple-darwin) | See **Assets** on the [latest release](https://github.com/JoeyRussoniello/rust-autograder-setup/releases/latest) |
| Windows (x86_64-pc-windows-gnu) | See **Assets** on the [latest release](https://github.com/JoeyRussoniello/rust-autograder-setup/releases/latest) |

> Assets are named: `autograder-setup-vX.Y.Z-<target>.tar.gz` (macOS) or `.zip` (Windows).

---

## Installation

### Option A — Install from release (recommended)

#### macOS

```bash
# 1) Download the macOS asset from the latest release
# 2) Extract and install:
tar -xzf autograder-setup-vX.Y.Z-x86_64-apple-darwin.tar.gz
sudo install -m 0755 autograder-setup-vX.Y.Z-x86_64-apple-darwin/autograder-setup /usr/local/bin/autograder-setup
autograder-setup --version
```

#### Windows (PowerShell)

```powershell
# 1) Download the Windows .zip from the latest release
# 2) Extract and install:
Expand-Archive autograder-setup-vX.Y.Z-x86_64-pc-windows-gnu.zip -DestinationPath .

$dir = Get-ChildItem -Directory "autograder-setup-v*-x86_64-pc-windows-gnu" | Select-Object -First 1
$exe = Join-Path $dir.FullName "autograder-setup.exe"

$UserBin = "$env:USERPROFILE\.local\bin"
New-Item -ItemType Directory -Force -Path $UserBin | Out-Null
Move-Item $exe "$UserBin\autograder-setup.exe" -Force

# Add to PATH for current session (optionally add permanently in System settings)
$env:PATH = "$UserBin;$env:PATH"
autograder-setup --version
```

### Option B - Build from source

```bash
git clone https://github.com/JoeyRussoniello/rust-autograder-setup
cd rust-autograder-setup
cargo build --release

# binary at target/release/autograder-setup. Add to PATH, or migrate binary to the working
# directory of the desired assignment
```

## Usage

### Quickstart

```bash
# Show top-level help
autograder-setup --help

# 1) Scan src/ recursively and create tests/autograder.json
autograder-setup init

# OR scan tests/  for tests if the assignment is a packages (uses lib.rs instead of main/mod.rs)
autograder-setup --tests-dir tests

# 2) (Optional) Edit tests/autograder.json to adjust points/timeouts

# 3) Generate the GitHub Actions workflow
autograder-setup build
# -> .github/workflows/classroom.yaml
```

To see flags for a specific command:

```bash
autograder-setup init --help
autograder-setup build --help
autograder-setup table --help
autograder-setup reset --help
```

### Command Reference

#### `init`

Scans `.` (recursively), finds test functions, and writes `.autograder/autograder.json`. Offers support for nested rust directorie.

Options:

```bash
  -r, --root <ROOT>
          Root of the Rust project (defaults to current directory) [default: .]
  -t, --tests-dir <TESTS_DIR>
          Location of all test cases (defaults to <root>) [default: .]
      --default-points <DEFAULT_POINTS>
          Default number of points per test [default: 1]
      --no-style-check
          Disable the Clippy style check (enabled by default)
      --no-commit-count
          Disable Commit Counting (enabled by default)
      --num-commit-checks <NUM_COMMIT_CHECKS>
          Number of commit count checks (default: 1) [default: 1]
      --require-tests [<REQUIRE_TESTS>]
          Require a minimum number of tests (default: 0, set to 1 if flag is passed without a value) [default: 0]
  -h, --help
          Print help
```

Examples:

```bash
# Initialize an autograder.json in ../student-assignment/.autograder
autograder-setup init --root ../student-assignment

# Initialize an autograder.json by searching ./tests/ recursively
autograder-setup init --tests-dir tests

# Initialize autograder.json with 5 as the default amount of points instead of 1
autograder-setup init --default-points 5

# Omit the style check or commit counting steps of the autograder build
autograder-setup init --no-style-check
autograder-setup init --no-commit-count

# Create multiple commit check steps for awarding partial credit
autograder-setup init --num-commit-checks 3

# Require at least 5 tests in the project
autograder-setup init --require-tests 5

# Require at least 1 test (shortcut: omit value)
autograder-setup init --require-tests
```

>Note: When commit counting is enabled, the generator creates separate checks for each threshold up to num-commit-checks.
> For example, --num-commit-checks 3 would produce three independent checks:
>
> - 1 point for reaching 1 commits
> - 1 point for reaching 2 commits
> - 1 point for reaching 3 commits
>
> The number of commits required to earn a point can be tweaked in `autograder.json`
> This lets you award partial credit as students make more commits.

##### JSON Output

Schema:

| Field           | Type   | Required | Description                                                                               |
| --------------- | ------ | -------- | ----------------------------------------------------------------------------------------- |
| `name`          | string | yes      | Display name in the workflow and test filter                                              |
| `timeout`       | number | yes      | Seconds for the autograder step (default 10)                                              |
| `points`        | number | yes      | Max score for this test (default 1)                                                       |
| `docstring`     | string | yes      | The docstring pulled from the test case                                                   |
| `min_commits`   | number | no       | Minimum number of commits required for a `COMMIT_COUNT` step (only present if applicable) |
| `min_tests`     | number | no       | Minimum number of tests required for a `TEST_COUNT` step (only present if applicable)     |
| `manifest_path` | string | no       | Path to the Rust project’s `Cargo.toml` for the test’s crate (only present if applicable) |

Example:

```json
[
  {
    "name": "test_func_1",
    "timeout": 10,
    "points": 1,
    "docstring": "a test function",
    "manifest_path": "Cargo.toml"
  },
  {
    "name": "COMMIT_COUNT_0",
    "timeout": 10,
    "points": 1,
    "docstring": "Ensures at least ## commits.",
    "min_commits": 5
  },
  {
    "name": "TEST_COUNT",
    "timeout": 10,
    "points": 1,
    "docstring": "Ensures at least ## tests exist.",
    "min_tests": 3
  }
]

```

> Note: The `##` characters in `COMMIT_COUNT` and `TEST_COUNT` steps can be left as-is, and will be replaced on `autograder-setup table` runs

> Note: For `TEST_COUNT` checks. The autograder will look for `min_tests` plus the amount of `cargo test` statemenets are left in the `.autograder.json` file
> when running `autograder-setup build`
>
> Ex: I released a homework with **5** autograder test cases and want the students to add **3** test cases. The autograder will ensure **8** test cases to receive a point
---

#### `build`

Generates `.github/workflows/classroom.yaml` from `.autograder/autograder.json`, as well as any required commit counting shell scripts.

Options:

```bash
-r, --root <ROOT>
        Root of the Rust project (defaults to current directory) [default: .]
    --grade-on-push
        Have autograder run on push to any branch (default: grade only on "Grade All" or `repository_dispatch`)
-h, --help
        Print help
```

Examples:

```bash
autograder-setup build
autograder-setup build --root ../student-assignment

# Setup the autograder to run every time the students pushes to their repo
# (Ideal for small class sizes)
autograder-setup build --grade-on-push
```

##### YAML Output

Emits `.github/workflows/classroom.yaml` with:

- A fixed preamble (permissions, checkout, Rust toolchain),
- One autograding step per entry in `autograder.json`,
- A final reporter step that wires up `${{ steps.<id>.outputs.result }}` into an autograder report.

Name/ID rules:

- **Step name** / `test-name`: uses name verbatim for all `cargo test` functions, and all caps names for other autograder steps (ex: `COMMIT_COUNTS`).
- **Step id**: slugified `name` (lowercase; spaces/non-alnum → `-`; collapsed).
- **Command**: `cargo test <name>` (uses name verbatim).
  - In future update, the CLI will use the `--exact` flag, but for now, this behavior is unsupported **so NO tests should share a name/prefix**

Workflow triggers (`on:`):

By default the generated workflow uses:

- `on: [repository_dispatch]` — this lets instructors trigger grading from the classroom UI or a manual dispatch without running on every push.

If you run autograder-setup build `--grade-on-push`, the preamble will include:

- `on: [repository_dispatch, push]` — the workflow will also run on every push to the repository (all branches).

Notes about choosing triggers:

- `repository_dispatch` is the safe default for instructor-initiated grading (avoid running CI for every student push and balooning compute costs)
- `--grade-on-push` is convenient for immediate feedback during development but may increase CI usage and spurious runs; consider branch/paths filters if you want to limit when pushes trigger the autograder.

Example Output:

```yaml
name: Autograding Tests
on: [push, repository_dispatch]

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

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy,rustfmt

      - name: basic_add_small_numbers
        id: basic-add-small-numbers
        uses: classroom-resources/autograding-command-grader@v1
        with:
          test-name: "basic_add_small_numbers"
          setup-command: ""
          command: "cargo test basic_add_small_numbers"
          timeout: 10
          max-score: 1

      - name: basic_add_with_negatives
        id: basic-add-with-negatives
        uses: classroom-resources/autograding-command-grader@v1
        with:
          test-name: "basic_add_with_negatives"
          setup-command: ""
          command: "cargo test basic_add_with_negatives"
          timeout: 10
          max-score: 1

      - name: CLIPPY_STYLE_CHECK
        id: clippy-style-check
        uses: classroom-resources/autograding-command-grader@v1
        with:
          test-name: "CLIPPY_STYLE_CHECK"
          setup-command: ""
          command: "cargo clippy -- -D warnings"
          timeout: 10
          max-score: 1

      - name: Autograding Reporter
        uses: classroom-resources/autograding-grading-reporter@v1
        env:
          BASIC-ADD-SMALL-NUMBERS_RESULTS: "${{steps.basic-add-small-numbers.outputs.result}}"
          BASIC-ADD-WITH-NEGATIVES_RESULTS: "${{steps.basic-add-with-negatives.outputs.result}}"
          CLIPPY-STYLE-CHECK_RESULTS: "${{steps.clippy-style-check.outputs.result}}"
        with:
          runners: basic-add-small-numbers,basic-add-with-negatives,clippy-style-check
```

---

#### `table`

Reads `.autograder/autograder.json` and generates a Markdown table of test names, docstrings, and points.
By default, the table is copied to the clipboard. Use  `--no-clipboard` to print to stdout instead, and `--to-readme` to append to the `README.md` file in the `root` directory.

Options:

```bash
  -r, --root <ROOT>
          Root of the Rust project (defaults to current directory) [default: .]
      --no-clipboard
          Do not copy the table to clipboard (print to terminal instead)
      --to-readme
          Append the table to the end of README.md
  -h, --help
          Print help
```

Examples:

```bash
# Copy a table to clipboard (default)
autograder-setup table

# Print table to stdout
autograder-setup table --no-clipboard

# Run against another directory and append the table to the readme directly
autograder-setup table --root ../student-assignment --to-readme
```

**Markdown Output**
Example Table for an assigment

| Test name                | Description                            | Points |
|--------------------------|----------------------------------------|--------|
| `add_core`               | Add function works in the core case    | 10     |
| `add_small_numbers`      | Add function works with small numbers  | 5      |
| `add_with_negatives`     | Add function handles negative inputs   | 3      |
| `clippy_style_check`     | Clippy linting check                   | 2      |

#### `reset`

Reset the autograder setup by deleting all created files (`.autograder` directory, and `.github/workflows/classroom.yml`)

Options

```bash
  -r, --root <ROOT>
          Root of the Rust project (defaults to current directory) [default: .]
  -h, --help
          Print help
```

Example

```bash
# Undo all setup done by the autograder-setup file
autograder-setup reset
```

## Repository Structure

```bash
.
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
└── src
    ├── cli
    │   ├── build           # renders the workflow yaml
    │   │   ├── mod.rs
    │   │   └── tests.rs
    │   ├── init            # scans tests and writes autograder.json
    │   │   ├── mod.rs
    │   │   └── tests.rs
    │   ├── table           # scans tests and creates a markdown table
    │   │   └── mod.rs
    │   ├── reset           # Removes any files created by the CLI
    │   │   ├── mod.rs
    │   │   └── tests.rs
    │   └── mod.rs          # Core CLI logic (arg parsing, documentation)
    ├── main.rs
    ├── types.rs            # Shared Structs (AutoTest)
    └── utils               # Shared Utility Functions (file walking/checking)
        ├── mod.rs          
        └── tests.rs
```

---

## Upcoming Features

- Additional CLI improvements and configuration options
- Publish to `crates.io` for installation via `cargo install autograder-setup`
