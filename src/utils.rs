use anyhow::Result;
use std::path::{Path, PathBuf};
use std::{fs, };

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