// Exhaustive integration tests for complex real-world scenarios
use serial_test::serial;

use cagents_core::loader::Rule;
use cagents_core::model::RuleFrontmatter;
use cagents_core::planner::{BuildContext, plan_outputs};
use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;
use std::env;

#[test]
#[serial]
fn test_scenario_monorepo_with_context() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create monorepo structure
    fs::create_dir_all("packages/backend/src").unwrap();
    fs::create_dir_all("packages/frontend/src").unwrap();
    fs::write("packages/backend/src/main.rs", "").unwrap();
    fs::write("packages/frontend/src/App.tsx", "").unwrap();

    let rules = vec![
        // Root rule
        Rule {
            frontmatter: RuleFrontmatter {
                name: Some("workspace".to_string()),
                // No when clause = implicitly always applies
                ..Default::default()
            },
            body: "Workspace rules".to_string(),
            path: PathBuf::from("workspace.md"),
        },
        // Backend rule with context
        Rule {
            frontmatter: RuleFrontmatter {
                name: Some("backend".to_string()),
                globs: Some(vec!["packages/backend/**".to_string()]),
                when: Some(cagents_core::model::When::legacy(
                    None,
                    Some(vec!["backend".to_string()]),
                    None,
                    None,
                )),
                simplify_globs_to_parent: Some(false), ..Default::default()
            },
            body: "Backend rules".to_string(),
            path: PathBuf::from("backend.md"),
        },
        // Frontend rule with context
        Rule {
            frontmatter: RuleFrontmatter {
                name: Some("frontend".to_string()),
                globs: Some(vec!["packages/frontend/**".to_string()]),
                when: Some(cagents_core::model::When::legacy(
                    None,
                    Some(vec!["frontend".to_string()]),
                    None,
                    None,
                )),
                simplify_globs_to_parent: Some(false), ..Default::default()
            },
            body: "Frontend rules".to_string(),
            path: PathBuf::from("frontend.md"),
        },
    ];

    // Build for backend role
    let context = BuildContext::new(None, Some("backend".to_string()), None);
    let outputs = plan_outputs(&rules, &context, &PathBuf::from(".")).unwrap();

    // plan_outputs now includes all rules; when filtering happens per-target in build
    // So both backend and frontend directories will be in outputs
    assert!(outputs.contains_key(&PathBuf::from(".")));
    assert!(outputs.contains_key(&PathBuf::from("packages/backend/src")));
    assert!(outputs.contains_key(&PathBuf::from("packages/frontend/src")),
        "plan_outputs includes all globs; when filtering happens in build");
}

#[test]
#[serial]
fn test_scenario_language_specific_rules() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all("src").unwrap();
    fs::write("src/main.rs", "").unwrap();
    fs::write("src/index.ts", "").unwrap();
    fs::write("src/app.py", "").unwrap();

    let rules = vec![
        Rule {
            frontmatter: RuleFrontmatter {
                name: Some("rust".to_string()),
                globs: Some(vec!["**/*.rs".to_string()]),
                when: Some(cagents_core::model::When::legacy(
                    None,
                    None,
                    Some(vec!["Rust".to_string()]),
                    None,
                )),
                simplify_globs_to_parent: Some(false), ..Default::default()
            },
            body: "Rust rules".to_string(),
            path: PathBuf::from("rust.md"),
        },
        Rule {
            frontmatter: RuleFrontmatter {
                name: Some("typescript".to_string()),
                globs: Some(vec!["**/*.ts".to_string()]),
                when: Some(cagents_core::model::When::legacy(
                    None,
                    None,
                    Some(vec!["TypeScript".to_string()]),
                    None,
                )),
                simplify_globs_to_parent: Some(false), ..Default::default()
            },
            body: "TS rules".to_string(),
            path: PathBuf::from("ts.md"),
        },
    ];

    // Build for Rust only
    let context = BuildContext::new(None, None, Some("Rust".to_string()));
    let outputs = plan_outputs(&rules, &context, &PathBuf::from(".")).unwrap();

    let src_rules = outputs.get(&PathBuf::from("src")).unwrap();
    // plan_outputs now includes all rules with matching globs
    assert_eq!(src_rules.len(), 2, "plan_outputs includes all rules with matching globs");

    // Verify when clause filtering via matches_when
    let rust_rule = src_rules.iter().find(|r| r.frontmatter.name.as_deref() == Some("rust")).unwrap();
    let ts_rule = src_rules.iter().find(|r| r.frontmatter.name.as_deref() == Some("typescript")).unwrap();

    assert!(context.matches_when(&rust_rule.frontmatter.when), "Rust context should match rust rule");
    assert!(!context.matches_when(&ts_rule.frontmatter.when), "Rust context should not match TS rule");
}

#[test]
#[serial]
fn test_scenario_mixed_always_apply_and_globs() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all("src").unwrap();
    fs::write("src/main.rs", "").unwrap();

    let rules = vec![
        Rule {
            frontmatter: RuleFrontmatter {
                name: Some("global".to_string()),
                // No when clause = implicitly always applies
                ..Default::default()
            },
            body: "Global rules".to_string(),
            path: PathBuf::from("global.md"),
        },
        Rule {
            frontmatter: RuleFrontmatter {
                name: Some("rust".to_string()),
                globs: Some(vec!["**/*.rs".to_string()]),
                simplify_globs_to_parent: Some(false), ..Default::default()
            },
            body: "Rust rules".to_string(),
            path: PathBuf::from("rust.md"),
        },
    ];

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&rules, &context, &PathBuf::from(".")).unwrap();

    // Root should have global rule
    assert!(outputs.contains_key(&PathBuf::from(".")));
    assert_eq!(outputs[&PathBuf::from(".")].len(), 1);
    assert_eq!(outputs[&PathBuf::from(".")][0].frontmatter.name.as_deref(), Some("global"));

    // src should have rust rule
    assert!(outputs.contains_key(&PathBuf::from("src")));
    assert_eq!(outputs[&PathBuf::from("src")][0].frontmatter.name.as_deref(), Some("rust"));
}

#[test]
#[serial]
fn test_scenario_order_matters() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all("src").unwrap();
    fs::write("src/main.rs", "").unwrap();

    let rules = vec![
        Rule {
            frontmatter: RuleFrontmatter {
                name: Some("high".to_string()),
                globs: Some(vec!["**/*.rs".to_string()]),
                order: Some(100),
                simplify_globs_to_parent: Some(false), ..Default::default()
            },
            body: "High order".to_string(),
            path: PathBuf::from("high.md"),
        },
        Rule {
            frontmatter: RuleFrontmatter {
                name: Some("low".to_string()),
                globs: Some(vec!["**/*.rs".to_string()]),
                order: Some(1),
                simplify_globs_to_parent: Some(false), ..Default::default()
            },
            body: "Low order".to_string(),
            path: PathBuf::from("low.md"),
        },
    ];

    let context = BuildContext::new(None, None, None);
    let outputs = plan_outputs(&rules, &context, &PathBuf::from(".")).unwrap();

    let src_rules = outputs.get(&PathBuf::from("src")).unwrap();
    assert_eq!(src_rules.len(), 2);
    // Note: plan_outputs doesn't sort, that happens during build
}

#[test]
#[serial]
fn test_scenario_environment_based_rules() {
    let rule_prod = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("prod-only".to_string()),
            when: Some(cagents_core::model::When::legacy(
                Some(vec!["prod".to_string(), "staging".to_string()]),
                None,
                None,
                None,
            )),
            ..Default::default()
        },
        body: "Production rules".to_string(),
        path: PathBuf::from("prod.md"),
    };

    let rule_dev = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("dev-only".to_string()),
            when: Some(cagents_core::model::When::legacy(
                Some(vec!["dev".to_string()]),
                None,
                None,
                None,
            )),
            ..Default::default()
        },
        body: "Dev rules".to_string(),
        path: PathBuf::from("dev.md"),
    };

    // Test prod context
    let context_prod = BuildContext::new(Some("prod".to_string()), None, None);
    let outputs_prod = plan_outputs(&[rule_prod.clone(), rule_dev.clone()], &context_prod, &PathBuf::from(".")).unwrap();
    let root_rules_prod = outputs_prod.get(&PathBuf::from(".")).unwrap();
    // plan_outputs now includes all rules; when filtering happens in build
    assert_eq!(root_rules_prod.len(), 2, "plan_outputs includes all rules");

    // Test dev context
    let context_dev = BuildContext::new(Some("dev".to_string()), None, None);
    let outputs_dev = plan_outputs(&[rule_prod, rule_dev], &context_dev, &PathBuf::from(".")).unwrap();
    let root_rules_dev = outputs_dev.get(&PathBuf::from(".")).unwrap();
    assert_eq!(root_rules_dev.len(), 2, "plan_outputs includes all rules");
}

#[test]
#[serial]
fn test_scenario_all_filters_combined() {
    let rule = Rule {
        frontmatter: RuleFrontmatter {
            name: Some("specific".to_string()),
            when: Some(cagents_core::model::When::legacy(
                Some(vec!["prod".to_string()]),
                Some(vec!["backend".to_string()]),
                Some(vec!["Rust".to_string()]),
                None,
            )),
            ..Default::default()
        },
        body: "Specific rules".to_string(),
        path: PathBuf::from("specific.md"),
    };

    // All match
    let context_match = BuildContext::new(
        Some("prod".to_string()),
        Some("backend".to_string()),
        Some("Rust".to_string())
    );
    let outputs_match = plan_outputs(std::slice::from_ref(&rule), &context_match, &PathBuf::from(".")).unwrap();
    assert_eq!(outputs_match.get(&PathBuf::from(".")).unwrap().len(), 1);

    // One mismatch - plan_outputs still includes the rule
    let context_nomatch = BuildContext::new(
        Some("dev".to_string()), // Wrong env
        Some("backend".to_string()),
        Some("Rust".to_string())
    );
    let outputs_nomatch = plan_outputs(std::slice::from_ref(&rule), &context_nomatch, &PathBuf::from(".")).unwrap();
    // plan_outputs includes the rule; when filtering happens in build
    assert!(outputs_nomatch.contains_key(&PathBuf::from(".")), "plan_outputs includes all rules");

    // But matches_when should return false
    assert!(!context_nomatch.matches_when(&rule.frontmatter.when), "Context should not match when clause");
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
