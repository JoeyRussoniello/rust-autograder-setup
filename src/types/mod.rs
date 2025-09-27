mod command_makers;

use crate::utils::replace_double_hashtag;
use command_makers::*;
use markdown_tables::MarkdownTableRow;
use serde::{Deserialize, Serialize};

/// Common, always present bits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMeta {
    pub name: String,
    pub description: String,
    pub points: u32,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TestKind {
    CargoTest {
        #[serde(skip_serializing_if = "Option::is_none")]
        manifest_path: Option<String>,
    },
    Clippy {
        #[serde(skip_serializing_if = "Option::is_none")]
        manifest_path: Option<String>,
    },
    CommitCount {
        min_commits: u32,
    },
    TestCount {
        min_tests: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        manifest_path: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoTest {
    pub meta: TestMeta,
    #[serde(flatten)]
    pub kind: TestKind,
}

// tiny esc to keep tables from breaking if  | or ` are used
fn esc(s: &str) -> String {
    s.replace('\\', r"\\")
        .replace('|', r"\|")
        .replace('`', r"\`")
}

/// Shared writing pattern between commit count and test count
fn mk_description(desc: &str, min: u32) -> String {
    esc(&replace_double_hashtag(desc, min))
}
impl AutoTest {
    /// Fill tokens like {min_commits}, {min_tests}, {manifest_path}, {function}
    fn resolved_description(&self) -> String {
        match &self.kind {
            TestKind::CommitCount { min_commits } => {
                mk_description(&self.meta.description, *min_commits)
            }
            TestKind::TestCount { min_tests, .. } => {
                mk_description(&self.meta.description, *min_tests)
            }
            _ => esc(&self.meta.description),
        }
    }

    fn command(&self) -> String {
        match &self.kind {
            TestKind::CargoTest { manifest_path } => {
                cargo_test_cmd(&self.meta.name, manifest_path.as_deref())
            }
            TestKind::Clippy { manifest_path } => clippy_cmd(manifest_path.as_deref()),
            TestKind::CommitCount { .. } => commit_count_cmd(&self.meta.name),
            TestKind::TestCount {
                min_tests,
                manifest_path,
            } => test_count_cmd(*min_tests, manifest_path.as_deref()),
        }
    }
}

impl MarkdownTableRow for AutoTest {
    fn column_names() -> Vec<&'static str> {
        vec!["Name", "Points", "Description"]
    }

    fn column_values(&self) -> Vec<String> {
        let doc = self.resolved_description();
        vec![
            format!("`{}`", self.meta.name),
            self.meta.points.to_string(),
            doc,
        ]
    }
}

// //-----------AutograderCommandTypes------------------
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum StepCmd {
//     CargoTest {
//         function_name: String,
//         manifest_path: Option<String>,
//     },
//     ClippyCheck {
//         manifest_path: Option<String>,
//     },
//     CommitCount {
//         name: String,
//         min: u32,
//     },
//     TestCount {
//         manifest_path: Option<String>,
//         min: u32,
//     },
//}

// impl StepCmd {
//     pub fn command(&self) -> String {
//         match self {
//             StepCmd::CargoTest {
//                 function_name,
//                 manifest_path,
//             } => match manifest_path {
//                 Some(p) if !p.is_empty() && p != "Cargo.toml" => {
//                     format!("cargo test {} --manifest-path {}", function_name.trim(), p)
//                 }
//                 _ => format!("cargo test {}", function_name.trim()),
//             },
//             StepCmd::ClippyCheck { manifest_path } => match manifest_path {
//                 Some(p) if !p.is_empty() && p != "." => {
//                     format!("cargo clippy --manifest-path {} -- -D warnings", p)
//                 }
//                 _ => "cargo clippy -- -D warnings".to_string(),
//             },
//             StepCmd::CommitCount { name, .. } => {
//                 format!(
//                     "bash ./.autograder/{}",
//                     get_commit_count_file_name_from_str(name)
//                 )
//             }
//             // Populate a shell script for a specific manifest path, or leave blank
//             StepCmd::TestCount { min, manifest_path } => match manifest_path {
//                 Some(p) if !p.is_empty() && p != "Cargo.toml" => {
//                     format!(
//                         r#"cargo test --manifest-path {} -- --list | tail -1 | awk '{{print $1}}' | awk '{{if ($1 < {}+##) {{print "Too few tests ("$1-##") expected {}"; exit 1}}}}'"#,
//                         p, min, min
//                     )
//                 }
//                 _ => format!(
//                     r#"cargo test -- --list | tail -1 | awk '{{print $1}}' | awk '{{if ($1 < {}+##) {{print "Too few tests ("$1-##") expected {}"; exit 1}}}}'"#,
//                     min, min
//                 ),
//             },
//         }
//     }
// }
