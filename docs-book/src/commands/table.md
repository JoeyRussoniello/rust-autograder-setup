# Command: `table`

Reads `.autograder/autograder.json` and generates a Markdown table of test names, descriptions, and points.

## Options

```bash
-r, --root <ROOT>   Root of the Rust project [default: .]
    --no-clipboard  Print to stdout instead of copying to clipboard
    --to-readme     Append the table to README.md
-h, --help          Print help
```

## Examples

```bash
autograder-setup table            # copy to clipboard
autograder-setup table --no-clipboard
autograder-setup table --root ../student-assignment --to-readme
```

## Example Output

| Test name                | Description                            | Points |
|--------------------------|----------------------------------------|--------|
| `add_core`               | Add function works in the core case    | 10     |
| `add_small_numbers`      | Add function works with small numbers  | 5      |
| `add_with_negatives`     | Add function handles negative inputs   | 3      |
| `clippy_style_check`     | Clippy linting check                   | 2      |