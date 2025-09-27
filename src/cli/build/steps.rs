// Minimal, serde-free emitters for GitHub Classroom steps.
use crate::utils::{YAML_INDENT, yaml_quote};
use std::collections::BTreeMap; // stable key order in YAML env

pub struct CommandWith {
    pub test_name: String,
    pub setup_command: String,
    pub command: String,
    pub timeout: u64,
    pub max_score: u32,
}

pub struct CommandStep {
    pub name: String,
    pub id: String,
    pub uses: String, // e.g., "classroom-resources/autograding-command-grader@v1"
    pub with: CommandWith,
}

pub struct ReporterStep {
    pub name: String, // "Autograding Reporter"
    pub uses: String, // "classroom-resources/autograding-grading-reporter@v1"
    pub env: BTreeMap<String, String>,
    pub runners_csv: String, // "id-a,id-b,id-c"
}

// --------- helpers ---------
fn indent(s: &mut String, level: usize, line: impl AsRef<str>) {
    for _ in 0..level {
        s.push_str(YAML_INDENT);
    }
    s.push_str(line.as_ref());
    s.push('\n');
}

// --------- emitters ---------
impl CommandStep {
    /// Append this step as YAML list item starting at `indent_level` (e.g., 3)
    pub fn write_to(&self, buf: &mut String, indent_level: usize) {
        indent(
            buf,
            indent_level,
            format!("- name: {}", yaml_quote(&self.name)),
        );
        indent(
            buf,
            indent_level + 1,
            format!("id: {}", yaml_quote(&self.id)),
        );
        indent(
            buf,
            indent_level + 1,
            format!("uses: {}", yaml_quote(&self.uses)),
        );
        indent(buf, indent_level + 1, "with:");
        indent(
            buf,
            indent_level + 2,
            format!("test-name: {}", yaml_quote(&self.with.test_name)),
        );
        indent(
            buf,
            indent_level + 2,
            format!("setup-command: {}", yaml_quote(&self.with.setup_command)),
        );
        indent(
            buf,
            indent_level + 2,
            format!("command: {}", yaml_quote(&self.with.command)),
        );
        indent(
            buf,
            indent_level + 2,
            format!("timeout: {}", self.with.timeout),
        );
        indent(
            buf,
            indent_level + 2,
            format!("max-score: {}", self.with.max_score),
        );
    }
}

impl ReporterStep {
    pub fn write_to(&self, buf: &mut String, indent_level: usize) {
        indent(buf, indent_level, format!("- name: {}", self.name));
        indent(buf, indent_level + 1, format!("uses: {}", self.uses));
        indent(buf, indent_level + 1, "env:");
        for (k, v) in &self.env {
            // env values in reporter are already ${{steps.id.outputs.result}}
            indent(buf, indent_level + 2, format!("{k}: \"{v}\""));
        }
        indent(buf, indent_level + 1, "with:");
        indent(
            buf,
            indent_level + 2,
            format!("runners: {}", self.runners_csv),
        );
    }
}
