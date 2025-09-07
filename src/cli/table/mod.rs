use crate::utils::read_autograder_config;
use anyhow::Result;
use cli_clipboard;
use markdown_tables::as_table;
use std::path::Path;

pub fn run(root: &Path, to_clipboard: bool) -> Result<()> {
    let tests = read_autograder_config(root)?;

    let table = as_table(&tests);

    if to_clipboard {
        cli_clipboard::set_contents(table.clone()).expect("copy to clipbard");
        println!("Table copied to clipboard:");
    } else {
        println!("README Table:\n{}", table);
    }
    Ok(())
}
