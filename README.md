# auto-setup

A tiny Rust CLI that bootstraps GitHub Classroom autograding for Rust projects.

- `auto-setup init` scans your `tests/` folder for test functions and creates `tests/autograder.json`.
- `auto-setup build` reads tests/autograder.json and generates a ready-to-run workflow at `.github/workflows/classroom.yaml`.

Designed for simple, reproducible classroom templates—no hand-editing YAML every assignment.

---

## Quick Start

```bash
# From this repo

cargo build --release

# (binary at target/release/auto-setup)

# 1) Create a config from existing tests

./target/release/auto-setup init

# 2) (Optional) Edit tests/autograder.json to adjust points/timeouts

# 3) Generate the GitHub Actions workflow

./target/release/auto-setup build

# -> .github/workflows/classroom.yaml
```

Use `--root <path>` to point at a different project:

```bash
./target/release/auto-setup --root ../student-assignment init
./target/release/auto-setup --root ../student-assignment build
```

---

## Outputs

### `auto-setup init`

Scans `tests/` (recursively), finds test functions (attributes containing `test`, e.g. `#[test]`, `#[tokio::test]`, `#[cfg_attr(..., test)]`), and writes:

```json
[
  { "name": "test_func_1",          "timeout": 10, "points": 0 },
  { "name": "test_func_2",   "timeout": 10, "points": 0  }
]
```

See an emaple json configuration in [`tests/autograder.json`](./tests/autograder.json)

Defaults are `timeout: 10`, `points: 1`. Edit as needed.

### `auto-setup build`

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
auto-setup [--root <path>] <COMMAND>

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

## Repo Structure

## Repository Layout

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
