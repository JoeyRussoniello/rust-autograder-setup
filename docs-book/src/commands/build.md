# Command: `build`

Generates `.github/workflows/classroom.yaml` from `.autograder/autograder.json` (and a commit-count script if needed).

## Options

```bash
-r, --root <ROOT>   Root of the Rust project [default: .]
    --grade-on-push Run on push to any branch (default is only `repository_dispatch`)
-h, --help          Print help
```

## Examples

```bash
autograder-setup build
autograder-setup build --root ../student-assignment

# Run on every push (small classes)
autograder-setup build --grade-on-push
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

By default the generated workflow uses:

- `on: [repository_dispatch]` — this lets instructors trigger grading from the classroom UI or a manual dispatch without running on every push.

Running `autograder-setup build --grade-on-push`, the trigger will change to :

- `on: [repository_dispatch, push]` — the workflow will also run on every push to the repository (all branches).

Notes about choosing triggers:

- `repository_dispatch` is the safe default for instructor-initiated grading (avoid running CI for every student push and balooning compute costs)
- `--grade-on-push` is convenient for immediate feedback during development but may increase CI usage and spurious runs; consider branch/paths filters if you want to limit when pushes trigger the autograder.

## Example Workflow YAML

```yaml
name: Autograding Tests
on: [repository_dispatch]

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
