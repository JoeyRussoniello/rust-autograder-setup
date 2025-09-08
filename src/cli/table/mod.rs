use crate::utils::{ensure_exists, read_autograder_config};
use anyhow::{Context, Result};
use cli_clipboard;
use markdown_tables::as_table;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn run(root: &Path, to_clipboard: bool, write_to_readme: bool) -> Result<()> {
    let tests = read_autograder_config(root)?;

    let table = as_table(&tests);

    if to_clipboard && !write_to_readme {
        cli_clipboard::set_contents(table.clone()).expect("copy to clipbard");
        println!("Table copied to clipboard:");
    } else {
        println!("README Table:\n{}", table);
    }

    if write_to_readme {
        write_content_to_readme(root, &table)?;
        println!("Test description table appended to README.md");
    }

    Ok(())
}

fn write_content_to_readme(root: &Path, content: &str) -> Result<()> {
    let readme_path = root.join("README.md");
    ensure_exists(&readme_path)?;
    let mut readme = OpenOptions::new()
        .append(true)
        .open(&readme_path)
        .with_context(|| {
            format!(
                "Failed to open {} in append mode",
                readme_path.to_string_lossy()
            )
        })?;

    // Prepend a header to separate from existing content
    let header = "\n## Autograder Test Cases\n\n";
    readme
        .write_all(header.as_bytes())
        .expect("Failed to write header to README.md");
    readme
        .write_all(content.as_bytes())
        .expect("Failed to write to README.md");

    Ok(())
}
