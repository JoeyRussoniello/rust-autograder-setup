use clap::ValueEnum;
use std::collections::BTreeSet;

#[derive(Copy, Clone, Debug, ValueEnum, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Trigger {
    /// Manual run via the Actions UI
    WorkflowDispatch,
    /// Run on git push
    Push,
    /// Run on PR events
    PullRequest,
    // NOTE: repository_dispatch is always included by the CLI, so it isn't exposed here.
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
    pub fn as_trigger(&self) -> BTreeSet<Trigger> {
        use Preset::*;
        use Trigger::*;

        match self {
            Instructor => BTreeSet::from([WorkflowDispatch]),
            StudentPush => BTreeSet::from([Push]),
            StudentPr => BTreeSet::from([PullRequest]),
        }
    }
}

pub fn resolve_triggers(preset: Option<Preset>, explicit: &[Trigger]) -> BTreeSet<Trigger> {
    let mut set: BTreeSet<Trigger> = BTreeSet::new();
    if let Some(p) = preset {
        set.extend(p.as_trigger());
    }
    set.extend(explicit.iter().copied());
    set
}
