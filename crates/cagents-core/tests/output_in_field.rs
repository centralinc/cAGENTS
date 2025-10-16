// Test new outputIn field for glob output location control
use cagents_core::loader::Rule;
use cagents_core::model::RuleFrontmatter;
use cagents_core::planner::{BuildContext, plan_outputs};
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
#[serial]
fn test_output_in_matched_for_directory_globs() {
    let tmp = TempDir::new().unwrap();
    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    // Create test directories
    fs::create_dir_all("crates/foo/tests").unwrap();
    fs::create_dir_all("crates/bar/tests").unwrap();
    fs::write("crates/foo/tests/test.rs", "").unwrap();
    fs::write("crates/bar/tests/test.rs", "").unwrap();

    let rule = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("test-rules".to_string()),
            globs: Some(vec!["crates/**/tests/".to_string()]), // Trailing slash = directory match
            output_in: Some("matched".to_string()), // Output IN the matched directory
            ..Default::default()
        },
        body: "Test rules".to_string(),
        path: PathBuf::from("test.md"),
    };

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    std::env::set_current_dir(original).unwrap();

    // Should create outputs IN the matched directories
    assert!(outputs.contains_key(&PathBuf::from("crates/foo/tests")));
    assert!(outputs.contains_key(&PathBuf::from("crates/bar/tests")));
}

#[test]
#[serial]
fn test_output_in_parent_for_file_globs() {
    let tmp = TempDir::new().unwrap();
    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    // Create test files
    fs::create_dir_all("src").unwrap();
    fs::write("src/main.rs", "").unwrap();
    fs::write("src/lib.rs", "").unwrap();

    let rule = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("rust-rules".to_string()),
            globs: Some(vec!["src/**/*.rs".to_string()]),
            output_in: Some("parent".to_string()), // Output in parent of matched files
            ..Default::default()
        },
        body: "Rust rules".to_string(),
        path: PathBuf::from("rust.md"),
    };

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    std::env::set_current_dir(original).unwrap();

    // Should create output in src/ (parent of matched files)
    assert!(outputs.contains_key(&PathBuf::from("src")));
}

#[test]
#[serial]
fn test_output_in_common_parent() {
    let tmp = TempDir::new().unwrap();
    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    // Create files in multiple directories
    fs::create_dir_all("src/api").unwrap();
    fs::create_dir_all("src/db").unwrap();
    fs::write("src/api/users.rs", "").unwrap();
    fs::write("src/db/schema.rs", "").unwrap();

    let rule = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("all-rust".to_string()),
            globs: Some(vec!["src/**/*.rs".to_string()]),
            output_in: Some("common-parent".to_string()), // Find common parent
            ..Default::default()
        },
        body: "All Rust rules".to_string(),
        path: PathBuf::from("rust.md"),
    };

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    std::env::set_current_dir(original).unwrap();

    // Should create single output in src/ (common parent)
    assert!(outputs.contains_key(&PathBuf::from("src")));
    assert_eq!(outputs.len(), 1);
}

#[test]
fn test_output_in_field_values() {
    // Test that outputIn field works with different strategies
    let rule_matched = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("test".to_string()),
            globs: Some(vec!["src/**/tests/".to_string()]),
            output_in: Some("matched".to_string()),
            ..Default::default()
        },
        body: "Test".to_string(),
        path: PathBuf::from("test.md"),
    };

    let rule_parent = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("test2".to_string()),
            globs: Some(vec!["src/**/*.rs".to_string()]),
            output_in: Some("parent".to_string()),
            ..Default::default()
        },
        body: "Test2".to_string(),
        path: PathBuf::from("test2.md"),
    };

    let rule_common = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("test3".to_string()),
            globs: Some(vec!["src/**/*.rs".to_string()]),
            output_in: Some("common-parent".to_string()),
            ..Default::default()
        },
        body: "Test3".to_string(),
        path: PathBuf::from("test3.md"),
    };

    // Verify strategies are read correctly
    assert_eq!(rule_matched.frontmatter.get_output_strategy(), "matched");
    assert_eq!(rule_parent.frontmatter.get_output_strategy(), "parent");
    assert_eq!(rule_common.frontmatter.get_output_strategy(), "common-parent");
}
