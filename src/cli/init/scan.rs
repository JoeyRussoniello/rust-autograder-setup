use anyhow::{Context, Result};

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use syn::Path as SynPath;
use syn::Token;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, File, Item, ItemFn, Lit, Meta, visit::Visit};

use crate::types::AutoTest;
use crate::utils::{RustFile, to_rel_unix_path};

//TODO: Replace Files with its own type
pub fn find_all_tests(files: &[RustFile]) -> Result<Vec<TestWithManifest>> {
    let mut tests: Vec<TestWithManifest> = Vec::new();

    for file in files.iter() {
        let src = file.get_path_string()?;

        let file_tests = extract_tests(&src)
            .with_context(|| format!("Failed to parse {}", file.path.to_string_lossy()))?;

        let file_tests_with_manifest: Vec<TestWithManifest> = file_tests
            .into_iter()
            .map(|t| TestWithManifest {
                test: t,
                manifest_path: file.manifest_path.clone(),
            })
            .collect();

        tests.extend(file_tests_with_manifest);
    }

    Ok(tests)
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

// * Keep test manifest logic outside of test AST visiter
#[derive(Clone)]
pub struct TestWithManifest {
    pub test: Test,
    pub manifest_path: Option<PathBuf>,
}
impl TestWithManifest {
    pub fn get_distinct_manifest_paths(tests: &[Self], root: &Path) -> HashSet<String> {
        let fallback = PathBuf::from("Cargo.toml");
        tests
            .iter()
            .map(|t| {
                let manifest = t.manifest_path.as_ref().unwrap_or(&fallback);
                to_rel_unix_path(root, manifest)
            })
            .collect()
    }

    fn get_manifest_path_string(&self, root: &Path) -> Option<String> {
        self.manifest_path
            .as_ref()
            .map(|m| to_rel_unix_path(root, m)) // unix-style, relative to repo root
    }

    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    /// Create an autotest from the TestWithManifest - consumes the item
    pub fn to_autotest(self, root: &Path, num_points: u32) -> AutoTest {
        // * Don't manifest path to ./Cargo.toml for brevity and easier to read jsons/YAMLs
        let mut manifest_path = self.get_manifest_path_string(root);
        if let Some(p) = &manifest_path {
            if p == "Cargo.toml" {
                manifest_path = None;
            }
        }

        AutoTest {
            name: self.test.name,
            timeout: 10,
            points: num_points,
            docstring: self.test.docstring,
            min_commits: None,
            min_tests: None,
            manifest_path,
        }
    }
}

#[derive(Clone)]
pub struct Test {
    pub name: String,
    pub docstring: String,
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
