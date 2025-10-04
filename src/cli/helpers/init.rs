// Helper types for CLI arguments
use std::path::PathBuf;
use crate::cli::{InitArgs};

#[derive(Debug, Clone)]
/// A helper container struct to stabilize Init::run() with the new feature additions
pub struct RunConfig {
    pub root: std::path::PathBuf,
    pub tests_dir_name: std::path::PathBuf,

    pub num_points: u32,
    pub style_check: bool,

    // Old / deprecated:
    pub commit_counts_flag: bool,       // legacy on/off gate
    pub num_commit_checks: Option<u32>, // DEPRECATED

    // New preferred:
    pub require_tests: Vec<u32>,
    pub require_commits: Vec<u32>,
    pub require_branches: Vec<u32>,
}

impl RunConfig {
    /// Canonicalize commit thresholds once (back-compat):
    /// - If `require_commits` is non-empty, use it.
    /// - Else if `num_commit_checks` is Some(n), expand to 1..=n.
    /// - Else if legacy `commit_counts_flag` is true, default to [1].
    /// - Else empty (no commit checks).
    pub fn resolve_commit_thresholds(&self) -> Vec<u32> {
        // Precedence 1: Return empty vector on explicit no
        if !self.commit_counts_flag {
            return Vec::new();
        }
        // Then default to require_commits
        if !self.require_commits.is_empty() {
            return self.require_commits.clone();
        }
        // Then the legacy gateway
        if let Some(n) = self.num_commit_checks {
            return (1..=n).collect();
        }
        // If none specified require just 1
        if self.commit_counts_flag {
            return vec![1];
        }
        // Safety fallthrough (should not be reached)
        Vec::new()
    }
}

impl From<InitArgs> for RunConfig {
    fn from(args: InitArgs) -> Self {
        // If tests_dir is default and root is not, use root for tests_dir
        let tests_dir = if args.tests_dir == PathBuf::from(".") && args.root != PathBuf::from(".") {
            &args.root
        } else {
            &args.tests_dir
        };

        Self {
            // Tests before root so we can resolve the reference to root
            tests_dir_name: tests_dir.to_path_buf(),
            root: args.root,
            num_points: args.default_points,
            style_check: !args.no_style_check,
            commit_counts_flag: !args.no_commit_count,
            num_commit_checks: args.num_commit_checks,
            require_tests: args.require_tests,
            require_commits: args.require_commits,
            require_branches: args.require_branches,
        }
    }
}
/// Default settings to trim test case harness
impl Default for RunConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            tests_dir_name: PathBuf::from("."),
            num_points: 1,
            style_check: false,
            commit_counts_flag: false,
            num_commit_checks: None,
            require_tests: Vec::new(),
            require_commits: Vec::new(),
            require_branches: Vec::new(),
        }
    }
}