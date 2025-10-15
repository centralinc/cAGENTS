use cagents_core::loader::{load_config, discover_rules};
use std::path::PathBuf;

#[test]
fn test_discover_rules() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let config_path = workspace_root.join("examples/basic/.cAGENTS/config.toml");
    let config = load_config(&config_path).expect("Failed to load config");

    let base_dir = config_path.parent().unwrap();
    let rules = discover_rules(&config, base_dir).expect("Failed to discover rules");

    // Should find at least the typescript.hbs.md template
    assert!(!rules.is_empty(), "Should discover at least one rule");

    // Find typescript rule
    let ts_rule = rules.iter().find(|r| {
        r.frontmatter.name.as_deref() == Some("ts-basics")
    }).expect("Should find typescript rule");

    // Verify frontmatter fields
    assert_eq!(ts_rule.frontmatter.name, Some("ts-basics".to_string()));
    assert_eq!(ts_rule.frontmatter.order, Some(10));
    assert!(ts_rule.frontmatter.globs.is_some());

    let globs = ts_rule.frontmatter.globs.as_ref().unwrap();
    assert!(globs.contains(&"**/*.ts".to_string()));

    // Verify body contains expected content
    assert!(ts_rule.body.contains("TypeScript"));
    assert!(ts_rule.body.contains("Prefer explicit types"));
}
