use std::path::Path;
use anyhow::{Context, Result};

use serde_json;
use std::fs::{File, create_dir_all};
use std::io::BufReader;

use crate::utils::ensure_exists;
use crate::types::AutoTest;

pub fn run(root: &Path) -> Result<()>{
    let autograder_config = root.join("tests").join("autograder.json");
    ensure_exists(&autograder_config)?;
    let tests = read_autograder_config(&autograder_config)?;

    if tests.is_empty() {
        anyhow::bail!("Autograder.json config not configured. Add tests using `auto-setup init`");
    }

    let workflows_dir = root.join(".github").join("workflows");
    create_dir_all(&workflows_dir)
        .with_context(|| format!("Failed to create {}", workflows_dir.to_string_lossy()))?;

    let workflow_path = workflows_dir.join("classroom.yaml");
    let workflow_content = render_workflow(&tests)?;

    write_workflow(&workflow_path, &workflow_content)?;
    println!("Wrote Configured autograder YAML to {}", workflow_path.to_string_lossy());
    return Ok(());
}

fn read_autograder_config(path: &Path) -> Result<Vec<AutoTest>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tests = serde_json::from_reader(reader)?;
    return Ok(tests);
}

fn render_workflow(tests: &[AutoTest]) -> Result<String> {
    let mut out = String::new();

    // --- PREAMBLE (fixed) ---
    out.push_str("name: Autograding Tests\n");
    out.push_str("on: [push, repository_dispatch]\n\n");
    out.push_str("permissions:\n");
    out.push_str("  checks: write\n");
    out.push_str("  actions: read\n");
    out.push_str("  contents: read\n\n");
    out.push_str("jobs:\n");
    out.push_str("  run-autograding-tests:\n");
    out.push_str("    runs-on: ubuntu-latest\n");
    out.push_str("    if: github.actor != 'github-classroom[bot]'\n");
    out.push_str("    steps:\n");
    out.push_str("      - name: Checkout code\n");
    out.push_str("        uses: actions/checkout@v4\n\n");
    out.push_str("      - name: Install Rust toolchain\n");
    out.push_str("        uses: dtolnay/rust-toolchain@stable\n");
    out.push_str("        with:\n");
    out.push_str("          components: clippy,rustfmt\n\n");

    // --- Test steps ---
    let mut ids: Vec<String> = Vec::with_capacity(tests.len());
    for t in tests {
        let name = t.name.trim();
        let id = slug_id(name);

        ids.push(id.clone());

        // Step
        out.push_str(&format!("      - name: {}\n", yaml_quote(name)));
        out.push_str(&format!("        id: {}\n", id));
        out.push_str("        uses: classroom-resources/autograding-command-grader@v1\n");
        out.push_str("        with:\n");
        out.push_str(&format!("          test-name: {}\n", yaml_quote(name)));
        out.push_str("          setup-command: \"\"\n");
        out.push_str(&format!("          command: cargo test {}\n", name));
        out.push_str(&format!("          timeout: {}\n", t.timeout));
        out.push_str(&format!("          max-score: {}\n\n", t.points));
    }

    // --- Reporter step ---
    out.push_str("      - name: Autograding Reporter\n");
    out.push_str("        uses: classroom-resources/autograding-grading-reporter@v1\n");
    out.push_str("        env:\n");
    for id in &ids {
        let env_key = format!("{}_RESULTS", id.to_uppercase());
        out.push_str(&format!(
            "          {}: \"${{{{steps.{}.outputs.result}}}}\"\n",
            env_key, id
        ));
    }
    out.push_str("        with:\n");
    out.push_str("          runners: ");
    out.push_str(&ids.join(","));
    out.push('\n');

    Ok(out)
}

fn write_workflow(path: &Path, content: &str) -> Result<()> {
    let mut f = File::create(path)
        .with_context(|| format!("Failed to create {}", path.to_string_lossy()))?;
    use std::io::Write;
    f.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write {}", path.to_string_lossy()))?;
    Ok(())
}
// Lowercase; spaces/non-alnum -> hyphens; collapse/trim hyphens.
fn slug_id(name: &str) -> String {
    let mut s = String::new();
    let mut last_dash = false;
    for ch in name.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            s.push(c);
            last_dash = false;
        } else {
            if !last_dash {
                s.push('-');
                last_dash = true;
            }
        }
    }
    // trim leading/trailing dashes
    while s.starts_with('-') { s.remove(0); }
    while s.ends_with('-') { s.pop(); }
    // collapse multiple dashes already handled by last_dash flag
    s
}

// Quote for YAML (simple: double-quote and escape double quotes)
fn yaml_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\\\""))
}