use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod build;
pub mod init;

#[derive(Parser, Debug)]
#[command(name = "auto-setup", version, about = "Autograder helper")]
pub struct Cli {
    /// Root of the Rust project (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    pub root: PathBuf,

    #[arg(
        long = "default-points",
        default_value_t = 1,
        help = "Default number of points per test"
    )]
    pub default_points: u32,

    #[arg(
        long = "no-style-check",
        help = "Disable the Clippy style check (enabled by default)"
    )]
    pub no_style_check: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Scan tests and create tests/autograder.json
    Init,
    /// (stub) Build CI YAML from tests/autograder.json
    Build,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    let clippy_style_check = !cli.no_style_check;
    match cli.command {
        Command::Init => init::run(&cli.root, cli.default_points, clippy_style_check),
        Command::Build => build::run(&cli.root),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defaults_init() {
        // auto-setup init  (all defaults)
        let cli = Cli::try_parse_from(["auto-setup", "init"]).expect("parse ok");
        assert!(matches!(cli.command, Command::Init));
        assert_eq!(cli.root, std::path::PathBuf::from("."));
        assert_eq!(cli.default_points, 1);
        assert!(!cli.no_style_check); // default is enabled, so flag is false
    }

    #[test]
    fn parse_all_flags_build() {
        // auto-setup --root proj --default-points 5 --no-style-check build
        let cli = Cli::try_parse_from([
            "auto-setup",
            "--root",
            "proj",
            "--default-points",
            "5",
            "--no-style-check",
            "build",
        ])
        .expect("parse ok");

        assert!(matches!(cli.command, Command::Build));
        assert_eq!(cli.root, std::path::PathBuf::from("proj"));
        assert_eq!(cli.default_points, 5);
        assert!(cli.no_style_check);
    }

    #[test]
    fn parse_requires_subcommand() {
        // Missing subcommand should error
        let err = Cli::try_parse_from(["auto-setup"]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Usage"));
        assert!(msg.contains("init") && msg.contains("build"));
    }
}
