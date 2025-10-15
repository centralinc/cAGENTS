use cagents_core::{planner, loader, model};
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
#[serial]
fn test_simplify_globs_to_parent_enabled() {
    let temp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(temp.path());

    // Create directory structure
    fs::create_dir_all("src/api").unwrap();
    fs::create_dir_all("src/db").unwrap();
    fs::create_dir_all("src/cache").unwrap();

    // Create files that match patterns
    fs::write("src/api/users.int.ts", "test").unwrap();
    fs::write("src/db/migrations.int.ts", "test").unwrap();
    fs::write("src/cache/redis.int.ts", "test").unwrap();

    // Create a rule with multiple globs and simplifyGlobsToParent: true
    let rule = loader::Rule {
        frontmatter: model::RuleFrontmatter {
            name: Some("integration-tests".to_string()),
            globs: Some(vec![
                "src/api/**/*.int.ts".to_string(),
                "src/db/**/*.int.ts".to_string(),
                "src/cache/**/*.int.ts".to_string(),
            ]),
            simplify_globs_to_parent: Some(true),
            ..Default::default()
        },
        body: "# Integration Test Rules".to_string(),
        path: PathBuf::from("test.md"),
    };

    let context = planner::BuildContext::new(None, None, None);
    let outputs = planner::plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    // Should create AGENTS.md only in common parent (src/)
    assert_eq!(outputs.len(), 1, "Should have single output");
    assert!(outputs.contains_key(&PathBuf::from("src")),
        "Should output to src/ (common parent)");

    // Should NOT create in subdirectories
    assert!(!outputs.contains_key(&PathBuf::from("src/api")));
    assert!(!outputs.contains_key(&PathBuf::from("src/db")));
    assert!(!outputs.contains_key(&PathBuf::from("src/cache")));
}

#[test]
#[serial]
fn test_simplify_globs_to_parent_disabled() {
    let temp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(temp.path());

    // Create directory structure
    fs::create_dir_all("src/api").unwrap();
    fs::create_dir_all("src/db").unwrap();

    // Create files
    fs::write("src/api/test.ts", "test").unwrap();
    fs::write("src/db/test.ts", "test").unwrap();

    // Create a rule with simplifyGlobsToParent: false
    let rule = loader::Rule {
        frontmatter: model::RuleFrontmatter {
            name: Some("services".to_string()),
            globs: Some(vec![
                "src/api/**/*.ts".to_string(),
                "src/db/**/*.ts".to_string(),
            ]),
            simplify_globs_to_parent: Some(false),
            ..Default::default()
        },
        body: "# Rules".to_string(),
        path: PathBuf::from("test.md"),
    };

    let context = planner::BuildContext::new(None, None, None);
    let outputs = planner::plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    // Should create AGENTS.md in each matching directory
    assert!(outputs.len() >= 2, "Should have multiple outputs");
    assert!(outputs.contains_key(&PathBuf::from("src/api")) ||
            outputs.contains_key(&PathBuf::from("src/db")),
        "Should output to subdirectories when simplify disabled");
}

#[test]
#[serial]
fn test_default_simplify_globs_is_true() {
    let temp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(temp.path());

    fs::create_dir_all("components/Button").unwrap();
    fs::create_dir_all("components/Form").unwrap();
    fs::write("components/Button/index.tsx", "test").unwrap();
    fs::write("components/Form/index.tsx", "test").unwrap();

    // Rule without simplifyGlobsToParent field (should default to true)
    let rule = loader::Rule {
        frontmatter: model::RuleFrontmatter {
            name: Some("components".to_string()),
            globs: Some(vec![
                "components/Button/**/*.tsx".to_string(),
                "components/Form/**/*.tsx".to_string(),
            ]),
            simplify_globs_to_parent: None, // Default should be true
            ..Default::default()
        },
        body: "# Component Rules".to_string(),
        path: PathBuf::from("test.md"),
    };

    let context = planner::BuildContext::new(None, None, None);
    let outputs = planner::plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    // Should simplify to common parent by default
    assert!(outputs.contains_key(&PathBuf::from("components")),
        "Should default to simplifying to parent");
}

/// Helper to change directory and restore on drop
struct ChangeDir {
    original: std::path::PathBuf,
}

impl ChangeDir {
    fn new(path: &std::path::Path) -> Self {
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();
        Self { original }
    }
}

impl Drop for ChangeDir {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}
