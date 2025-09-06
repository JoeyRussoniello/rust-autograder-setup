use std::path::PathBuf;

use anyhow::Result;

use clap::{Args, Parser, Subcommand};

pub mod build;
pub mod init;

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
}

#[derive(Args, Debug, Default)]
pub struct BuildArgs {
    // Intentionally empty: build has no flags.
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init(a) => init::run(&a.root, a.default_points, !a.no_style_check),
        // Build has no args; default to current dir root like init would.
        Command::Build(_) => build::run(&PathBuf::from(".")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Parser, CommandFactory};

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
