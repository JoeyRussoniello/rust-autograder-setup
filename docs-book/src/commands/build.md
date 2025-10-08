# Command: `build`

Generates `.github/workflows/classroom.yaml` from `.autograder/autograder.json`
(and a commit-count script if needed).

## Options

```bash
-r, --root <ROOT>
        Root directory of the Rust project (defaults to current directory)
        [default: .]

    --triggers <TRIGGERS>...
        Add one or more event triggers (always includes `repository_dispatch`)

        Examples:
          --triggers push pull-request

        Possible values:
          - workflow-dispatch: Manual run via the Actions UI
          - push:              Run on git push
          - pull-request:      Run on PR events

    --preset <PRESET>
        Quick config for common classroom setups

        Possible values:
          - instructor:   Instructor-driven grading; manual-friendly
          - student-push: Students get feedback on push
          - student-pr:   Students get feedback on PRs

    --branches <BRANCHES>...
        Restrict pushes and PRs to these branches

        Example:
          --branches main feature

    --paths <PATHS>...
        Only grade when changes affect these files or paths

        Example:
          --paths src/** Cargo.toml

    --paths-ignore <PATHS_IGNORE>...
        Ignore pushes and PRs that *only* affect these files or paths

        Example:
          --paths-ignore README.md docs/**

-h, --help
        Print help (see a summary with '-h')
```

## Examples

```bash
autograder-setup build
autograder-setup build --root ../student-assignment

# Run on every push and pull request
autograder-setup build --triggers push pull-request

# Restrict to main branch only
autograder-setup build --triggers push pull-request --branches main

# Ignore documentation-only changes
autograder-setup build --triggers push --paths-ignore README.md docs/**
```

## Workflow details

- Fixed preamble (permissions, checkout, Rust toolchain).
- One autograding step per entry in `autograder.json`.
- Final reporter step wiring `${{ steps.<id>.outputs.result }}` into the report.

### **Name/ID rules**

- Step `name` / `test-name`: verbatim for `cargo test` entries; ALL_CAPS for other steps (e.g., `CLIPPY_STYLE_CHECK`).
- Step `id`: slugified `name` (lowercase; spaces & non-alnum → `-`).
- Command: `cargo test <name>`.

### Workflow triggers (`on:`)

> By default, the generated workflow uses: `on: [repository_dispatch]`. This allows instructors to trigger grading manually from the Classroom UI.

Running with flags modifies the triggers:

- `--triggers push` — run the autograder on every push.
- `--triggers push pull-request` — run on every push and pull request.
- `--branches` — restrict which branches are eligible for push/PR grading.
- `--paths` and `--paths-ignore` — limit workflow runs to specific file changes.

Notes about choosing triggers:

- `repository_dispatch` is the safe default for instructor-initiated grading (avoids CI load on every push).
- `push` or `pull-request` triggers give students instant feedback but increase compute usage.
- Combine `--branches`, `--paths`, and `--paths-ignore` to control when grading runs.

## Example Workflow YAML

```yaml
name: Autograding Tests
on: [repository_dispatch, push]

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
          fetch-depth: 0

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
          CLIPPY-STYLE-CHECK_RESULTS: "${{steps.clippy-style-check.outputs.result}}"
        with:
          runners: basic-add-small-numbers,clippy-style-check
```
