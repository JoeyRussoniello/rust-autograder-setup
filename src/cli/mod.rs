use std::path::PathBuf;

use anyhow::Result;

use clap::{Args, Parser, Subcommand};

pub mod build;
pub mod init;
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
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Root of the Rust project (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    pub root: PathBuf,

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

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init(a) => init::run(
            &a.root,
            a.default_points,
            !a.no_style_check,
            !a.no_commit_count,
            a.num_commit_checks,
        ),
        // Build has no args; default to current dir root like init would.
        Command::Build(a) => build::run(&a.root),
        Command::Table(a) => table::run(&a.root, !a.no_clipboard, a.to_readme),
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
