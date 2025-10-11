use cagents_core::loader::load_config;
use std::path::PathBuf;

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
