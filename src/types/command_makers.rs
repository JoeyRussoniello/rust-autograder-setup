// Helper functions for the `.command()` method on `AutoTest`
use crate::utils::scripts::SCRIPT_NAMES;

fn is_root_manifest(p: &str) -> bool {
    p.is_empty() || p == "Cargo.toml" || p == "."
}

fn manifest_flag(mp: Option<&str>) -> Option<String> {
    match mp {
        Some(p) if !is_root_manifest(p) => Some(format!("--manifest-path {}", p.trim())),
        _ => None,
    }
}

pub fn cargo_test_cmd(function: &str, mp: Option<&str>) -> String {
    // TODO: Reimplement let exact = format!("{} -- --exact", function.trim());
    match manifest_flag(mp) {
        Some(flag) => format!("cargo test {} {}", function.trim(), flag),
        None => format!("cargo test {}", function.trim()),
    }
}

pub fn clippy_cmd(mp: Option<&str>) -> String {
    match manifest_flag(mp) {
        Some(flag) => format!("cargo clippy {} -- -D warnings", flag),
        None => "cargo clippy -- -D warnings".to_string(),
    }
}

// Uses your existing “+##” placeholder convention so you can inject framework baseline later.
pub fn test_count_cmd(min: u32, mp: Option<&str>) -> String {
    let base = match manifest_flag(mp) {
        Some(flag) => format!("cargo test {} -- --list", flag),
        None => "cargo test -- --list".to_string(),
    };
    format!(
        r#"{base} | tail -1 | awk '{{print $1}}' | awk '{{if ($1 < {min}+##) {{print "Too few tests ("$1-##") expected {min}"; exit 1}}}}'"#,
        base = base,
        min = min
    )
}

pub fn commit_count_cmd(min_commits: &u32) -> String {
    format!("bash ./.autograder/{} {}", SCRIPT_NAMES.commit_count, min_commits,)
}

pub fn branch_count_cmd(min_branches: &u32) -> String {
    format!("bash ./.autograder/{} {}", SCRIPT_NAMES.branch_count, min_branches,)
}