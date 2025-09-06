use super::extract_test_names;

fn sorted(mut v: Vec<String>) -> Vec<String> {
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
    assert_eq!(sorted(extract_test_names(src)), vec!["Kebab__ok", "with_doc"]);
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

