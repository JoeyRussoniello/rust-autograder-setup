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

#[test]
fn parse_init_require_tests_default_is_zero() {
    let cli = Cli::try_parse_from(["autograder-setup", "init"]).expect("parse ok");
    match cli.command {
        Command::Init(a) => {
            assert_eq!(a.require_tests, 0, "default require-tests should be 0");
        }
        _ => panic!("expected init"),
    }
}

#[test]
fn parse_init_require_tests_with_no_value_is_one() {
    // Requires: #[arg(default_missing_value = "1", num_args(0..=1))]
    let cli =
        Cli::try_parse_from(["autograder-setup", "init", "--require-tests"]).expect("parse ok");
    match cli.command {
        Command::Init(a) => {
            assert_eq!(
                a.require_tests, 1,
                "--require-tests (no value) should default to 1"
            );
        }
        _ => panic!("expected init"),
    }
}

#[test]
fn parse_init_require_tests_with_explicit_value() {
    let cli = Cli::try_parse_from(["autograder-setup", "init", "--require-tests", "5"])
        .expect("parse ok");

    match cli.command {
        Command::Init(a) => {
            assert_eq!(a.require_tests, 5);
        }
        _ => panic!("expected init"),
    }
}

#[test]
fn parse_init_all_related_flags_together() {
    let cli = Cli::try_parse_from([
        "autograder-setup",
        "init",
        "--root",
        "proj",
        "--tests-dir",
        "t",
        "--default-points",
        "3",
        "--no-style-check",
        "--no-commit-count",
        "--num-commit-checks",
        "7",
        "--require-tests",
        "2",
    ])
    .expect("parse ok");

    match cli.command {
        Command::Init(a) => {
            assert_eq!(a.root, PathBuf::from("proj"));
            assert_eq!(a.tests_dir, PathBuf::from("t"));
            assert_eq!(a.default_points, 3);
            assert!(a.no_style_check);
            assert!(a.no_commit_count);
            assert_eq!(a.num_commit_checks, Some(7));
            assert_eq!(a.require_tests, 2);
        }
        _ => panic!("expected init"),
    }
}

#[test]
fn parse_table_flags() {
    // Sanity check other subcommand flags
    let cli = Cli::try_parse_from([
        "autograder-setup",
        "table",
        "--root",
        "proj",
        "--no-clipboard",
        "--to-readme",
    ])
    .expect("parse ok");

    match cli.command {
        Command::Table(a) => {
            assert_eq!(a.root, PathBuf::from("proj"));
            assert!(a.no_clipboard);
            assert!(a.to_readme);
        }
        _ => panic!("expected table"),
    }
}
