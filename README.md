# autograder-setup

<div align="center">

[![Latest release](https://img.shields.io/github/v/release/JoeyRussoniello/rust-autograder-setup?display_name=tag&sort=semver)](https://github.com/JoeyRussoniello/rust-autograder-setup/releases/latest)&nbsp;&nbsp;
[![Downloads](https://img.shields.io/github/downloads/JoeyRussoniello/rust-autograder-setup/total)](https://github.com/JoeyRussoniello/rust-autograder-setup/releases)&nbsp;&nbsp;
[![Release status](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/release.yaml/badge.svg)](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/release.yaml)&nbsp;&nbsp;
[![Build](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/ci.yaml/badge.svg)](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/ci.yaml)

</div>

A tiny Rust CLI that bootstraps **GitHub Classroom autograding for Rust projects**.  

> 🚀 Currently deployed in Boston University’s *Intro to Rust* course (130+ students, 1000+ student repos).

## ✨ Key Features

- ⚡ **Fast setup** — go from repo → Classroom-ready assignment in under 60 seconds.  
- 📝 **Flexible outputs** — grading tables copied to clipboard *or* written directly to your README.  
- 🏎️ **Optimized CI** — precompiled YAMLs (no runtime parsing) for faster, cheaper runs.  
- 🔧 **Instructor-friendly CLI** — `init`, `build`, `table`, `reset` cover the full workflow.  

## How it Works

- **`init`** — scans for Rust tests and builds `.autograder/autograder.json`.  
- **`build`** — converts that config into a ready-to-run GitHub Actions workflow at `.github/workflows/classroom.yaml`.  
- **`table`** — generates a Markdown grading table for READMEs, keeping grading criteria transparent.  
- **`reset`** — cleans up generated files for a fresh start.  

Keeps autograding setup **simple for instructors** while making grading criteria **clear for students**.

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

# 3) Remove the Quarantine Attribute to disable MacOS Gatekeeper and code signing requirement.
sudo xattr -r -d com.apple.quarantine /usr/local/bin/autograder-setup

# 4) Check that you can run it
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
autograder-setup init --tests-dir tests

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

    --no-commit-count
        Disable Commit Counting (enabled by default)

    --require-commits <REQUIRE_COMMITS>...
        Require specific commit thresholds (e.g. --require-commits 5 10 15 20)

        [default: 1]

    --require-branches <REQUIRE_BRANCHES>...
        Require specific branch tresholds (e.g --require-branhes 2 4 6)

        [default: 0]

    --require-tests [<REQUIRE_TESTS>]
        Require a minimum number of tests (default: 0, set to 1 if flag is passed without a value)

        [default: 0]

-h, --help
        Print help (see a summary with '-h')
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

# Require at least 5 tests in the project
autograder-setup init --require-tests 5

# Require at least 1 test (shortcut: omit value)
autograder-setup init --require-tests

# Award 1 point for reaching 5 commits, a second for reaching 10, and a third for 20 
autograder-setup init --require-commits 5 10 20

# Award 1 point for reaching 10 and 20 commits,
# and 1 point each for having 2 and 3 branches
autograder-setup init --require-commits 10 20 --require-branches 2 3
```

**Commit counting:**  
By default, the generator creates a single commit check requiring **at least 1 commit**.  
You can override this with `--require-commits` to specify one or more thresholds:
//

```bash
--require-commits 5 10 15
```

produces three independent checks:

- ✅ 1 point for reaching 5 commits  
- ✅ 1 point for reaching 10 commits  
- ✅ 1 point for reaching 15 commits

This makes it easy to award **partial credit** as students commit more frequently.  
Each threshold can also be fine-tuned directly in `.autograder/autograder.json`.

> **Deprecated:** The old `--num-commit-checks N` option is still accepted, and expands to thresholds `1..=N`.  
>For example:
>
>```bash
>--num-commit-checks 3
>```
>
>is equivalent to `--require-commits 1 2 3`.  
>Please migrate to `--require-commits`, which offers clearer and more flexible control.

##### JSON Output

| Field                 | Type   | Req | Description                                                                 |
| --------------------- | ------ | --- | --------------------------------------------------------------------------- |
| `meta.name`           | string | yes | Display name in the workflow and test filter                                |
| `meta.description`    | string | yes | Student-facing description (supports `##` placeholder for counts)           |
| `meta.points`         | number | yes | Max score for this test (default 1)                                         |
| `meta.timeout`        | number | yes | Seconds for the autograder step (default 10)                                |
| `type`                | string | yes | One of: `cargo_test`, `clippy`, `commit_count`, `test_count`                |
| `manifest_path`       | string | no  | Path to `Cargo.toml` (for `cargo_test`, `clippy`, `test_count`)             |
| `min_commits`         | number | no  | Required commits (only for `commit_count`)                                  |
| `min_tests`           | number | no  | Required tests (only for `test_count`)                                      |

Example:

```json
[
  {
     "meta": { "name": "test_func_1", "description": "a test function", "points": 1, "timeout": 10 },
     "type": "cargo_test",
    "manifest_path": "Cargo.toml"
  },
  {
    "meta": { "name": "COMMIT_COUNT_1", "description": "Ensure at least ## commits.", "points": 1, "timeout": 10 },
    "type": "commit_count",
    "min_commits": 5
  },
  {
    "meta": { "name": "TEST_COUNT", "description": "Ensure at least ## tests exist.", "points": 1, "timeout": 10 },
    "type": "test_count",
    "min_tests": 3
  }
]
```

> Note: The `##` characters in `COMMIT_COUNT` and `TEST_COUNT` steps can be left as-is, and will be replaced on `autograder-setup table` runs

> Note: For `TEST_COUNT` checks. The autograder will look for `min_tests` plus the amount of `cargo test` statements that are present in the `.autograder.json` file
> when running `autograder-setup build`
>
> *Example*: For a homework with **5** autograder test cases, where we want the students to add **3** test cases the autograder will ensure **8** test cases to receive a point
---

#### `build`

Generates `.github/workflows/classroom.yaml` from `.autograder/autograder.json`, as well as the commit counting shell script if necessary.

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

Running `autograder-setup build --grade-on-push`, the trigger will change to :

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
├── Cargo.lock                           # Cargo dependency lockfile (generated; checked in for reproducible builds)
├── Cargo.toml                           # Crate metadata and dependencies
├── LICENSE                              # Project license
├── README.md                            # Documentation and usage guide (this file)
└── src
    ├── cli                              # CLI subcommands and orchestration
    │   ├── build                        # `autograder-setup build` — render workflow YAML from autograder.json
    │   │   ├── build_functions.rs       # Preamble, YAML helpers, commit-count script writer, small utilities
    │   │   ├── mod.rs                   # Subcommand entry + YAMLAutograder builder (ties everything together)
    │   │   ├── steps.rs                 # Hand-assembled YAML step emitters (CommandStep / ReporterStep)
    │   │   └── tests.rs                 # Unit tests for YAML rendering and build behavior
    │   ├── init                         # `autograder-setup init` — scan tests and write `.autograder/autograder.json`
    │   │   ├── functions.rs             # High-level constructors for AutoTests (clippy/commit count/test count)
    │   │   ├── mod.rs                   # Subcommand entry and pipeline glue
    │   │   ├── scan.rs                  # Rust source scanner (finds #[test]/#[..::test], docs, manifests)
    │   │   └── tests.rs                 # Parser/scan tests and manifest-path logic tests
    │   ├── mod.rs                       # Top-level CLI wiring (arg parsing, subcommand dispatch)
    │   ├── reset                        # `autograder-setup reset` — remove generated files
    │   │   ├── mod.rs                   # Subcommand entry
    │   │   └── tests.rs                 # Safety checks for deleting generated artifacts
    │   ├── table                        # `autograder-setup table` — generate student-facing Markdown table
    │   │   └── mod.rs                   # Subcommand entry and table rendering
    │   └── tests.rs                     # Cross-subcommand/integration-style tests for the CLI layer
    ├── main.rs                          # Binary entrypoint; delegates to `cli`
    ├── types                            # Core data model for the autograder
    │   ├── command_makers.rs            # Per-variant command builders (cargo test/clippy/test-count/commit-count)
    │   └── mod.rs                       # `AutoTest { meta, kind }`, `TestMeta`, `TestKind` + Markdown row impl
    └── utils
        ├── mod.rs                       # Shared helpers: path walking, slug/id, yaml_quote, replace_double_hashtag, etc.
        └── tests.rs                     # Unit tests for utilities
```

---

## Upcoming Features

- Additional CLI improvements and configuration options
- Publish to `crates.io` for installation via `cargo install autograder-setup`
