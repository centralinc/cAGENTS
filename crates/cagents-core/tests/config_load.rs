use cagents_core::loader::load_config;
use cagents_core::config::load_config_with_precedence;
use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;
use serial_test::serial;

#[test]
fn test_load_basic_config() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let config_path = workspace_root.join("examples/basic/.cAGENTS/config.toml");
    let config = load_config(&config_path).expect("Failed to load config");

    // Verify paths
    assert_eq!(config.paths.templates_dir, "templates");
    assert_eq!(config.paths.output_root, ".");
    assert_eq!(
        config.paths.cursor_rules_dir,
        Some(".cursor/rules".to_string())
    );

    // Verify defaults
    let defaults = config.defaults.expect("defaults missing");
    assert_eq!(
        defaults.engine,
        Some("command:python3 .cAGENTS/tools/render.py".to_string())
    );
    assert_eq!(defaults.order, Some(50));

    // Verify static variables
    let vars = config.variables.expect("variables missing");
    let static_vars = vars.static_.expect("static vars missing");
    let static_obj = static_vars.as_object().expect("static should be object");
    assert_eq!(static_obj.get("owner").unwrap().as_str().unwrap(), "Jordan");
    assert_eq!(static_obj.get("tone").unwrap().as_str().unwrap(), "concise");
}

#[test]
#[serial]
fn test_partial_local_config() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join(".cAGENTS");
    fs::create_dir_all(&config_dir).expect("Failed to create .cAGENTS dir");

    // Create base project config with all required fields
    let project_config = config_dir.join("config.toml");
    fs::write(
        &project_config,
        r#"
[paths]
templatesDir = "templates"
outputRoot = "."
cursorRulesDir = ".cursor/rules"

[defaults]
engine = "command:python3 render.py"
order = 50

[variables.static]
owner = "Alice"
tone = "formal"
"#,
    )
    .expect("Failed to write project config");

    // Create partial local config that only overrides variables
    // This should be valid even though it doesn't have [paths]
    let local_config = config_dir.join("config.local.toml");
    fs::write(
        &local_config,
        r#"
[variables.static]
tone = "casual"
project = "test-project"
"#,
    )
    .expect("Failed to write local config");

    // Change to temp directory and load config with precedence
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    let result = load_config_with_precedence();

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");

    // This should succeed - local config should be allowed to be partial
    let config = result.expect("Failed to load config with partial local override");

    // Verify paths from base config are preserved
    assert_eq!(config.paths.templates_dir, "templates");
    assert_eq!(config.paths.output_root, ".");

    // Verify variables were merged correctly
    let vars = config.variables.expect("variables missing");
    let static_vars = vars.static_.expect("static vars missing");
    let static_obj = static_vars.as_object().expect("static should be object");

    // Base value
    assert_eq!(static_obj.get("owner").unwrap().as_str().unwrap(), "Alice");
    // Overridden value from local config
    assert_eq!(static_obj.get("tone").unwrap().as_str().unwrap(), "casual");
    // New value from local config
    assert_eq!(
        static_obj.get("project").unwrap().as_str().unwrap(),
        "test-project"
    );
}
