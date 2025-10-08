use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::fs::{create_dir_all, write};
use std::path::Path;
// Import all utility script constants
use crate::cli::helpers::build::{Preset, Trigger, TriggerKind, TriggerOption, resolve_triggers};
use crate::utils::scripts::*;

/// Any trigger with options means we need the expanded `on:` block.
fn needs_expanded_on(triggers: &BTreeSet<Trigger>) -> bool {
    triggers.iter().any(|t| t.options_to_yaml_lines().is_some())
}

/// `on: [repository_dispatch, push, pull_request]`
fn render_compact_on(triggers: &BTreeSet<Trigger>) -> String {
    // NOTE: This assumes each trigger would render to a single token in compact form.
    let mut tokens = Vec::with_capacity(triggers.len() + 1);
    tokens.push("repository_dispatch".to_string());
    for t in triggers {
        tokens.push(t.kind.get_yaml_str());
    }
    format!("on: [{}]", tokens.join(", "))
}

/// Expanded form:
/// on:
///   repository_dispatch:
///   push:
///     branches: [main]
///     paths: [src/**]
fn render_expanded_on(triggers: &std::collections::BTreeSet<Trigger>) -> Vec<String> {
    let mut lines = vec!["on:".into(), "  repository_dispatch:".into()];

    for t in triggers {
        match t.options_to_yaml_lines() {
            None => {
                // No options: emit just the mapping key
                let key = t.kind.get_yaml_str();
                lines.push(format!("  {}:", key));
            }
            Some(opts) => {
                // With options: first the key line, then the options already formatted
                lines.push(format!("  {}:", t.kind.get_yaml_str()));
                for o in opts {
                    lines.push(format!("    {}", o)); // options are already "key: [..]" lines
                }
            }
        }
    }
    lines
}

/// Decide compact vs expanded and append to `out`.
fn render_on_block(triggers: &std::collections::BTreeSet<Trigger>, out: &mut Vec<String>) {
    if needs_expanded_on(triggers) {
        out.extend(render_expanded_on(triggers));
    } else {
        out.push(render_compact_on(triggers));
    }
}

const FIXED_PREAMBLE_TAIL: &[&str] = &[
    "",
    "permissions:",
    "  checks: write",
    "  actions: read",
    "  contents: read",
    "",
    "jobs:",
    "  run-autograding-tests:",
    "    runs-on: ubuntu-latest",
    "    if: github.actor != 'github-classroom[bot]'",
    "    steps:",
    "      - name: Checkout code",
    "        uses: actions/checkout@v4",
    "        with:",
    "          # Checkout with fetch depth 0 to get a full git history for commit counting",
    "          fetch-depth: 0",
    "",
    "      - name: Install Rust toolchain",
    "        uses: dtolnay/rust-toolchain@stable",
    "        with:",
    "          components: clippy,rustfmt",
    "",
];

/// Full version: accepts shared `TriggerOption`s for all triggers
pub fn get_yaml_preamble_with_options(
    preset: Option<Preset>,
    explicit_triggers: &[TriggerKind],
    trigger_options: &[TriggerOption],
) -> String {
    let triggers = resolve_triggers(preset, explicit_triggers, trigger_options);

    let mut lines = vec!["name: Autograding Tests".into()];
    render_on_block(&triggers, &mut lines);
    lines.extend(FIXED_PREAMBLE_TAIL.iter().map(|s| s.to_string()));
    lines.join("\n")
}

/// Shared function to write any script to the .autograder directory
fn write_script(root: &Path, script_name: &str, contents: &str) -> Result<()> {
    let script_path = root.join(".autograder").join(script_name);

    // Bail early if the script already exists
    if script_path.exists() {
        return Ok(());
    }

    create_dir_all(script_path.parent().unwrap()).with_context(|| {
        format!(
            "Failed to create directory for {}",
            script_path.to_string_lossy()
        )
    })?;
    write(&script_path, contents)
        .with_context(|| format!("Failed to write to {}", script_path.to_string_lossy()))?;
    Ok(())
}

/// Write a single parameterized shell script to count commits in a git repository.
pub fn write_commit_count_shell(root: &Path) -> Result<()> {
    write_script(
        root,
        SCRIPT_NAMES.commit_count,
        COMMIT_COUNT_SCRIPT_CONTENTS,
    )?;
    Ok(())
}

/// Write a single parameterized shell script to count branches in a git repository.
pub fn write_branch_count_shell(root: &Path) -> Result<()> {
    write_script(
        root,
        SCRIPT_NAMES.branch_count,
        BRANCH_COUNT_SCRIPT_CONTENTS,
    )?;
    Ok(())
}
