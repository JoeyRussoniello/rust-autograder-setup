use super::*;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

// -------- collect_rs_files / recurse --------

#[test]
fn collect_rs_files_finds_rs_recursively_and_ignores_others() -> anyhow::Result<()> {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;

    // Arrange: temp dir with mixed contents
    let tmp = tempdir()?;
    let root = tmp.path();

    // root files
    let hello_rs = root.join("hello.rs");
    File::create(&hello_rs)?.write_all(b"// rust file")?;

    let readme = root.join("README.md");
    File::create(&readme)?.write_all(b"# not rust")?;

    // nested dir a/
    let a_dir = root.join("a");
    fs::create_dir(&a_dir)?;
    let a_mod_rs = a_dir.join("mod.rs");
    File::create(&a_mod_rs)?.write_all(b"// rust file")?;

    // nested dir a/b/
    let b_dir = a_dir.join("b");
    fs::create_dir(&b_dir)?;
    let deep_rs = b_dir.join("deep.rs");
    File::create(&deep_rs)?.write_all(b"// rust file")?;

    let deep_txt = b_dir.join("deep.txt");
    File::create(&deep_txt)?.write_all(b"not rust")?;

    let found = collect_rs_files_with_manifest(root)?;
    let mut files: Vec<&PathBuf> = found.iter().map(|(p, _)| p).collect();

    // Order of read_dir is not guaranteed; normalize for assertions
    files.sort();

    // Assert
    let mut expected = vec![&hello_rs, &a_mod_rs, &deep_rs];
    expected.sort();

    assert_eq!(files, expected);

    // Order of read_dir is not guaranteed; normalize for assertions
    files.sort();

    // Assert: exactly the three .rs files, no others
    let mut expected = vec![&hello_rs, &a_mod_rs, &deep_rs];
    expected.sort();

    assert_eq!(files, expected);

    Ok(())
}

#[test]
fn collect_rs_files_empty_dir_is_ok() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let files = collect_rs_files_with_manifest(tmp.path())?;
    assert!(files.is_empty());
    Ok(())
}

// -------- ensure_exists --------

#[test]
fn ensure_exists_ok_when_dir_exists() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    // create a subdir named tests to mirror typical usage
    let tests_dir = tmp.path().join("tests");
    fs::create_dir(&tests_dir)?;
    assert!(ensure_exists(&tests_dir).is_ok());
    Ok(())
}

#[test]
fn ensure_exists_err_when_missing() {
    let tmp = tempdir().unwrap();
    let missing = tmp.path().join("tests"); // doesn’t exist
    let err = ensure_exists(&missing).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("Nothing found at"), "got: {msg}");
    assert!(
        msg.contains("\\") | msg.contains("/"),
        "message should surface the path"
    );
}

// -------- slug_id --------

#[test]
fn slug_id_basic_lowercases_and_replaces_non_alnum_with_single_dashes() {
    assert_eq!(slug_id("HelloWorld"), "helloworld");
    assert_eq!(slug_id("Hello World"), "hello-world");
    assert_eq!(slug_id("hello_world"), "hello-world"); // underscore → dash
    assert_eq!(slug_id("hello---world"), "hello-world"); // collapse
    assert_eq!(slug_id(" hello\tworld "), "hello-world"); // trim/collapse
    assert_eq!(slug_id("A  B   C"), "a-b-c");
    assert_eq!(slug_id("--Already-Slug--"), "already-slug"); // trims ends
}

#[test]
fn slug_id_handles_unicode_by_treating_non_ascii_as_separators() {
    // Non-ASCII letters aren’t considered ascii_alphanumeric → become dashes
    assert_eq!(slug_id("naïve Café — test"), "na-ve-caf-test");
    // Emojis / punctuation collapse to single dashes
    assert_eq!(slug_id("ok✅?no!"), "ok-no");
}

#[test]
fn slug_id_only_dashes_or_symbols_yields_empty_or_clean_slug() {
    assert_eq!(slug_id("----"), "");
    assert_eq!(slug_id("   "), "");
    assert_eq!(slug_id("!!!abc???"), "abc");
}

// -------- yaml_quote --------

#[test]
fn yaml_quote_wraps_in_double_quotes_and_escapes_internal_quotes() {
    assert_eq!(yaml_quote("plain"), "\"plain\"");
    assert_eq!(yaml_quote("he said \"hi\""), "\"he said \\\"hi\\\"\"");
    assert_eq!(yaml_quote("a:b"), "\"a:b\""); // just wrapped; no extra escaping
    assert_eq!(yaml_quote("path\\with\\slashes"), "\"path\\with\\slashes\"");
    // No escaping of backslashes/newlines by design (doc states simple quote)
    assert_eq!(yaml_quote("line1\nline2"), "\"line1\nline2\"");
}

// -------- convenience: helper to compare by filenames when needed --------
#[allow(dead_code)]
fn filenames(paths: &[PathBuf]) -> Vec<String> {
    let mut v: Vec<String> = paths
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    v.sort();
    v
}

#[test]
fn collect_rs_files_attaches_nearest_manifest() -> anyhow::Result<()> {
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    let tmp = tempdir()?;
    let root = tmp.path();

    // workspace under a/ (has its own Cargo.toml)
    let a_dir = root.join("a");
    fs::create_dir(&a_dir)?;
    File::create(a_dir.join("Cargo.toml"))?
        .write_all(b"[package]\nname=\"a\"\nversion=\"0.1.0\"\n")?;
    let a_test = a_dir.join("mod.rs");
    File::create(&a_test)?.write_all(b"// rust file")?;

    // deeper file inherits a/'s Cargo.toml
    let b_dir = a_dir.join("b");
    fs::create_dir(&b_dir)?;
    let b_test = b_dir.join("deep.rs");
    File::create(&b_test)?.write_all(b"// rust file")?;

    // root rust file with NO manifest at root
    let root_rs = root.join("root.rs");
    File::create(&root_rs)?.write_all(b"// rust file")?;

    let found = collect_rs_files_with_manifest(root)?;

    // helper to lookup manifest path for a given .rs
    let manifest_for = |path: &std::path::Path| -> Option<std::path::PathBuf> {
        found
            .iter()
            .find_map(|(p, m)| if p == path { m.clone() } else { None })
    };

    assert_eq!(manifest_for(&a_test), Some(a_dir.join("Cargo.toml")));
    assert_eq!(manifest_for(&b_test), Some(a_dir.join("Cargo.toml")));
    assert_eq!(manifest_for(&root_rs), None);

    Ok(())
}
