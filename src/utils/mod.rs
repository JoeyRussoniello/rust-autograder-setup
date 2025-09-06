use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

//pub static DEFAULT_POINTS: u32 = 1;
pub const YAML_PREAMBLE: &str = r#"name: Autograding Tests
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

"#;

pub const YAML_INDENT: &str = "  ";
pub fn collect_rs_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    recurse(dir, &mut out)?;
    Ok(out)
}

fn recurse(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        let md = entry.metadata()?;
        if md.is_dir() {
            recurse(&p, out)?;
        } else if md.is_file() && p.extension().map(|e| e == "rs").unwrap_or(false) {
            out.push(p);
        }
    }
    Ok(())
}

pub fn ensure_exists(tests_dir: &Path) -> Result<()> {
    if !tests_dir.exists() {
        anyhow::bail!(
            "No `tests/` directory found at {}",
            tests_dir.to_string_lossy()
        );
    }
    Ok(())
}

// Lowercase; spaces/non-alnum -> hyphens; collapse/trim hyphens.
pub fn slug_id(name: &str) -> String {
    let mut s = String::new();
    let mut last_dash = false;
    for ch in name.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            s.push(c);
            last_dash = false;
        } else if !last_dash {
            s.push('-');
            last_dash = true;
        }
    }
    // trim leading/trailing dashes
    while s.starts_with('-') {
        s.remove(0);
    }
    while s.ends_with('-') {
        s.pop();
    }
    // collapse multiple dashes already handled by last_dash flag
    s
}

// Quote for YAML (simple: double-quote and escape double quotes)
pub fn yaml_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\\\""))
}

#[cfg(test)]
pub mod tests;
