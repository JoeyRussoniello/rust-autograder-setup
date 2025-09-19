use crate::types::AutoTest;
use anyhow::{Context, Result};
use std::fs;
use std::io::BufReader;
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

/// A wrapper struct used to simplify manifest path locations
pub struct RustFile {
    pub path: PathBuf,
    pub manifest_path: Option<PathBuf>,
}
impl RustFile {
    pub fn get_path_string(&self) -> Result<String> {
        fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read {}", &self.path.to_string_lossy()))
    }
}
fn recurse(dir: &Path, out: &mut Vec<RustFile>, current_manifest: Option<PathBuf>) -> Result<()> {
    // see if THIS dir has a Cargo.toml
    let manifest_here = {
        let m = dir.join("Cargo.toml");
        if m.exists() {
            Some(m)
        } else {
            current_manifest.clone()
        }
    };

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        let md = entry.metadata()?;

        if md.is_dir() {
            recurse(&p, out, manifest_here.clone())?;
        } else if md.is_file() && p.extension().map(|e| e == "rs").unwrap_or(false) {
            out.push(RustFile {
                path: p,
                manifest_path: manifest_here.clone(),
            });
        }
    }
    Ok(())
}

pub fn collect_rs_files_with_manifest(dir: &Path) -> Result<Vec<RustFile>> {
    let mut out: Vec<RustFile> = Vec::new();
    recurse(dir, &mut out, None)?;
    Ok(out)
}

pub fn ensure_exists(tests_dir: &Path) -> Result<()> {
    if !tests_dir.exists() {
        anyhow::bail!("Nothing found at {}", tests_dir.to_string_lossy());
    }
    Ok(())
}

pub fn read_autograder_config(root: &Path) -> Result<Vec<AutoTest>> {
    let path = root.join(".autograder").join("autograder.json");
    ensure_exists(&path)?;
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let tests: Vec<AutoTest> = serde_json::from_reader(reader)?;

    if tests.is_empty() {
        anyhow::bail!("Autograder.json config not configured. Add tests using `auto-setup init`");
    }

    // Validation: min_commits only allowed for COMMIT_COUNT*
    for t in &tests {
        let is_commit = t.name.trim().starts_with("COMMIT_COUNT");
        if t.min_commits.is_some() && !is_commit {
            anyhow::bail!(
                "Field `min_commits` is only valid for COMMIT_COUNT steps (offending test: `{}`)",
                t.name
            );
        }
    }

    Ok(tests)
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

pub fn replace_double_hashtag(s: String, num_commits: u32) -> String {
    s.replace("##", &num_commits.to_string())
}

// Convert absolute path under `root` into a clean, unix-style relative string for GH actions
pub fn to_rel_unix_path(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path).to_path_buf();
    rel.to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

/// Safe wrapper to prevent accidental joins when root = tests_dir
pub fn get_tests_dir(root: &Path, tests_dir_name: &Path) -> PathBuf {
    let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let tests_abs = tests_dir_name
        .canonicalize()
        .unwrap_or_else(|_| tests_dir_name.to_path_buf());

    if root_abs == tests_abs {
        root.to_path_buf()
    } else if tests_dir_name.is_absolute() {
        tests_dir_name.to_path_buf()
    } else {
        root.join(tests_dir_name)
    }
}

#[cfg(test)]
pub mod tests;
