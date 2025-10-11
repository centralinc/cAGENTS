use cagents_core::loader::load_config;
use std::path::PathBuf;

#[test]
fn test_config_with_command_vars() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let config_path = workspace_root.join("examples/basic/.cAGENTS/config.toml");
    let config = load_config(&config_path).expect("Failed to load config");

    // Verify command variables exist in config
    let vars = config.variables.expect("variables missing");
    let command_vars = vars.command.expect("command vars missing");
    let command_obj = command_vars.as_object().expect("command should be object");

    // Verify the branch command is present
    assert!(command_obj.contains_key("branch"));
    assert_eq!(
        command_obj.get("branch").unwrap().as_str().unwrap(),
        "git rev-parse --abbrev-ref HEAD"
    );
}
