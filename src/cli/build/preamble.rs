use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TriggerKind {
    RepositoryDispatch, // always-on implicit baseline
    WorkflowDispatch,
    Push,
    PullRequest,
}

impl TriggerKind {
    pub fn yaml_key(&self) -> &'static str {
        match self {
            TriggerKind::RepositoryDispatch => "repository_dispatch",
            TriggerKind::WorkflowDispatch   => "workflow_dispatch",
            TriggerKind::Push               => "push",
            TriggerKind::PullRequest        => "pull_request",
        }
    }
}

impl Display for TriggerKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.yaml_key())
    }
}

/// Per-trigger filters (only emit when Some/NonEmpty)
#[derive(Clone, Debug, Default)]
pub struct TriggerOptions {
    pub branches: Option<Vec<String>>,      // push / pull_request
    pub paths: Option<Vec<String>>,         // push / pull_request
    pub paths_ignore: Option<Vec<String>>,  // push / pull_request
    pub pr_types: Option<Vec<String>>,      // pull_request only
}

impl TriggerOptions {
    fn is_empty(&self) -> bool {
        self.branches.as_ref().map_or(true, |v| v.is_empty())
            && self.paths.as_ref().map_or(true, |v| v.is_empty())
            && self.paths_ignore.as_ref().map_or(true, |v| v.is_empty())
            && self.pr_types.as_ref().map_or(true, |v| v.is_empty())
    }
}

use crate::cli::helpers::build::{Preset, Trigger, resolve_triggers};

#[derive(Clone, Debug, Default)]
pub struct OnConfig {
    /// Will be used eventually
    pub preset: Option<Preset>,
    /// Map trigger → options (filters). Use BTreeMap for deterministic order.
    pub triggers: BTreeMap<TriggerKind, TriggerOptions>,
}

impl OnConfig {
    /// Build from CLI inputs; always includes repository_dispatch.
    pub fn from_args(preset: Option<Preset>, explicit: &[Trigger]) -> Self {
        use TriggerKind::*;
        let chosen: BTreeSet<TriggerKind> = resolve_triggers(preset, explicit)
            .into_iter()
            .map(|t| match t {
                Trigger::WorkflowDispatch => WorkflowDispatch,
                Trigger::Push => Push,
                Trigger::PullRequest => PullRequest,
            })
            .collect();

        let mut triggers = BTreeMap::<TriggerKind, TriggerOptions>::new();

        // Always-on baseline
        triggers.insert(RepositoryDispatch, TriggerOptions::default());

        // Fill chosen triggers with empty options 
        // TODO: Expand to add options as an enum
        for k in chosen {
            triggers.entry(k).or_default();
        }

        Self { preset, triggers }
    }

    /// Convenience builder to add a trigger (e.g., when wiring future flags).
    // pub fn add_trigger(mut self, kind: TriggerKind, opts: TriggerOptions) -> Self {
    //     self.triggers
    //         .entry(kind)
    //         .and_modify(|o| *o = opts.clone())
    //         .or_insert(opts);
    //     self
    // }

    /// Renders the `on:` block as lines (with indentation + filters).
    pub fn to_yaml_lines(&self) -> Vec<String> {
        let mut lines = vec!["on:".into()];

        for (kind, opts) in &self.triggers {
            let key = kind.yaml_key().to_string();
            if opts.is_empty() {
                // plain key:
                lines.push(format!("  {}:", key));
                continue;
            }

            // key with nested filters:
            lines.push(format!("  {}:", key));

            // Only emit filters that exist and are non-empty
            if let Some(v) = &opts.branches {
                if !v.is_empty() {
                    lines.push("    branches:".into());
                    for s in v {
                        lines.push(format!("      - {}", s));
                    }
                }
            }
            if let Some(v) = &opts.paths {
                if !v.is_empty() {
                    lines.push("    paths:".into());
                    for s in v {
                        lines.push(format!("      - {}", s));
                    }
                }
            }
            if let Some(v) = &opts.paths_ignore {
                if !v.is_empty() {
                    lines.push("    paths-ignore:".into());
                    for s in v {
                        lines.push(format!("      - {}", s));
                    }
                }
            }
            // Only meaningful for pull_request
            if matches!(kind, TriggerKind::PullRequest) {
                if let Some(v) = &opts.pr_types {
                    if !v.is_empty() {
                        lines.push("    types:".into());
                        for s in v {
                            lines.push(format!("      - {}", s));
                        }
                    }
                }
            }
        }

        lines
    }
}

