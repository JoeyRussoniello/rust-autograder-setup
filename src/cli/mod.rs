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

    match cli.command {
        Command::Init => init::run(&cli.root),
        Command::Build => build::run(&cli.root),
    }
}
