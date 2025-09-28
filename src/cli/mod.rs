use std::path::PathBuf;

use anyhow::Result;

use clap::{Args, Parser, Subcommand};

pub mod build;
pub mod init;
pub mod reset;
pub mod table;

#[derive(Parser, Debug)]
#[command(
    name = "autograder-setup",
    version,
    about = "Autograder helper",
    subcommand_required = true,
    arg_required_else_help = true,
    next_line_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Scan tests and create tests/autograder.json
    Init(InitArgs),

    /// Build CI YAML from tests/autograder.json
    Build(BuildArgs),

    /// Get a table of test names, docstrings, and points for assignment READMEs
    Table(TableArgs),

    /// Delete all files created by autograder-setup
    Reset(ResetArgs),
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Root of the Rust project (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    pub root: PathBuf,

    /// Location of all test cases (defaults to <root>)
    #[arg(short, long, default_value = ".")]
    pub tests_dir: PathBuf,

    /// Default number of points per test
    #[arg(long = "default-points", default_value_t = 1)]
    pub default_points: u32,

    /// Disable the Clippy style check (enabled by default)
    #[arg(long = "no-style-check")]
    pub no_style_check: bool,

    /// Disable Commit Counting (enabled by default)
    #[arg(long = "no-commit-count")]
    pub no_commit_count: bool,

    /// DEPRECATED: use --require-commits instead. Kept for backward compatibility.
    /// Hidden from short help, visible under a "DEPRECATED" heading in --help --help (optional).
    #[arg(
        long = "num-commit-checks",
        // allow `--num-commit-checks` *or* `--num-commit-checks 4`
        num_args(0..=1),
        default_missing_value = "1",
        value_parser = clap::value_parser!(u32),
        hide_short_help = true,
        help_heading = "DEPRECATED",
        long_help = "DEPRECATED: Use --require-commits <N...> instead.\n\
                     If provided without a value, defaults to 1."
    )]
    pub num_commit_checks: Option<u32>,

    /// Require specific commit thresholds (e.g. --require-commits 5 10 15 20)
    #[arg(long = "require-commits", value_delimiter = ' ', num_args(1..), default_values_t = [1])]
    pub require_commits: Vec<u32>,

    /// Require specific branch tresholds (e.g --require-branhes 2 4 6)
    #[arg(long = "require-branches", value_delimiter = ' ', num_args(1..), default_values_t = [1])]
    pub require_branches: Vec<u32>,

    /// Require a minimum number of tests (default: 0, set to 1 if flag is passed without a value)
    #[arg(long = "require-tests", default_value_t = 0, default_missing_value = "1", num_args(0..=1))]
    pub require_tests: u32,
}

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
    pub require_tests: u32,
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
            require_tests: 0,
            require_commits: Vec::new(),
            require_branches: Vec::new(),
        }
    }
}

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Root of the Rust project (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    pub root: PathBuf,

    /// Have autograder run on push to any branch (default: grade only on "Grade All" or `repository_dispatch`)
    #[arg(long = "grade-on-push", default_value_t = false)]
    pub grade_on_push: bool,
}

#[derive(Args, Debug)]
pub struct TableArgs {
    /// Root of the Rust project (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    pub root: PathBuf,

    /// Do not copy the table to clipboard (print to terminal instead)
    #[arg(long = "no-clipboard")]
    pub no_clipboard: bool,

    /// Append the table to the end of README.md
    #[arg(long = "to-readme")]
    pub to_readme: bool,
}

#[derive(Args, Debug)]
pub struct ResetArgs {
    /// Root of the Rust project (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    pub root: PathBuf,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init(a) => {
            let cfg = RunConfig::from(a);
            init::run(&cfg)
        }
        // Build has no args; default to current dir root like init would.
        Command::Build(a) => build::run(&a.root, a.grade_on_push),
        Command::Table(a) => table::run(&a.root, !a.no_clipboard, a.to_readme),
        Command::Reset(a) => reset::run(&a.root),
    }
}

#[cfg(test)]
pub mod tests;
