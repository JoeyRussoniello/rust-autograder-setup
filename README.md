# autograder-setup

<div align="center">

[![Latest release](https://img.shields.io/github/v/release/JoeyRussoniello/rust-autograder-setup?display_name=tag&sort=semver)](https://github.com/JoeyRussoniello/rust-autograder-setup/releases/latest)&nbsp;&nbsp;
[![Downloads](https://img.shields.io/github/downloads/JoeyRussoniello/rust-autograder-setup/total)](https://github.com/JoeyRussoniello/rust-autograder-setup/releases)&nbsp;&nbsp;
[![Release status](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/release.yaml/badge.svg)](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/release.yaml)&nbsp;&nbsp;
[![Build](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/ci.yaml/badge.svg)](https://github.com/JoeyRussoniello/rust-autograder-setup/actions/workflows/ci.yaml)&nbsp;&nbsp;
[![Docs](https://img.shields.io/badge/docs-mdBook-blue)](https://joeyrussoniello.github.io/rust-autograder-setup/)

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

- **Latest:** [https://github.com/JoeyRussoniello/rust-autograder-setup/releases/latest](https://github.com/JoeyRussoniello/rust-autograder-setup/releases/latest)
- **All releases:** [https://github.com/JoeyRussoniello/rust-autograder-setup/releases](https://github.com/JoeyRussoniello/rust-autograder-setup/releases)

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

## Quickstart

```bash
# Show top-level help
autograder-setup --help

# 1) Scan src/ recursively and create .autograder/autograder.json
autograder-setup init

# 2) (Optional) Edit .autograder/autograder.json to adjust points/timeouts

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

For usage instructions see the complete documentation

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
