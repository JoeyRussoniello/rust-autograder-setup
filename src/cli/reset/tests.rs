use super::*;
use tempfile::tempdir;
use std::fs::{self, File};

#[test]
fn removes_autograder_dir() {
    let dir = tempdir().unwrap();
    let autograder = dir.path().join(".autograder");
    fs::create_dir(&autograder).unwrap();
    File::create(autograder.join("dummy.txt")).unwrap();

    assert!(autograder.exists());
    run(dir.path()).unwrap();
    assert!(!autograder.exists());
}

#[test]
fn removes_classroom_yml() {
    let dir = tempdir().unwrap();
    let workflows = dir.path().join(".github").join("workflows");
    fs::create_dir_all(&workflows).unwrap();
    let classroom_yml = workflows.join("classroom.yml");
    File::create(&classroom_yml).unwrap();

    assert!(classroom_yml.exists());
    run(dir.path()).unwrap();
    assert!(!classroom_yml.exists());
}

#[test]
fn does_nothing_if_files_missing() {
    let dir = tempdir().unwrap();
    // No .autograder or classroom.yml
    assert!(run(dir.path()).is_ok());
}