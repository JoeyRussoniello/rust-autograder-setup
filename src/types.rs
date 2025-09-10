use crate::utils::replace_commit_count_docstring;
use markdown_tables::MarkdownTableRow;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoTest {
    pub name: String,
    pub docstring: String,
    pub timeout: u64,
    pub points: u32,

    // Only used when name maps to COMMIT_COUNT
    #[serde(default)]
    pub min_commits: Option<u32>,

    #[serde(default)]
    pub manifest_path: Option<String>,
}
impl MarkdownTableRow for AutoTest {
    fn column_names() -> Vec<&'static str> {
        vec!["Name", "Points", "Description"]
    }

    fn column_values(&self) -> Vec<String> {
        let doc = if let Some(min_commits) = self.min_commits {
            replace_commit_count_docstring(self.docstring.clone(), min_commits)
        } else {
            self.docstring.clone()
        };

        vec![format!("`{}`", self.name), self.points.to_string(), doc]
    }
}

//-----------AutograderCommandTypes------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepCmd {
    CargoTest {
        function_name: String,
        manifest_path: Option<String>,
    },
    ClippyCheck,
    CommitCount {
        min: u32,
    },
}

impl StepCmd {
    pub fn command(&self) -> String {
        match self {
            StepCmd::CargoTest {
                function_name,
                manifest_path,
            } => match manifest_path {
                Some(p) if !p.is_empty() => {
                    format!(
                        "cargo test {} --manifest-path {} -- --exact",
                        function_name.trim(),
                        p
                    )
                }
                _ => format!("cargo test {} -- --exact", function_name.trim()),
            },
            StepCmd::ClippyCheck => "cargo clippy -- -D warnings".to_string(),
            StepCmd::CommitCount { .. } => {
                // ! Builder will ovewrite with the path to the shell script on disk.
                String::new()
            }
        }
    }
}
