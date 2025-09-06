use super::{Test, extract_tests};

/// Helper function for test cases
fn extract_test_names(src: &str) -> Vec<String> {
    extract_tests(src)
        .expect("Error parsing file")
        .iter()
        .map(|t| t.name.clone())
        .collect()
}

fn sorted(names: Vec<String>) -> Vec<String> {
    let mut v = names.clone();
    v.sort();
    v
}

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

        // Out-of-line module (we do not open the file here)
        mod helpers;

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

fn by_name<'a>(tests: &'a [Test], name: &str) -> &'a Test {
    tests.iter().find(|t| t.name == name).unwrap_or_else(|| {
        panic!(
            "test `{}` not found in {:?}",
            name,
            tests.iter().map(|t| &t.name).collect::<Vec<_>>()
        )
    })
}

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

    // Rust translates block doc comments to multiple #[doc="..."] lines.
    // Leading asterisks/spaces are normalized by the lexer; expect lines joined by '\n'.
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
    // Blank /// line becomes an empty #[doc=""] → yields a lone '\n' between alpha and gamma.
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
