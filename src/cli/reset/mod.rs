use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn run(root: &Path) -> Result<()> {
    // Remove the generated .autograder directory
    let autograder_dir = root.join(".autograder");
    if autograder_dir.exists() {
        fs::remove_dir_all(&autograder_dir)?;
        println!("Deleted {}", autograder_dir.to_string_lossy());
    }

    // Remove .github/workflows/classroom.yml if it exists
    let classroom_yml = root.join(".github").join("workflows").join("classroom.yml");
    if classroom_yml.exists() {
        fs::remove_file(&classroom_yml)?;
        println!("Deleted {}", classroom_yml.to_string_lossy());
    }

    Ok(())
}

#[cfg(test)]
pub mod tests;
