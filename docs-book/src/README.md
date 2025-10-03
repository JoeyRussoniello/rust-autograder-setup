# Overview

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
