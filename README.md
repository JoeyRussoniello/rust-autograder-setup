# autograder-setup

[![Latest release](https://img.shields.io/github/v/release/JoeyRussoniello/rust-autograder-setup?display_name=tag&sort=semver)](https://github.com/JoeyRussoniello/rust-autograder-setup/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/JoeyRussoniello/rust-autograder-setup/total)](https://github.com/JoeyRussoniello/rust-autograder-setup/releases)
[![Release status](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/release.yml/badge.svg)](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/release.yml)

A tiny Rust CLI that bootstraps GitHub Classroom autograding for Rust projects.

- `autograder-setup init` scans your `tests/` folder for test functions and creates `tests/autograder.json`.
- `autograder-setup build` reads tests/autograder.json and generates a ready-to-run workflow at `.github/workflows/classroom.yaml`.

Designed for simple, reproducible classroom templates. No need to hand-edit YAML for every assignment.

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

## Quick Start

### Installation

#### Option A — Install from release (recommended)

##### macOS

```bash
# 1) Download the macOS asset from the latest release
# 2) Extract and install:
tar -xzf autograder-setup-vX.Y.Z-x86_64-apple-darwin.tar.gz
sudo install -m 0755 autograder-setup-vX.Y.Z-x86_64-apple-darwin/autograder-setup /usr/local/bin/autograder-setup
autograder-setup --version
```

##### Windows (PowerShell)

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

#### Option B - Build from source

```bash
git clone https://github.com/JoeyRussoniello/rust-autograder-setup
cd rust-autograder-setup
cargo build --release

# binary at target/release/autograder-setup. Add to PATH, or migrate binary to the working
# directory of the desired assignment
```

### Usage

Once the binary is on your PATH

```bash
# 1) Create a config from existing tests

autograder-setup init

# 2) (Optional) Edit tests/autograder.json to adjust points/timeouts

# 3) Generate the GitHub Actions workflow

autograder-setup build

# -> .github/workflows/classroom.yaml

```

Use `--root <path>` to point at a different project:

```bash
autograder-setup --root ../student-assignment init
autograder-setup --root ../student-assignment build
```

---

## Outputs

### `autograder-setup init`

Scans `tests/` (recursively), finds test functions (attributes containing `test`, e.g. `#[test]`, `#[tokio::test]`, `#[cfg_attr(..., test)]`), and writes:

```json
[
  { "name": "test_func_1",          "timeout": 10, "points": 0 },
  { "name": "test_func_2",   "timeout": 10, "points": 0  }
]
```

See an example json configuration in [`tests/autograder.json`](./tests/autograder.json)

Defaults are `timeout: 10`, `points: 1`. Edit as needed.

### `autograder-setup build`

Emits `.github/workflows/classroom.yaml` with:

- A fixed preamble (permissions, checkout, Rust toolchain).
- One autograding step per test in `autograder.json`.
- A final reporter step that wires up `${{steps.<id>.outputs.result}}`.

#### Name/ID rules

- **Step name** / `test-name`: uses the `name` field verbatim.
- **Step `id`**: slug of the name — lowercase, spaces and non-alnum → `-`, collapsed (e.g., Corner Cases → `corner-cases`).
- **Command** `cargo test name`: also uses the `name` field verbatim

---

## CLI

```bash
autograder-setup [--root <path>] <COMMAND>

Commands:
  init   Scan tests/ and create tests/autograder.json
  build  Generate .github/workflows/classroom.yaml from tests/autograder.json

Options:
  -r, --root <path>   Project root (default: .)
  -h, --help          Show help
  -V, --version       Show version
```

---

## `autograder.json` schema

| Field     | Type   | Required | Description                                  |
| --------- | ------ | -------- | -------------------------------------------- |
| `name`    | string | yes      | Display name in the workflow and test filter |
| `timeout` | number | yes      | Seconds for the autograder step (default 10) |
| `points`  | number | yes      | Max score awarded by this test (default 1)   |

---

## Example Workflow Snippet (Output)

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

      - name: Basic
        id: basic
        uses: classroom-resources/autograding-command-grader@v1
        with:
          test-name: "Basic"
          setup-command: ""
          command: cargo test basic
          timeout: 10
          max-score: 15

      - name: Corner Cases
        id: corner-cases
        uses: classroom-resources/autograding-command-grader@v1
        with:
          test-name: "Corner Cases"
          setup-command: ""
          command: cargo test corner
          timeout: 10
          max-score: 5

      - name: Autograding Reporter
        uses: classroom-resources/autograding-grading-reporter@v1
        env:
          BASIC_RESULTS: "${{steps.basic.outputs.result}}"
          CORNER-CASES_RESULTS: "${{steps.corner-cases.outputs.result}}"
        with:
          runners: basic,corner-cases
```

---

## Repository Structure

```bash
.
├── Cargo.toml
├── src
│   ├── cli
│   │   ├── build.rs  # renders the workflow yaml
│   │   ├── init.rs   # scans tests and writes autograder.json
│   │   └── mod.rs    
│   ├── main.rs
│   ├── types.rs      # Shared Structs (AutoTest)
│   └── utils.rs      # Shared Utility Functions (file walking/checking)
└── tests
    ├── autograder.json
    └── main.rs
```

---

## Development

```bash
# lint/format (if installed via toolchain step)
cargo fmt --all
cargo clippy --all-targets --all-features

# run
cargo run -- init
cargo run -- build

# tests
cargo test
```

---

## Upcoming Features

- Flags to add linting steps to the autograder json and configured YAML
- Markdown table support to export test cases and documentation to template READMEs
