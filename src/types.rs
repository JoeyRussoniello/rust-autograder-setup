use crate::utils::get_commit_count_file_name_from_str;
use crate::utils::replace_double_hashtag;
use markdown_tables::MarkdownTableRow;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoTest {
    pub name: String,
    pub docstring: String,
    pub timeout: u64,
    pub points: u32,

    // Only used when name maps to COMMIT_COUNT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_commits: Option<u32>,

    // Only used when name maps to TEST_COUNT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_tests: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_path: Option<String>,
}
impl MarkdownTableRow for AutoTest {
    fn column_names() -> Vec<&'static str> {
        vec!["Name", "Points", "Description"]
    }

    fn column_values(&self) -> Vec<String> {
        let doc = if let Some(min_commits) = self.min_commits {
            replace_double_hashtag(self.docstring.clone(), min_commits)
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
    ClippyCheck {
        manifest_path: Option<String>,
    },
    CommitCount {
        name: String,
        min: u32,
    },
    TestCount {
        manifest_path: Option<String>,
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
                Some(p) if !p.is_empty() && p != "Cargo.toml" => {
                    format!("cargo test {} --manifest-path {}", function_name.trim(), p)
                }
                _ => format!("cargo test {}", function_name.trim()),
            },
            StepCmd::ClippyCheck { manifest_path } => match manifest_path {
                Some(p) if !p.is_empty() && p != "." => {
                    format!("cargo clippy --manifest-path {} -- -D warnings", p)
                }
                _ => "cargo clippy -- -D warnings".to_string(),
            },
            StepCmd::CommitCount { name, .. } => {
                format!(
                    "bash ./.autograder/{}",
                    get_commit_count_file_name_from_str(name)
                )
            }
            // Populate a shell script for a specific manifest path, or leave blank
            StepCmd::TestCount { min, manifest_path } => match manifest_path {
                Some(p) if !p.is_empty() && p != "Cargo.toml" => {
                    format!(
                        r#"cargo test --manifest-path {} -- --list | tail -1 | awk '{{print $1}}' | awk '{{if ($1 < {}+##) {{print "Too few tests ("$1-##") expected {}"; exit 1}}}}'"#,
                        p, min, min
                    )
                }
                _ => format!(
                    r#"cargo test -- --list | tail -1 | awk '{{print $1}}' | awk '{{if ($1 < {}+##) {{print "Too few tests ("$1-##") expected {}"; exit 1}}}}'"#,
                    min, min
                ),
            },
        }
    }
}
