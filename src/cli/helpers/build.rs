use crate::cli::BuildArgs;
use crate::utils::yaml_quote;
use clap::ValueEnum;
use std::collections::BTreeSet;
#[derive(Copy, Clone, Debug, ValueEnum, Eq, PartialEq, Hash, PartialOrd, Ord)]
/// An enum that represents the different kinds of triggers
pub enum TriggerKind {
    /// Manual run via the Actions UI
    WorkflowDispatch,
    /// Run on git push
    Push,
    /// Run on PR events
    PullRequest,
    // NOTE: repository_dispatch is always included by the CLI, so it isn't exposed here.
}
impl TriggerKind {
    pub fn get_yaml_str(&self) -> String {
        let str = match self {
            TriggerKind::WorkflowDispatch => "workflow_dispatch",
            TriggerKind::Push => "push",
            TriggerKind::PullRequest => "pull_request",
        };
        str.to_string()
    }
}

/// A struct that represents the different configuration options for triggers
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum TriggerOption {
    Branches(Vec<String>),    // push / pull_request
    Paths(Vec<String>),       // push / pull_request
    PathsIgnore(Vec<String>), // push / pull_request
}

/// A helper function to surround necessary triggers with quotes 
fn quote_all(options: &Vec<String>) -> Vec<String>{
    options.iter().map(|o| yaml_quote(o)).collect()
}
impl TriggerOption {
    fn get_yaml_str(&self) -> Option<String> {
        let (header_str, options_vec) = match self {
            Self::Branches(v) => ("branches", v),
            // * Surround paths in quotes
            Self::Paths(v) => ("paths", &quote_all(v)),
            Self::PathsIgnore(v) => ("paths-ignore", &quote_all(v)),
        };

        // Don't accidentally return branches: [], as that is invalid yaml
        if options_vec.is_empty() {
            return None;
        }

        Some(format!("{}: [{}]", header_str, options_vec.join(", ")))
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Trigger {
    pub kind: TriggerKind,
    pub options: Vec<TriggerOption>,
}
impl Trigger {
    pub fn options_to_yaml_lines(&self) -> Option<Vec<String>> {
        let mut lines = Vec::with_capacity(self.options.len());
        for opt in &self.options {
            // If the option is PR types but this trigger isn't a pull_request, skip it.
            if let Some(s) = opt.get_yaml_str() {
                lines.push(s);
            }
        }
        if lines.is_empty() { None } else { Some(lines) }
    }
}

/// Turn CLI flags into TriggerOption(s)
pub fn collect_trigger_options(args: &BuildArgs) -> Vec<TriggerOption> {
    let mut opts = Vec::with_capacity(3);

    if !args.branches.is_empty() {
        // NOTE: clone so later mutation of args.branches won’t affect stored options
        opts.push(TriggerOption::Branches(args.branches.clone()));
    }
    if !args.paths.is_empty() {
        opts.push(TriggerOption::Paths(args.paths.clone()));
    }
    if !args.paths_ignore.is_empty() {
        opts.push(TriggerOption::PathsIgnore(args.paths_ignore.clone()));
    }

    opts
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Preset {
    /// Instructor-driven grading; manual-friendly
    Instructor,
    /// Students get feedback on push
    StudentPush,
    /// Students get feedback on PRs
    StudentPr,
}
impl Preset {
    pub fn as_trigger_kinds(&self) -> &[TriggerKind] {
        use Preset::*;
        use TriggerKind::*;

        match self {
            Instructor => &[WorkflowDispatch],
            StudentPush => &[Push],
            StudentPr => &[PullRequest],
        }
    }
}

pub fn resolve_triggers(
    preset: Option<Preset>,
    explicit: &[TriggerKind],
    trigger_options: &[TriggerOption],
) -> BTreeSet<Trigger> {
    let mut set: BTreeSet<Trigger> = BTreeSet::new();
    let options = trigger_options.to_vec();

    if let Some(p) = preset {
        let kinds = p.as_trigger_kinds();

        kinds.iter().for_each(|kind| {
            set.insert(Trigger {
                kind: *kind,
                options: options.clone(),
            });
        });
    }

    // Slightly non-optimal to store the options in each, but could eventually allow different trigger_options for different
    // triggers
    explicit.iter().for_each(|kind| {
        set.insert(Trigger {
            kind: *kind,
            options: options.clone(),
        });
    });

    set
}
