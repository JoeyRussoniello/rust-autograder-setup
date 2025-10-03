# autograder-setup

<div align="center">

[![Latest release](https:img.shields.io/github/v/release/JoeyRussoniello/rust-autograder-setup?display_name=tag&sort=semver)](https:github.com/JoeyRussoniello/rust-autograder-setup/releases/latest)&nbsp;&nbsp;
[![Downloads](https:img.shields.io/github/downloads/JoeyRussoniello/rust-autograder-setup/total)](https:github.com/JoeyRussoniello/rust-autograder-setup/releases)&nbsp;&nbsp;
[![Release status](https:github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/release.yaml/badge.svg)](https:github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/release.yaml)&nbsp;&nbsp;
[![Build](https:github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/ci.yaml/badge.svg)](https:github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/ci.yaml)&nbsp;&nbsp;
[![Docs](https:img.shields.io/badge/docs-mdBook-blue)](https:joeyrussoniello.github.io/rust-autograder-setup/)

</div>

A tiny Rust CLI that bootstraps **GitHub Classroom autograding for Rust projects**.  

> 🚀 Currently deployed in Boston University’s *Intro to Rust* course (130+ students, 1000+ student repos).

## Key Features

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
- [Installation](#installation)
- [Quickstart](#quickstart)
- [Usage](#usage)
- [Repository Structure](#repository-structure)
- [Upcoming Features](#upcoming-features)

---

## 📦 Releases

- **Latest:** [https:github.com/JoeyRussoniello/rust-autograder-setup/releases/latest](https:github.com/JoeyRussoniello/rust-autograder-setup/releases/latest)
- **All releases:** [https:github.com/JoeyRussoniello/rust-autograder-setup/releases](https:github.com/JoeyRussoniello/rust-autograder-setup/releases)

## Installation

### Option A — Install via Cargo (recommended)

If you already have Rust installed:  

```bash
cargo install autograder-setup
```

Check installation:

```bash
autograder-setup --version
```

---

### Option B — Download a prebuilt binary

Precompiled binaries are available on the latest GitHub release:  
<https:github.com/JoeyRussoniello/rust-autograder-setup/releases/latest>

| OS / Target                  | Archive  | Notes                                                   |
|------------------------------|----------|---------------------------------------------------------|
| macOS (x86_64-apple-darwin)  | `.tar.gz` | Extract and install to `/usr/local/bin`                 |
| Windows (x86_64-pc-windows-gnu) | `.zip` | Extract and move `autograder-setup.exe` to your `PATH` |

> See the docs for detailed OS-specific instructions:  
> <https:joeyrussoniello.github.io/rust-autograder-setup/installation.html>

## Quickstart

```bash
# Show top-level help
autograder-setup --help

# 1) Scan src/ recursively and create .autograder/autograder.json
autograder-setup init

# 2) (Optional) Edit tests/autograder.json to adjust points/timeouts

# 3) Generate the GitHub Actions workflow
autograder-setup build
# -> .github/workflows/classroom.yaml
```

For command-specific flags:

```bash
autograder-setup init --help
autograder-setup build --help
autograder-setup table --help
autograder-setup reset --help
```

## Usage

For a full CLI guide and usage instructions, see the [Complete Documentation](https:joeyrussoniello.github.io/rust-autograder-setup/)

## Repository Structure

```bash
.
├── Cargo.lock                           # Cargo dependency lockfile (generated; checked in for reproducible builds)
├── Cargo.toml                           # Crate metadata and dependencies
├── LICENSE                              # Project license
├── README.md                            # Basic installation and usage guide (this file)
├── docs-book                            # Complete mdbook documentation
│   ├── book.toml
│   └── src
│       ├── README.md
│       ├── SUMMARY.md
│       ├── commands
│       │   ├── build.md
│       │   ├── init.md
│       │   ├── reset.md
│       │   └── table.md
│       ├── faq.md
│       ├── installation.md
│       ├── json-schema.md
│       ├── quickstart.md
│       ├── releases.md
│       └── repository-structure.md
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
    │   │   ├── scan                     # Module for AST parsing and test case discovery
    │   │   │   ├── mod.rs
    │   │   │   └── tests.rs
    │   │   └── tests.rs                 
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
