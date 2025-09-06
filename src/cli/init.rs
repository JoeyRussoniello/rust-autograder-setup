use anyhow::{Context, Result};
use regex::Regex;
use std::{collections::BTreeSet, fs, io::Write, path::Path};

use crate::types::AutoTest;
use crate::utils::{collect_rs_files, ensure_exists};

pub fn run(root: &Path, num_points: u32, style_check: bool) -> Result<()> {
    let tests_dir = root.join("tests");
    ensure_exists(&tests_dir)?;

    let files = collect_rs_files(&tests_dir)
        .with_context(|| format!("While scanning {}", tests_dir.to_string_lossy()))?;
    if files.is_empty() {
        anyhow::bail!("No `.rs` files found under {}", tests_dir.to_string_lossy());
    }

    let mut names: BTreeSet<String> = BTreeSet::new();
    for file in files {
        let src = fs::read_to_string(&file)
            .with_context(|| format!("Failed to read {}", file.to_string_lossy()))?;
        for n in extract_test_names(&src) {
            names.insert(n);
        }
    }

    if names.is_empty() {
        anyhow::bail!("Found no test functions (looked for #[test]/#[...::test])");
    }

    let out_dir = root.join("tests");
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("Failed to create {}", out_dir.to_string_lossy()))?;
    let out_path = out_dir.join("autograder.json");

    let mut items: Vec<AutoTest> = names
        .into_iter()
        .map(|name| AutoTest {
            name,
            timeout: 10,
            points: num_points,
        })
        .collect();
    if style_check {
        items.push(AutoTest {
            name: "CLIPPY_STYLE_CHECK".to_string(),
            timeout: 10,
            points: num_points,
        });
    }

    let json = serde_json::to_string_pretty(&items)?;
    let mut f = fs::File::create(&out_path)
        .with_context(|| format!("Failed to create {}", out_path.to_string_lossy()))?;
    f.write_all(json.as_bytes())?;

    println!("Wrote {}", out_path.to_string_lossy());
    Ok(())
}

/// Very simple extractor:
/// 1) Strip line/block comments
/// 2) Find attributes that contain `test` (e.g., #[test], #[tokio::test], #[my::attr(test)])
/// 3) Grab the following `fn <name>`
fn extract_test_names(src: &str) -> Vec<String> {
    let cleaned = strip_comments(src);

    // Attribute that contains "test" followed by fn <ident>
    // - Accepts optional pub/async between attribute and fn
    // - Handles #[test], #[tokio::test], #[cfg_attr(..., test)], etc. (anything containing "test")
    let re = Regex::new(
        r#"#\s*\[[^\]]*test[^\]]*\]\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)"#,
    )
    .unwrap();

    re.captures_iter(&cleaned)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

fn strip_comments(s: &str) -> String {
    // Remove block comments (/* ... */), then line comments (// ...).
    // This is intentionally simple and may remove comment-like text in strings,
    // but is fine for attribute scanning in test files.
    let block = Regex::new(r"(?s)/\*.*?\*/").unwrap().replace_all(s, "");
    let line = Regex::new(r"//.*").unwrap().replace_all(&block, "");
    line.to_string()
}
