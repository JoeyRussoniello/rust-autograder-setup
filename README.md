# autograder-setup

[![Latest release](https://img.shields.io/github/v/release/JoeyRussoniello/rust-autograder-setup?display_name=tag&sort=semver)](https://github.com/JoeyRussoniello/rust-autograder-setup/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/JoeyRussoniello/rust-autograder-setup/total)](https://github.com/JoeyRussoniello/rust-autograder-setup/releases)
[![Release status](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/release.yaml/badge.svg)](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/release.yaml)
[![Build](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/ci.yaml/badge.svg)](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/ci.yaml)

A tiny Rust CLI that bootstraps GitHub Classroom autograding for Rust projects.

- `autograder-setup init` scans your `tests/` folder and builds a `tests/autograder.json` config, making it quick and consistent to set up assignments without manually tracking test cases.  
- `autograder-setup build` turns that config into a ready-to-run GitHub Actions workflow at `.github/workflows/classroom.yaml`, so you don’t need to hand-edit YAML for every homework.  
- `autograder-setup table` reads `tests/autograder.json` and generates a Markdown table for assignment READMEs, giving students a transparent overview of each test, its purpose, and its point value.  

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

# 1) Scan tests/ and create tests/autograder.json
autograder-setup init

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
```

### Command Reference

#### `init`

Scans `tests/` (recursively), finds test functions, and writes `tests/autograder.json`.

Options:

```bash
-r, --root <path>        Project root (default: .)
    --default-points <n> Default points per test (default: 1)
    --no-style-check     Disable Clippy style checks (enabled by default)
```

Examples:

```bash
autograder-setup init --root ../student-assignment
autograder-setup init --default-points 5
autograder-setup init --no-style-check
```

##### JSON Output

Schema:

| Field   | Type   | Required | Description                                  |
| ------- | ------ | -------- | -------------------------------------------- |
| name    | string | yes      | Display name in the workflow and test filter |
| timeout | number | yes      | Seconds for the autograder step (default 10) |
| points  | number | yes      | Max score for this test (default 1)          |
| docstring| string| yes      | The docstring pulled from the test case      |

Example:

```json
[
  { "name": "test_func_1", "timeout": 10, "points": 1, "docstring": "a test function"},
  { "name": "test_func_2", "timeout": 10, "points": 1, "docstring": ""}
]
```

---

#### `build`

Generates `.github/workflows/classroom.yaml` from `tests/autograder.json`.

Options:

```bash
-r, --root <path>        Project root (default: .)
```

Examples:

```bash
autograder-setup build
autograder-setup build --root ../student-assignment
```

##### YAML Output

Emits `.github/workflows/classroom.yaml` with:

- A fixed preamble (permissions, checkout, Rust toolchain),
- One autograding step per entry in `autograder.json`,
- A final reporter step that wires up `${{ steps.<id>.outputs.result }}` into an autograder report.

Name/ID rules:

- **Step name** / `test-name`: uses name verbatim.
- **Step id**: slugified `name` (lowercase; spaces/non-alnum → `-`; collapsed).
- **Command**: `cargo test <name> -- --exact` (uses name verbatim).

Example:

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
          command: "cargo test basic_add_small_numbers -- --exact"
          timeout: 10
          max-score: 1

      - name: basic_add_with_negatives
        id: basic-add-with-negatives
        uses: classroom-resources/autograding-command-grader@v1
        with:
          test-name: "basic_add_with_negatives"
          setup-command: ""
          command: "cargo test basic_add_with_negatives -- --exact"
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

Reads `tests/autograder.json` and generates a Markdown table of test names, docstrings, and points.
By default, the table is copied to the clipboard. Use  `--no-clipboard` to print to stdout instead.

Options:

```bash
-r, --root <path>        Project root (default: .)
    --no-clipboard       Do not copy the table to clipboard (print to terminal instead)
```

Examples:

```bash
# Copy a table to clipboard (default)
autograder-setup table

# Print table to stdout
autograder-setup table --no-clipboard

# Run against another directory
autograder-setup table --root ../student-assignment
```

**Markdown Output**
Example Table for an assigment

| Test name              | Description                            | Points |
|------------------------|----------------------------------------|--------|
| add_core               | Add function works in the core case    | 10     |
| add_small_numbers      | Add function works with small numbers  | 5      |
| add_with_negatives     | Add function handles negative inputs   | 3      |
| clippy_style_check     | Clippy linting check                   | 2      |

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
    │   │   ├── mod.rs
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
