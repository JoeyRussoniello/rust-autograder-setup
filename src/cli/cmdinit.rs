pub fn cmd_init(root: &Path) -> Result<()> {
    let tests_dir = root.join("tests");
    if !tests_dir.exists() {
        anyhow::bail!(
            "No `tests/` directory found at {}",
            tests_dir.to_string_lossy()
        );
    }

    let files = collect_rs_files(&tests_dir)?;
    if files.is_empty() {
        anyhow::bail!("No `.rs` files found under {}", tests_dir.to_string_lossy());
    }

    let mut names: BTreeSet<String> = BTreeSet::new();

    for file in files {
        let src = fs::read_to_string(&file)
            .with_context(|| format!("Failed to read {}", file.to_string_lossy()))?;
        for n in extract_test_names(&src) {
            names.insert(n);
        }
    }

    if names.is_empty() {
        anyhow::bail!("Found no test functions (looked for #[test]/#[...::test])");
    }

    let out_dir = root.join("tests");
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("Failed to create {}", out_dir.to_string_lossy()))?;
    let out_path = out_dir.join("autograder.json");

    let items: Vec<AutoTest> = names
        .into_iter()
        .map(|name| AutoTest {
            name,
            timeout: 10,
            points: 0,
        })
        .collect();

    let json = serde_json::to_string_pretty(&items)?;
    let mut f = fs::File::create(&out_path)
        .with_context(|| format!("Failed to create {}", out_path.to_string_lossy()))?;
    f.write_all(json.as_bytes())?;

    println!("Wrote {}", out_path.to_string_lossy());
    Ok(())
}

fn collect_rs_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    recurse(dir, &mut out)?;
    Ok(out)
}

fn recurse(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        let md = entry.metadata()?;
        if md.is_dir() {
            recurse(&p, out)?;
        } else if md.is_file() && p.extension().map(|e| e == "rs").unwrap_or(false) {
            out.push(p);
        }
    }
    Ok(())
}