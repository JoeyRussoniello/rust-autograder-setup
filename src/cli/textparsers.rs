use regex::Regex;

pub fn extract_test_names(src: &str) -> Vec<String> {
    let cleaned = strip_comments(src);

    // Attribute that contains "test" followed by fn <ident>
    // - Accepts optional pub/async between attribute and fn
    // - Handles #[test], #[tokio::test], #[cfg_attr(..., test)], etc. (anything containing "test")
    let re = Regex::new(
        r#"#\s*\[[^\]]*test[^\]]*\]\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)"#,
    )
    .unwrap();

    re.captures_iter(&cleaned)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

pub fn strip_comments(s: &str) -> String {
    // Remove block comments (/* ... */), then line comments (// ...).
    // This is intentionally simple and may remove comment-like text in strings,
    // but is fine for attribute scanning in test files.
    let block = Regex::new(r"(?s)/\*.*?\*/").unwrap().replace_all(s, "");
    let line = Regex::new(r"//.*").unwrap().replace_all(&block, "");
    line.to_string()
}

#[cfg(test)]
fn basic_test_extraction_functionality(){
    let src = r#"
    // This is a test file
    #[test]
    fn test_one() {
        assert_eq!(1, 1);
    }

    #[tokio::test]
    async fn test_two() {
        assert_eq!(2, 2);
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn test_three() {
            assert_eq!(3, 3);
        }
    }

    /* 
    #[test]
    fn commented_out() {
        assert_eq!(0, 0);
    }
    */

    // Another comment
    "#;
    let test_names = extract_test_names(src);
    assert_eq!(test_names, vec!["test_one", "test_two", "test_three"]);
}
