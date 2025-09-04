pub mod textparsers;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use regex::Regex;
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Command{
    init,
    build
}

#[derive(Parser, Debug)]
#[command(name = "auto-setup", version, about = "Autograder helper")]
struct Cli {
    /// Root of the Rust project (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    root: PathBuf,

    #[command(subcommand)]
    command: Command,
}
