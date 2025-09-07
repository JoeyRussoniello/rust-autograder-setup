use serde::{Deserialize, Serialize};
use markdown_tables::MarkdownTableRow;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoTest {
    pub name: String,
    pub docstring: String,
    pub timeout: u64,
    pub points: u32,
}
impl MarkdownTableRow for AutoTest {
    fn column_names() -> Vec<&'static str> {
        vec!["Name", "Points", "Description"]
    }

    fn column_values(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.points.to_string(),
            self.docstring.clone(),
        ]
    }
}
