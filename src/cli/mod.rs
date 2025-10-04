use std::path::PathBuf;

use anyhow::Result;

use clap::{Args, Parser, Subcommand};
use helpers::init::RunConfig;

pub mod helpers;
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

    /// DEPRECATED: --require-commits is no longer default. Kept for backward compatibility.
    /// Hidden from short help, visible under a "DEPRECATED" heading in --help --help (optional)
    #[arg(
        long = "no-commit-count",
        hide_short_help = true,
        help_heading = "DEPRECATED",
        long_help = "DEPRECATED: --require-commits is no longer default behavior. \n\
                     This flag is not necessary."
    )]
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

    /// Require specific branch tresholds (e.g --require-branches 2 4 6)
    #[arg(long = "require-branches", value_delimiter = ' ', num_args(1..), default_values_t = Vec::<u32>::new())]
    pub require_branches: Vec<u32>,

    /// Require specific student-written test thresholds (e.g --require-tests 2 4 6)
    #[arg(long = "require-tests", value_delimiter = ' ', num_args(1..), default_values_t = Vec::<u32>::new())]
    pub require_tests: Vec<u32>,
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
