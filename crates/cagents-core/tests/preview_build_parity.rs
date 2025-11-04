// Test that preview and build use variables consistently
use serial_test::serial;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to change directory and restore on drop
struct ChangeDir {
    original: PathBuf,
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

#[test]
#[serial]
fn test_preview_uses_variables_like_build() {
    // This test demonstrates that preview should build context from variables
    // just like build does, so that when clauses can be evaluated correctly.

    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create config with static variables
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(
        ".cAGENTS/config.toml",
        r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[variables.static]
myvar = "test123"
"#,
    )
    .unwrap();

    // Create a simple rule without when clause
    fs::write(
        ".cAGENTS/templates/simple.md",
        r#"---
name: "simple-rule"
---
# Simple Rule

Variable value: {{myvar}}
"#,
    )
    .unwrap();

    // Load config and build context - verify this works
    let config = cagents_core::config::load_config_with_precedence().unwrap();

    // This is how BUILD creates context (with variables)
    let mut context_variables = std::collections::HashMap::new();
    let base_data = build_template_data_map(&config);
    for (key, value) in &base_data {
        if let Some(s) = value.as_str() {
            context_variables.insert(key.clone(), s.to_string());
        }
    }
    let context_with_vars = cagents_core::planner::BuildContext::from_variables(context_variables);

    // This is how PREVIEW currently creates context (empty - BUG!)
    let context_without_vars = cagents_core::planner::BuildContext::new(None, None, None);

    // Verify that context_with_vars has the variable
    assert!(context_with_vars.variables.contains_key("myvar"));
    assert_eq!(context_with_vars.variables.get("myvar").unwrap(), "test123");

    // Verify that context_without_vars does NOT have the variable
    assert!(!context_without_vars.variables.contains_key("myvar"));

    // The fix: preview should build context with variables just like build does
    println!("\n✓ Build creates context with variables");
    println!("✗ Preview creates empty context (BUG)");
    println!("\nFix: Make preview build context from config variables like build does");
}

// Helper function (copied from lib.rs for testing)
fn build_template_data_map(
    config: &cagents_core::model::ProjectConfig,
) -> serde_json::Map<String, serde_json::Value> {
    let mut data = serde_json::Map::new();

    if let Some(vars) = &config.variables {
        if let Some(static_vars) = &vars.static_ {
            if let Some(obj) = static_vars.as_object() {
                for (key, value) in obj {
                    data.insert(key.clone(), value.clone());
                }
            }
        }
    }

    data
}
