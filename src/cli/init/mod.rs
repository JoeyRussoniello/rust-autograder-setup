use anyhow::{Context, Result};
use std::{fs, io::Write, path::Path};

use syn::Path as SynPath;
use syn::Token;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, File, Item, ItemFn, Lit, Meta, visit::Visit};

use crate::types::AutoTest;
use crate::utils::{collect_rs_files, ensure_exists};

#[cfg(test)]
mod tests;

pub fn run(
    root: &Path,
    tests_dir_name: &Path,
    num_points: u32,
    style_check: bool,
    commit_counts: bool,
    num_commit_checks: u32,
) -> Result<()> {
    let tests_dir = root.join(tests_dir_name);
    ensure_exists(&tests_dir)?;

    let files = collect_rs_files(&tests_dir)
        .with_context(|| format!("While scanning {}", tests_dir.to_string_lossy()))?;
    if files.is_empty() {
        anyhow::bail!("No `.rs` files found under {}", tests_dir.to_string_lossy());
    }

    let mut tests: Vec<Test> = Vec::new();
    for file in files {
        let src = fs::read_to_string(&file)
            .with_context(|| format!("Failed to read {}", file.to_string_lossy()))?;

        let file_tests = extract_tests(&src)
            .with_context(|| format!("Failed to parse {}", file.to_string_lossy()))?;

        file_tests.iter().for_each(|t| tests.push(t.clone()));
    }

    if tests.is_empty() {
        anyhow::bail!("Found no test functions (looked for #[test]/#[...::test])");
    }

    let out_dir = root.join(".autograder");
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("Failed to create {}", out_dir.to_string_lossy()))?;
    let out_path = out_dir.join("autograder.json");

    let mut items: Vec<AutoTest> = tests
        .into_iter()
        .map(|test| AutoTest {
            name: test.name,
            timeout: 10,
            points: num_points,
            docstring: test.docstring,
            min_commits: None,
        })
        .collect();
    if style_check {
        items.push(AutoTest {
            name: "CLIPPY_STYLE_CHECK".to_string(),
            timeout: 10,
            points: num_points,
            docstring: "cargo clippy style check".to_string(),
            min_commits: None,
        });
    }

    if commit_counts {
        for i in 1..=num_commit_checks {
            items.push(AutoTest {
                name: format!("COMMIT_COUNT_{}", i),
                timeout: 10,
                points: num_points,
                // ## Left because table command will replace dynamically
                docstring: "Ensures at least ## commits.".to_string(),
                min_commits: Some(i),
            });
        }
    }

    let json = serde_json::to_string_pretty(&items)?;
    let mut f = fs::File::create(&out_path)
        .with_context(|| format!("Failed to create {}", out_path.to_string_lossy()))?;
    f.write_all(json.as_bytes())?;

    println!("Wrote {}", out_path.to_string_lossy());
    Ok(())
}

///  Extractor using `syn` AST parsing and visiting:
/// 1) Parse the file into an AST
/// 2) Visit all inline modules and free functions
/// 3) A function is a test if it has an attribute:
/// - whose path's last segment is `test` (e.g., `#[test]`, `#[tokio::test]`)
/// - OR a `cfg_attr(...)` where any *applied* attribute ends with `test`
/// - we skip the first cfg predicate arg and inspect the rest
pub fn extract_tests(src: &str) -> Result<Vec<Test>> {
    let file: File =
        syn::parse_file(src).map_err(|e| anyhow::anyhow!("failed to parse Rust source: {}", e))?;

    let mut finder = TestFinder::default();
    finder.visit_file(&file);
    Ok(finder.tests)
}

#[derive(Clone)]
pub struct Test {
    name: String,
    docstring: String,
}

#[derive(Default)]
struct TestFinder {
    tests: Vec<Test>,
}

impl<'ast> Visit<'ast> for TestFinder {
    fn visit_item(&mut self, i: &'ast Item) {
        match i {
            Item::Fn(f) => self.visit_item_fn(f),
            Item::Mod(m) => {
                // Recurse into inline modules (mod m { ... })
                if let Some((_, items)) = &m.content {
                    for it in items {
                        self.visit_item(it);
                    }
                }
                // Skip out-of-line modules (mod m;), since we don't have their files here.
            }
            _ => {}
        }
    }

    fn visit_item_fn(&mut self, f: &'ast ItemFn) {
        if has_test_attr(&f.attrs) {
            let name = f.sig.ident.to_string();
            let docstring = collect_docstring(&f.attrs);
            self.tests.push(Test { name, docstring });
        }
        // No need to recurse into fn bodies for this task
    }
}

/// Returns true if any attribute marks this function as a test.
fn has_test_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(attr_is_test)
}

fn attr_is_test(attr: &Attribute) -> bool {
    let path = attr.path();

    // #[test], #[tokio::test], #[foo::bar::test]
    if path_ends_with(path, "test") {
        return true;
    }

    // #[cfg_attr(pred, test)] or #[cfg_attr(pred, tokio::test)]
    if path.is_ident("cfg_attr") {
        // Parse inner meta list: (pred, attr1, attr2, ...)
        // NOTE: use a parser function, not parse2::<Punctuated<..., Comma>>(...)
        if let Ok(args) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
            let mut iter = args.into_iter();
            let _ = iter.next(); // skip cfg predicate
            for meta in iter {
                if meta_ends_with_test(&meta) {
                    return true;
                }
            }
        }
    }

    false
}

fn path_ends_with(path: &SynPath, ident: &str) -> bool {
    path.segments
        .last()
        .map(|s| s.ident == ident)
        .unwrap_or(false)
}

fn meta_ends_with_test(meta: &Meta) -> bool {
    match meta {
        Meta::Path(p) => path_ends_with(p, "test"),
        Meta::List(ml) => {
            // If the list path itself ends with `test` (e.g., tokio::test), that’s a match.
            if path_ends_with(&ml.path, "test") {
                return true;
            }
            // Otherwise, try parsing the tokens inside the list as more Meta items:
            if let Ok(nested) =
                Punctuated::<Meta, Token![,]>::parse_terminated.parse2(ml.tokens.clone())
            {
                nested.into_iter().any(|m| meta_ends_with_test(&m))
            } else {
                false
            }
        }
        Meta::NameValue(nv) => path_ends_with(&nv.path, "test"),
    }
}

fn collect_docstring(attrs: &[Attribute]) -> String {
    let mut buf = String::new();

    for attr in attrs {
        // Only care about #[doc = "..."]
        let Meta::NameValue(nv) = &attr.meta else {
            continue;
        };
        if !nv.path.is_ident("doc") {
            continue;
        }

        let Expr::Lit(expr_lit) = &nv.value else {
            continue;
        };
        let Lit::Str(s) = &expr_lit.lit else { continue };

        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(&s.value());
    }

    buf.trim().to_string()
}
