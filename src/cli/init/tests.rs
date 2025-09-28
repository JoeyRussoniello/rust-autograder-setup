use super::scan::{Test, extract_tests};
use super::*;
use crate::types::{AutoTest, TestKind};
use crate::utils::read_autograder_config;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

/// ------------------- Small helpers -------------------

fn extract_test_names(src: &str) -> Vec<String> {
    extract_tests(src)
        .expect("Error parsing file")
        .into_iter()
        .map(|t| t.name)
        .collect()
}

fn sorted(mut names: Vec<String>) -> Vec<String> {
    names.sort();
    names
}

fn is_test_count(t: &AutoTest) -> bool {
    matches!(t.kind, TestKind::TestCount { .. })
}
fn is_commit_count(t: &AutoTest) -> bool {
    matches!(t.kind, TestKind::CommitCount { .. })
}
fn test_count_steps<'a>(items: &'a [AutoTest]) -> Vec<&'a AutoTest> {
    items.iter().filter(|t| is_test_count(t)).collect()
}
fn commit_count_steps<'a>(items: &'a [AutoTest]) -> Vec<&'a AutoTest> {
    items.iter().filter(|t| is_commit_count(t)).collect()
}
fn min_commits(t: &AutoTest) -> Option<u32> {
    match t.kind {
        TestKind::CommitCount { min_commits } => Some(min_commits),
        _ => None,
    }
}
fn min_tests(t: &AutoTest) -> Option<u32> {
    match t.kind {
        TestKind::TestCount { min_tests, .. } => Some(min_tests),
        _ => None,
    }
}
fn manifest_path_str(t: &AutoTest) -> Option<&str> {
    match &t.kind {
        TestKind::CargoTest { manifest_path }
        | TestKind::Clippy { manifest_path }
        | TestKind::TestCount { manifest_path, .. } => manifest_path.as_deref(),
        TestKind::CommitCount { .. } => None,
    }
}

fn by_name<'a>(tests: &'a [Test], name: &str) -> &'a Test {
    tests.iter().find(|t| t.name == name).unwrap_or_else(|| {
        panic!(
            "test `{}` not found in {:?}",
            name,
            tests.iter().map(|t| &t.name).collect::<Vec<_>>()
        )
    })
}

/// ------------------- test discovery -------------------

#[test]
fn finds_plain_test() {
    let src = r#"
        #[test]
        fn a() {}

        fn not_a_test() {}
    "#;
    assert_eq!(sorted(extract_test_names(src)), vec!["a"]);
}

#[test]
fn finds_namespaced_test_and_async_pub() {
    let src = r#"
        #[tokio::test]
        async fn b_async() {}

        #[foo::bar::test]
        pub fn c_pub() {}

        #[tokio::test(flavor = "multi_thread")]
        async fn d_with_args() {}
    "#;
    assert_eq!(
        sorted(extract_test_names(src)),
        vec!["b_async", "c_pub", "d_with_args"]
    );
}

#[test]
fn respects_modifiers_between_attr_and_fn() {
    let src = r#"
        #[test] pub    fn e() {}
        #[test]    async   fn f() {}
        #[tokio::test]   pub   async   fn g() {}
    "#;
    assert_eq!(sorted(extract_test_names(src)), vec!["e", "f", "g"]);
}

#[test]
fn finds_cfg_attr_test_simple() {
    let src = r#"
        #[cfg_attr(test, test)]
        fn h() {}

        #[cfg_attr(test, tokio::test)]
        async fn i() {}
    "#;
    assert_eq!(sorted(extract_test_names(src)), vec!["h", "i"]);
}

#[test]
fn cfg_attr_with_multiple_applied_attrs() {
    let src = r#"
        #[cfg_attr(test, my::attr(test), tokio::test(flavor="current_thread"))]
        async fn j_multi() {}
    "#;
    assert_eq!(sorted(extract_test_names(src)), vec!["j_multi"]);
}

#[test]
fn cfg_attr_non_applied_predicate_does_not_mark_test() {
    let src = r#"
        #[cfg_attr(not(test), allow(dead_code))]
        fn not_test() {}

        fn also_not_test() {}
    "#;
    assert!(extract_test_names(src).is_empty());
}

#[test]
fn recurses_inline_modules_but_not_out_of_line() {
    let src = r#"
        mod inline_mod {
            #[test]
            fn inner_ok() {}
            mod deeper {
                #[tokio::test] async fn deep_ok() {}
            }
        }

        mod helpers; // out-of-line

        #[test]
        fn top_level() {}
    "#;
    assert_eq!(
        sorted(extract_test_names(src)),
        vec!["deep_ok", "inner_ok", "top_level"]
    );
}

#[test]
fn ignores_commented_out_attrs_and_functions() {
    let src = r#"
        // #[test]
        // fn commented_line() {}

        /*
        #[test]
        fn commented_block() {}
        */

        #[test]
        fn real() {}
    "#;
    assert_eq!(sorted(extract_test_names(src)), vec!["real"]);
}

#[test]
fn multiple_attrs_on_same_fn() {
    let src = r#"
        #[allow(non_snake_case)]
        #[test]
        fn Kebab__ok() {}

        #[doc = "Some doc"]
        #[tokio::test]
        async fn with_doc() {}
    "#;
    assert_eq!(
        sorted(extract_test_names(src)),
        vec!["Kebab__ok", "with_doc"]
    );
}

#[test]
fn tricky_spacing_and_newlines() {
    let src = r#"
        #[  test  ]
        fn spaced() {}

        #[tokio
            ::
            test
        (   flavor    =   "current_thread"   )]
        async fn weird() {}
    "#;
    assert_eq!(sorted(extract_test_names(src)), vec!["spaced", "weird"]);
}

#[test]
fn cfg_attr_with_nested_list_tokens() {
    let src = r#"
        #[cfg_attr(test, my::attr(test(foo, bar)), my::attr(baz), test)]
        fn nested() {}
    "#;
    assert_eq!(sorted(extract_test_names(src)), vec!["nested"]);
}

#[test]
fn does_not_false_positive_on_non_test_attrs() {
    let src = r#"
        #[allow(dead_code)]
        fn x() {}

        #[derive(Debug)]
        fn y() {}

        #[cfg_attr(test, allow(dead_code))]
        fn z() {}
    "#;
    assert!(extract_test_names(src).is_empty());
}

#[test]
fn supports_underscore_and_digits_in_idents() {
    let src = r#"
        #[test] fn _leading_underscore() {}
        #[test] fn snake_case_123() {}
    "#;
    assert_eq!(
        sorted(extract_test_names(src)),
        vec!["_leading_underscore", "snake_case_123"]
    );
}

#[test]
fn many_in_one_file() {
    let src = r#"
        #[test] fn a() {}
        #[tokio::test] async fn b() {}
        #[cfg_attr(test, test)] fn c() {}
        #[cfg_attr(test, tokio::test(flavor="multi_thread"))] async fn d() {}
        fn not() {}
        mod m {
            #[test] fn e() {}
            mod n { #[tokio::test] async fn f() {} }
        }
    "#;
    assert_eq!(
        sorted(extract_test_names(src)),
        vec!["a", "b", "c", "d", "e", "f"]
    );
}

/// ------------------- docstring extraction -------------------

#[test]
fn single_line_doc_on_plain_test() {
    let src = r#"
        /// adds two positive numbers
        #[test]
        fn add_simple() {}
    "#;

    let tests = extract_tests(src).expect("Error parsing file");
    let t = by_name(&tests, "add_simple");
    assert_eq!(t.docstring, "adds two positive numbers");
}

#[test]
fn multi_line_doc_triple_slash() {
    let src = r#"
        /// first line
        /// second line
        /// third line
        #[test]
        fn multi() {}
    "#;

    let tests = extract_tests(src).expect("Error parsing file");
    let t = by_name(&tests, "multi");
    assert_eq!(t.docstring, "first line\n second line\n third line");
}

#[test]
fn block_doc_comment_preserved() {
    let src = r#"
        /** 
         * line one
         * line two
         * line three
         */
        #[test]
        fn blocky() {}
    "#;

    let tests = extract_tests(src).expect("Error parsing file");
    let t = &tests[0];

    assert_eq!(t.name, "blocky");
    assert!(t.docstring.contains("line one"));
    assert!(t.docstring.contains("line two"));
    assert!(t.docstring.contains("line three"));
}

#[test]
fn doc_with_namespaced_test_and_async() {
    let src = r#"
        /// runs on tokio runtime
        #[tokio::test(flavor = "current_thread")]
        async fn tok() {}
    "#;

    let tests = extract_tests(src).expect("Error parsing file");
    let t = by_name(&tests, "tok");
    assert_eq!(t.docstring, "runs on tokio runtime");
}

#[test]
fn ignores_non_doc_attributes() {
    let src = r#"
        #[allow(non_snake_case)]
        /// the doc should be captured, not allow(...)
        #[test]
        fn MixedCase() {}
    "#;

    let tests = extract_tests(src).expect("Error parsing file");
    let t = by_name(&tests, "MixedCase");
    assert_eq!(t.docstring, "the doc should be captured, not allow(...)");
}

#[test]
fn empty_doc_when_absent() {
    let src = r#"
        #[test]
        fn no_docs() {}
    "#;

    let tests = extract_tests(src).expect("Error parsing file");
    let t = by_name(&tests, "no_docs");
    assert_eq!(t.docstring, "");
}

#[test]
fn preserves_order_and_newlines_exactly() {
    let src = r#"
        /// alpha
        ///
        /// gamma
        #[test]
        fn spaced() {}
    "#;

    let tests = extract_tests(src).expect("Error parsing file");
    let t = by_name(&tests, "spaced");
    assert!(t.docstring.contains("alpha"));
    assert!(t.docstring.contains("gamma"));
    assert!(t.docstring.contains("\n\n"));
}

#[test]
fn inline_mods_collect_their_docs() {
    let src = r#"
        mod inner {
            /// inner doc
            #[test]
            fn inner_test() {}
            mod deeper {
                /// deeper doc
                #[tokio::test] async fn deep_test() {}
            }
        }

        /// top doc
        #[test]
        fn top() {}
    "#;

    let tests = extract_tests(src).expect("Error parsing file");
    let names_docs: Vec<(&str, &str)> = tests
        .iter()
        .map(|t| (t.name.as_str(), t.docstring.as_str()))
        .collect();

    assert!(names_docs.contains(&("inner_test", "inner doc")));
    assert!(names_docs.contains(&("deep_test", "deeper doc")));
    assert!(names_docs.contains(&("top", "top doc")));
}

#[test]
fn doc_attribute_form_is_supported() {
    let src = r#"
        #[doc = "first"]
        #[doc = "second"]
        #[test]
        fn explicit_attr() {}
    "#;

    let tests = extract_tests(src).expect("Error parsing file");
    let t = by_name(&tests, "explicit_attr");
    assert_eq!(t.docstring, "first\nsecond");
}

/// ------------------- commit-count config generation -------------------

#[test]
fn creates_commit_steps_from_require_commits() {
    let dir = tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "#[test] fn a() {}").unwrap();

    // run(root, tests_dir, default_points, style_check, commit_counts,
    //     num_commit_checks, require_tests, require_commits)
    super::run(
        dir.path(),
        &src_dir,
        1,
        false,
        true,    // commit_counts enabled
        Some(0), // deprecated flag ignored because require_commits present
        0,
        &vec![5, 10, 20],
    )
    .unwrap();

    let items = read_autograder_config(dir.path()).expect("Error Reading Autograder Config");
    let steps = commit_count_steps(&items);
    assert_eq!(steps.len(), 3);

    let mins: Vec<u32> = steps.iter().map(|s| min_commits(s).unwrap()).collect();
    assert_eq!(mins, vec![5, 10, 20]);
}

#[test]
fn require_commits_overrides_num_commit_checks() {
    let dir = tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "#[test] fn a() {}").unwrap();

    super::run(
        dir.path(),
        &src_dir,
        1,
        false,
        true,
        Some(4), // would expand to 1..=4, BUT must be ignored
        0,
        &vec![2, 8, 16], // precedence
    )
    .unwrap();

    let items = read_autograder_config(dir.path()).expect("Error Reading Autograder Config");
    let steps = commit_count_steps(&items);
    let mins: Vec<u32> = steps.iter().map(|s| min_commits(s).unwrap()).collect();
    assert_eq!(mins, vec![2, 8, 16]);
}

#[test]
fn expands_num_commit_checks_when_require_commits_missing() {
    let dir = tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "#[test] fn a() {}").unwrap();

    super::run(
        dir.path(),
        &src_dir,
        1,
        false,
        true,
        Some(3), // deprecated flag still supported → 1..=3
        0,
        &vec![], // missing => use num_commit_checks
    )
    .unwrap();

    let items = read_autograder_config(dir.path()).expect("Error Reading Autograder Config");
    let steps = commit_count_steps(&items);
    let mins: Vec<u32> = steps.iter().map(|s| min_commits(s).unwrap()).collect();
    assert_eq!(mins, vec![1, 2, 3]);
}

#[test]
fn does_not_create_commit_steps_if_disabled() {
    let dir = tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "#[test] fn a() {}").unwrap();

    super::run(
        dir.path(),
        &src_dir,
        1,
        false,    // style_check
        false,    // commit_counts disabled
        Some(99), // ignored
        0,
        &vec![1, 2, 3, 4, 5], // ignored
    )
    .unwrap();

    let items = read_autograder_config(dir.path()).expect("Error Reading Autograder Config");
    assert!(commit_count_steps(&items).is_empty());
}

/// ------------------- test-count config generation -------------------

#[test]
fn manifest_paths_are_distinct_and_relative() {
    let root = PathBuf::from("/repo");
    let test1 = TestWithManifest {
        test: Test {
            name: "a".to_string(),
            docstring: "".to_string(),
        },
        manifest_path: Some(root.join("Cargo.toml")),
    };
    let test2 = TestWithManifest {
        test: Test {
            name: "b".to_string(),
            docstring: "".to_string(),
        },
        manifest_path: Some(root.join("sub/Cargo.toml")),
    };
    let test3 = TestWithManifest {
        test: Test {
            name: "c".to_string(),
            docstring: "".to_string(),
        },
        manifest_path: None,
    };

    let tests = vec![test1, test2, test3];
    let paths = TestWithManifest::get_distinct_manifest_paths(&tests, &root);

    assert!(paths.contains("Cargo.toml"));
    assert!(paths.contains("sub/Cargo.toml"));
    assert_eq!(paths.len(), 2);
}

#[test]
fn manifest_path_none_uses_fallback() {
    let root = PathBuf::from("/repo");
    let test = TestWithManifest {
        test: Test {
            name: "a".to_string(),
            docstring: "".to_string(),
        },
        manifest_path: None,
    };
    let tests = vec![test];
    let paths = TestWithManifest::get_distinct_manifest_paths(&tests, &root);
    assert!(paths.contains("Cargo.toml"));
    assert_eq!(paths.len(), 1);
}

#[test]
fn manifest_path_absolute_is_preserved() {
    let root = PathBuf::from("/repo");
    let abs_manifest = PathBuf::from("/other/path/Cargo.toml");
    let test = TestWithManifest {
        test: Test {
            name: "a".to_string(),
            docstring: "".to_string(),
        },
        manifest_path: Some(abs_manifest.clone()),
    };
    let tests = vec![test];
    let paths = TestWithManifest::get_distinct_manifest_paths(&tests, &root);
    assert!(paths.contains("/other/path/Cargo.toml"));
}

#[test]
fn test_count_member_uses_member_manifest_path_in_name() {
    let dir = tempdir().unwrap();

    fs::create_dir_all(dir.path().join("member/src")).unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace]\nmembers=[\"member\"]\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("member/Cargo.toml"),
        "[package]\nname=\"member\"\nversion=\"0.1.0\"\n",
    )
    .unwrap();
    fs::write(dir.path().join("member/src/lib.rs"), "#[test] fn m() {}").unwrap();

    super::run(
        dir.path(),
        dir.path(),
        1,
        false,
        false,
        Some(1),
        1, // require at least 1 test
        &vec![],
    )
    .unwrap();

    let items = read_autograder_config(dir.path()).expect("Error Reading Autograder Config");
    let member = test_count_steps(&items)
        .into_iter()
        .find(|s| manifest_path_str(s) == Some("member/Cargo.toml"))
        .expect("missing member TEST_COUNT");

    assert!(
        member.meta.name.contains("MEMBER/CARGO.TOML"),
        "expected uppercased manifest path in name; got {}",
        member.meta.name
    );
}

#[test]
fn creates_single_test_count_step_for_root_when_required() {
    let dir = tempdir().unwrap();

    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname=\"root\"\nversion=\"0.1.0\"\n",
    )
    .unwrap();
    fs::write(dir.path().join("src/lib.rs"), "#[test] fn a() {}").unwrap();

    super::run(
        dir.path(),
        dir.path(),
        1,       // default points
        false,   // style_check
        false,   // commit_counts
        Some(1), // ignored
        3,       // require_tests
        &vec![],
    )
    .unwrap();

    let items = read_autograder_config(dir.path()).expect("Error Reading Autograder Config");
    let steps = test_count_steps(&items);
    assert_eq!(
        steps.len(),
        1,
        "expected exactly one TEST_COUNT step for root"
    );

    let s = steps[0];
    assert_eq!(s.meta.name, "TEST_COUNT");
    assert_eq!(min_tests(s), Some(3));
    assert!(
        manifest_path_str(s).is_none(),
        "root should not store manifest_path"
    );
    assert_eq!(s.resolved_description(), "Submission has at least 3 tests");
}

#[test]
fn creates_test_count_steps_for_each_manifest_in_workspace() {
    let dir = tempdir().unwrap();

    fs::write(
        dir.path().join("Cargo.toml"),
        r#"[workspace]
members = ["member"]
"#,
    )
    .unwrap();

    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/lib.rs"), "#[test] fn root_test() {}").unwrap();

    fs::create_dir_all(dir.path().join("member/src")).unwrap();
    fs::write(
        dir.path().join("member/Cargo.toml"),
        r#"[package]
name = "member"
version = "0.1.0"
"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("member/src/lib.rs"),
        "#[test] fn member_test() {}",
    )
    .unwrap();

    super::run(
        dir.path(),
        dir.path(),
        2,       // default points
        true,    // style_check (irrelevant here)
        false,   // commit_counts
        Some(3), // ignored
        5,       // require_tests
        &vec![],
    )
    .unwrap();

    let items = read_autograder_config(dir.path()).expect("Error Reading Autograder Config");
    let steps = test_count_steps(&items);
    assert_eq!(
        steps.len(),
        2,
        "expected TEST_COUNT for both root and member"
    );

    // Root
    let root = steps
        .iter()
        .find(|s| manifest_path_str(s).is_none())
        .expect("missing root TEST_COUNT");
    assert_eq!(root.meta.name, "TEST_COUNT");
    assert_eq!(root.meta.points, 2);
    assert_eq!(min_tests(root), Some(5));
    assert_eq!(
        root.resolved_description(),
        "Submission has at least 5 tests"
    );

    // Member
    let member = steps
        .iter()
        .find(|s| manifest_path_str(s) == Some("member/Cargo.toml"))
        .expect("missing member TEST_COUNT");

    assert!(member.meta.name.starts_with("TEST_COUNT_"));
    assert!(
        member.meta.name.contains("MEMBER/CARGO.TOML"),
        "member TEST_COUNT name should include uppercased manifest path; got {}",
        member.meta.name
    );
    assert_eq!(member.meta.points, 2);
    assert_eq!(min_tests(member), Some(5));
    assert_eq!(
        member.resolved_description(),
        "member submission has at least 5 tests"
    );
}

#[test]
fn does_not_create_test_count_steps_when_disabled() {
    let dir = tempdir().unwrap();

    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname=\"root\"\nversion=\"0.1.0\"\n",
    )
    .unwrap();
    fs::write(dir.path().join("src/lib.rs"), "#[test] fn a() {}").unwrap();

    super::run(
        dir.path(),
        dir.path(),
        1,
        false,   // style_check
        true,    // commit_counts (unrelated)
        Some(2), // creates commit steps, but not test-count steps
        0,       // require_tests disabled
        &vec![],
    )
    .unwrap();

    let items = read_autograder_config(dir.path()).expect("Error Reading Autograder Config");
    assert!(
        test_count_steps(&items).is_empty(),
        "no TEST_COUNT when require_tests == 0"
    );
}
