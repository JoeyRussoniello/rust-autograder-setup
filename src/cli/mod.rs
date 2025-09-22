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

    /// Number of commit count checks (default: 1).
    #[arg(long = "num-commit-checks", default_value_t = 1)]
    pub num_commit_checks: u32,

    /// Require a minimum number of tests (default: 0, set to 1 if flag is passed without a value)
    #[arg(long = "require-tests", default_value_t = 0, num_args(0..=1))]
    pub require_tests: u32,
}

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Root of the Rust project (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    pub root: PathBuf,
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
            let mut tests_dir = &a.tests_dir;

            // If tests_dir is default and root is not, use root for tests_dir
            if a.tests_dir == PathBuf::from(".") && a.root != PathBuf::from(".") {
                tests_dir = &a.root;
            }
            init::run(
                &a.root,
                tests_dir,
                a.default_points,
                !a.no_style_check,
                !a.no_commit_count,
                a.num_commit_checks,
                a.require_tests,
            )
        }
        // Build has no args; default to current dir root like init would.
        Command::Build(a) => build::run(&a.root),
        Command::Table(a) => table::run(&a.root, !a.no_clipboard, a.to_readme),
        Command::Reset(a) => reset::run(&a.root),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};

    #[test]
    fn verify_cli_schema() {
        // Catches invalid clap configuration at test time.
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_defaults_init() {
        // autograder-setup init  (all defaults)
        let cli = Cli::try_parse_from(["autograder-setup", "init"]).expect("parse ok");
        match cli.command {
            Command::Init(a) => {
                assert_eq!(a.root, PathBuf::from("."));
                assert_eq!(a.default_points, 1);
                assert!(!a.no_style_check);
            }
            _ => panic!("expected init"),
        }
    }

    #[test]
    fn parse_all_flags_init() {
        // autograder-setup init --root proj --default-points 5 --no-style-check
        let cli = Cli::try_parse_from([
            "autograder-setup",
            "init",
            "--root",
            "proj",
            "--default-points",
            "5",
            "--no-style-check",
        ])
        .expect("parse ok");

        match cli.command {
            Command::Init(a) => {
                assert_eq!(a.root, PathBuf::from("proj"));
                assert_eq!(a.default_points, 5);
                assert!(a.no_style_check);
            }
            _ => panic!("expected init"),
        }
    }

    #[test]
    fn parse_build_no_args() {
        // autograder-setup build
        let cli = Cli::try_parse_from(["autograder-setup", "build"]).expect("parse ok");
        match cli.command {
            Command::Build(_) => {} // ok
            _ => panic!("expected build"),
        }
    }

    #[test]
    fn parse_requires_subcommand() {
        // Missing subcommand should print help/error
        let err = Cli::try_parse_from(["autograder-setup"]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Usage") || msg.contains("USAGE"));
        assert!(msg.contains("init") && msg.contains("build"));
    }
}
