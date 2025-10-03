# Quickstart

The goal of `autograder-setup` is to take a plain Rust project template and make it **GitHub Classroom–ready in under a minute**.  
This quickstart shows the minimal workflow instructors need to get up and running.

---

## Step 0: Explore the CLI

```bash
autograder-setup --help
```

This prints top-level usage and shows all available subcommands. Use `--help` after any command to see its options.

---

## Step 1: Initialize an autograder configuration

```bash
autograder-setup init
```

- Scans your project recursively for all `#[test]` functions.  
- Builds a JSON config at `.autograder/autograder.json`.  
- Automatically adds optional checks (Clippy linting, commit count) unless disabled.  

Why it’s useful: this JSON file acts as a **single source of truth** for grading. You can tweak points, timeouts, or thresholds before generating CI.

---

## Step 2: Review and adjust points

Open `.autograder/autograder.json` in your editor:

```json
[
  {
    "meta": { "name": "add_two", "description": "check add_two works", "points": 2, "timeout": 10 },
    "type": "cargo_test",
    "manifest_path": "Cargo.toml"
  }
]
```

You can increase/decrease point values, set timeouts, or change descriptions here. This makes grading **customizable**.

---

## Step 3: Build the GitHub Actions workflow

```bash
autograder-setup build
```

- Reads `.autograder/autograder.json`.  
- Emits `.github/workflows/classroom.yaml` with one job per test/check.  
- Uses the official `classroom-resources/autograding-*` actions.  

Why it’s useful: this workflow is what Classroom actually runs when grading. It ensures consistency between local tests and CI.

---

## (Optional) Step 4: Generate a grading table

```bash
autograder-setup table --to-readme
```

- Produces a Markdown table of test names, descriptions, and points.  
- Appends to your `README.md`, or copy to clipboard by default.  

Why it’s useful: students can **see exactly how they’ll be graded** up front.

---

## Step 5: Reset if needed

```bash
autograder-setup reset
```

Deletes `.autograder/` and the generated workflow, so you can start fresh.

---

## Summary

- `init` = discover tests and set up config  
- `build` = turn config into a ready-to-run workflow  
- `table` = generate a transparent grading table for students  
- `reset` = undo everything  

Together, these steps let you go from **zero → reproducible autograder** in under a minute, while keeping grading criteria explicit for both instructors and students.
