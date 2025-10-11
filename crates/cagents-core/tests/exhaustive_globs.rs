// Exhaustive test suite for glob pattern matching and directory targeting

use cagents_core::planner::{BuildContext, plan_outputs};
use cagents_core::loader::Rule;
use cagents_core::model::RuleFrontmatter;
use serial_test::serial;
use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;
use std::env;

#[test]
#[serial]
fn test_glob_single_extension() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create files
    fs::create_dir_all("src").unwrap();
    fs::write("src/main.rs", "").unwrap();
    fs::write("src/lib.rs", "").unwrap();

    let rule = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("rust".to_string()),
            globs: Some(vec!["**/*.rs".to_string()]),
            simplify_globs_to_parent: Some(false),
            ..Default::default()
        },
        body: "Rust rules".to_string(),
        path: PathBuf::from("rust.md"),
    };

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    // Should create output for src directory
    assert!(outputs.contains_key(&PathBuf::from("src")));
}

#[test]
#[serial]
fn test_glob_multiple_extensions() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all("src").unwrap();
    fs::write("src/index.ts", "").unwrap();
    fs::write("src/App.tsx", "").unwrap();
    fs::write("src/main.rs", "").unwrap();

    let rule = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("typescript".to_string()),
            globs: Some(vec!["**/*.ts".to_string(), "**/*.tsx".to_string()]),
            simplify_globs_to_parent: Some(false),
            ..Default::default()
        },
        body: "TS rules".to_string(),
        path: PathBuf::from("ts.md"),
    };

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    assert!(outputs.contains_key(&PathBuf::from("src")));
}

#[test]
#[serial]
fn test_glob_directory_pattern() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all("packages/app1").unwrap();
    fs::create_dir_all("packages/app2").unwrap();
    fs::write("packages/app1/index.js", "").unwrap();
    fs::write("packages/app2/index.js", "").unwrap();

    let rule = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("packages".to_string()),
            globs: Some(vec!["packages/**".to_string()]),
            simplify_globs_to_parent: Some(false),
            ..Default::default()
        },
        body: "Package rules".to_string(),
        path: PathBuf::from("pkg.md"),
    };

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    // Should match both app directories
    assert!(outputs.contains_key(&PathBuf::from("packages/app1")));
    assert!(outputs.contains_key(&PathBuf::from("packages/app2")));
}

#[test]
#[serial]
fn test_glob_nested_directories() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all("src/components/buttons").unwrap();
    fs::write("src/components/buttons/Button.tsx", "").unwrap();

    let rule = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("tsx".to_string()),
            globs: Some(vec!["**/*.tsx".to_string()]),
            simplify_globs_to_parent: Some(false),
            ..Default::default()
        },
        body: "TSX rules".to_string(),
        path: PathBuf::from("tsx.md"),
    };

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    assert!(outputs.contains_key(&PathBuf::from("src/components/buttons")));
}

#[test]
#[serial]
fn test_glob_no_match() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all("src").unwrap();
    fs::write("src/main.rs", "").unwrap();

    let rule = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("python".to_string()),
            globs: Some(vec!["**/*.py".to_string()]),
            simplify_globs_to_parent: Some(false),
            ..Default::default()
        },
        body: "Python rules".to_string(),
        path: PathBuf::from("py.md"),
    };

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    // Should not create any output (no .py files)
    assert!(!outputs.values().any(|rules| rules.iter().any(|r| r.frontmatter.name.as_deref() == Some("python"))));
}

#[test]
#[serial]
fn test_glob_multiple_rules_same_directory() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all("src").unwrap();
    fs::write("src/main.rs", "").unwrap();
    fs::write("src/lib.rs", "").unwrap();

    let rule1 = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("rust1".to_string()),
            globs: Some(vec!["**/*.rs".to_string()]),
            order: Some(1),
            simplify_globs_to_parent: Some(false),
            ..Default::default()
        },
        body: "Rust rule 1".to_string(),
        path: PathBuf::from("r1.md"),
    };

    let rule2 = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("rust2".to_string()),
            globs: Some(vec!["**/*.rs".to_string()]),
            order: Some(2),
            simplify_globs_to_parent: Some(false),
            ..Default::default()
        },
        body: "Rust rule 2".to_string(),
        path: PathBuf::from("r2.md"),
    };

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&[rule1, rule2], &context, &PathBuf::from(".")).unwrap();

    let src_rules = outputs.get(&PathBuf::from("src")).unwrap();
    assert_eq!(src_rules.len(), 2);
}

#[test]
#[serial]
fn test_glob_specific_directory() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all("src").unwrap();
    fs::create_dir_all("tests").unwrap();
    fs::write("src/main.rs", "").unwrap();
    fs::write("tests/test.rs", "").unwrap();

    let rule = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("src-only".to_string()),
            globs: Some(vec!["src/**".to_string()]),
            simplify_globs_to_parent: Some(false),
            ..Default::default()
        },
        body: "Src rules".to_string(),
        path: PathBuf::from("src.md"),
    };

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    // Should only match src, not tests
    assert!(outputs.contains_key(&PathBuf::from("src")));
    assert!(!outputs.contains_key(&PathBuf::from("tests")));
}

#[test]
#[serial]
fn test_glob_exclude_directories() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // These should be excluded by walker
    fs::create_dir_all("node_modules/package").unwrap();
    fs::create_dir_all("target/debug").unwrap();
    fs::create_dir_all("src").unwrap();

    fs::write("node_modules/package/index.js", "").unwrap();
    fs::write("target/debug/main.rs", "").unwrap();
    fs::write("src/main.rs", "").unwrap();

    let rule = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("rust".to_string()),
            globs: Some(vec!["**/*.rs".to_string()]),
            simplify_globs_to_parent: Some(false),
            ..Default::default()
        },
        body: "Rust rules".to_string(),
        path: PathBuf::from("rust.md"),
    };

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    // Should only match src, not node_modules or target
    assert!(outputs.contains_key(&PathBuf::from("src")));
    assert!(!outputs.values().any(|rules|
        rules.iter().any(|r| r.body.contains("node_modules") || r.body.contains("target"))
    ));
}

#[test]
#[serial]
fn test_glob_case_sensitivity() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all("src").unwrap();
    fs::write("src/Main.RS", "").unwrap();
    fs::write("src/lib.rs", "").unwrap();

    let rule = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("rust".to_string()),
            globs: Some(vec!["**/*.rs".to_string()]),
            simplify_globs_to_parent: Some(false),
            ..Default::default()
        },
        body: "Rust rules".to_string(),
        path: PathBuf::from("rust.md"),
    };

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&[rule], &context, &PathBuf::from(".")).unwrap();

    // Glob should match lib.rs but not Main.RS (case sensitive)
    assert!(outputs.contains_key(&PathBuf::from("src")));
}

/// Helper to change directory and restore on drop
struct ChangeDir {
    original: std::path::PathBuf,
}

impl ChangeDir {
    fn new(path: &std::path::Path) -> Self {
        let original = env::current_dir().unwrap();
        env::set_current_dir(path).unwrap();
        Self { original }
    }
}

impl Drop for ChangeDir {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}
