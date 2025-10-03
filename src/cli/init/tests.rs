// Tests and test harness for init cli
use crate::cli::RunConfig;
use crate::types::{AutoTest, TestKind};
use crate::utils::read_autograder_config;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};

/// -------- Minimal, builder-style harness for filesystem + run() --------

struct Harness {
    _tmp: TempDir,
    root: PathBuf,
}

impl Harness {
    fn new() -> Self {
        let tmp = tempdir().expect("tempdir");
        Self {
            root: tmp.path().to_path_buf(),
            _tmp: tmp,
        }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    /// Write a simple root crate.
    fn write_root_crate(&self, src: &str, package_name: &str) {
        fs::create_dir_all(self.root.join("src")).unwrap();
        fs::write(
            self.root.join("Cargo.toml"),
            format!("[package]\nname=\"{}\"\nversion=\"0.1.0\"\n", package_name),
        )
        .unwrap();
        fs::write(self.root.join("src/lib.rs"), src).unwrap();
    }

    /// Write a workspace root (no root package), optional members list.
    fn write_workspace_root(&self, members: &[&str]) {
        let mut toml = String::from("[workspace]\n");
        if !members.is_empty() {
            let quoted = members
                .iter()
                .map(|m| format!("\"{}\"", m))
                .collect::<Vec<_>>()
                .join(", ");
            toml.push_str(&format!("members=[{}]\n", quoted));
        }
        fs::write(self.root.join("Cargo.toml"), toml).unwrap();
        fs::create_dir_all(self.root.join("src")).unwrap(); // keep a src/ for misc
    }

    /// Write a member crate.
    fn write_member_crate(&self, member: &str, src: &str) {
        let base = self.root.join(member);
        fs::create_dir_all(base.join("src")).unwrap();
        fs::write(
            base.join("Cargo.toml"),
            format!("[package]\nname=\"{}\"\nversion=\"0.1.0\"\n", member),
        )
        .unwrap();
        fs::write(base.join("src/lib.rs"), src).unwrap();
    }

    /// Run the generator with a configurable RunConfig (via closure) and return the produced steps.
    fn run(&self, build: impl FnOnce(&mut RunConfig)) -> Vec<AutoTest> {
        let mut cfg = RunConfig {
            // sensible test defaults; override in `build` if needed
            tests_dir_name: self.root.clone(),
            ..Default::default()
        };
        // Always bind to this temp repo
        cfg.root = self.root.clone();

        build(&mut cfg);

        super::run(&cfg).expect("run() failed");
        read_autograder_config(self.root()).expect("read_autograder_config failed")
    }
}

/// -------- Focused step helpers (no giant trait needed) --------

fn commit_mins(items: &[AutoTest]) -> Vec<u32> {
    items
        .iter()
        .filter_map(|t| match t.kind {
            TestKind::CommitCount { min_commits } => Some(min_commits),
            _ => None,
        })
        .collect()
}

fn test_count_mins_by_manifest(items: &[AutoTest]) -> Vec<(Option<String>, u32)> {
    items
        .iter()
        .filter_map(|t| match &t.kind {
            TestKind::TestCount {
                min_tests,
                manifest_path,
            } => {
                // manifest_path: Option<String>
                Some((manifest_path.clone(), *min_tests))
            }
            _ => None,
        })
        .collect()
}

fn has_member_named(items: &[AutoTest], needle: &str) -> bool {
    items.iter().any(|t| t.meta.name.contains(needle))
}

/// ------------------- commit-count config generation -------------------

#[test]
fn creates_commit_steps_from_require_commits() {
    let h = Harness::new();
    h.write_root_crate("#[test] fn a() {}", "root");

    let items = h.run(|c| {
        c.commit_counts_flag = true;
        c.num_commit_checks = Some(0);
        c.require_commits = vec![5, 10, 20];
    });

    assert_eq!(commit_mins(&items), vec![5, 10, 20]);
}

#[test]
fn require_commits_overrides_num_commit_checks() {
    let h = Harness::new();
    h.write_root_crate("#[test] fn a() {}", "root");

    let items = h.run(|c| {
        c.commit_counts_flag = true;
        c.num_commit_checks = Some(4); // would expand to 1..=4, but must be ignored
        c.require_commits = vec![2, 8, 16]; // precedence
    });

    assert_eq!(commit_mins(&items), vec![2, 8, 16]);
}

#[test]
fn expands_num_commit_checks_when_require_commits_missing() {
    let h = Harness::new();
    h.write_root_crate("#[test] fn a() {}", "root");

    let items = h.run(|c| {
        c.commit_counts_flag = true;
        c.num_commit_checks = Some(3); // deprecated but still expands
        // require_commits empty -> use num_commit_checks
    });

    assert_eq!(commit_mins(&items), vec![1, 2, 3]);
}

#[test]
fn does_not_create_commit_steps_if_disabled() {
    let h = Harness::new();
    h.write_root_crate("#[test] fn a() {}", "root");

    let items = h.run(|c| {
        c.commit_counts_flag = false; // disabled
        c.num_commit_checks = Some(99); // ignored
        c.require_commits = vec![1, 2, 3, 4, 5]; // ignored
    });

    assert!(commit_mins(&items).is_empty());
}

/// ------------------- test-count config generation (samples) -------------------

#[test]
fn creates_single_test_count_step_for_root_when_required() {
    let h = Harness::new();
    h.write_root_crate("#[test] fn a() {}", "root");

    let items = h.run(|c| {
        c.require_tests = vec![3]; // require at least 3 tests for root
    });

    let pairs = test_count_mins_by_manifest(&items);
    assert_eq!(pairs.len(), 1, "expected exactly one TEST_COUNT (root)");
    assert_eq!(pairs[0], (None, 3)); // None => root (no manifest_path stored)
    assert!(items.iter().any(|t| t.meta.name.contains("TEST_COUNT")));
}

#[test]
fn creates_test_count_steps_for_each_manifest_in_workspace() {
    let h = Harness::new();
    h.write_workspace_root(&["member"]);
    h.write_member_crate("member", "#[test] fn member_test() {}");
    fs::write(h.root().join("src/lib.rs"), "#[test] fn root_test() {}").unwrap();

    let items = h.run(|c| {
        c.num_points = 2;
        c.style_check = true;
        c.require_tests = vec![5];
    });

    // Expect two TestCount steps: one root (None), one member ("member/Cargo.toml")
    let mut pairs = test_count_mins_by_manifest(&items);
    pairs.sort_by(|a, b| a.0.cmp(&b.0)); // stable assertion order
    assert_eq!(
        pairs,
        vec![(None, 5), (Some("member/Cargo.toml".into()), 5)]
    );

    // Names: root is "TEST_COUNT", member includes uppercased path
    assert!(items.iter().any(|t| t.meta.name.contains("TEST_COUNT")));
    assert!(has_member_named(&items, "MEMBER/CARGO.TOML"));
}
