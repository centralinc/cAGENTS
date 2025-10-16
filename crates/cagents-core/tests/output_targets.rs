// Test output targets configuration and multi-format build

use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use std::env;

#[test]
#[serial]
fn test_config_with_output_targets() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create config with output targets
    fs::create_dir_all(".cAGENTS").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[output]
targets = ["agents-md", "claude-md", "cursorrules"]
"#).unwrap();

    let config = cagents_core::config::load_config_with_precedence().unwrap();

    // Should parse output targets
    assert!(config.output.is_some());
    let output = config.output.unwrap();
    assert!(output.targets.is_some());
    let targets = output.targets.unwrap();
    assert_eq!(targets.len(), 3);
    assert!(targets.contains(&"agents-md".to_string()));
    assert!(targets.contains(&"claude-md".to_string()));
    assert!(targets.contains(&"cursorrules".to_string()));
}

#[test]
#[serial]
fn test_build_generates_agents_md_by_default() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create basic cAGENTS setup without output.targets specified
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"
"#).unwrap();

    fs::write(".cAGENTS/templates/main.md", r#"---
name: main
---
# Test Rules
"#).unwrap();

    // Run build
    cagents_core::cmd_build(None, false).unwrap();

    // Should create AGENTS.md by default
    assert!(PathBuf::from("AGENTS.md").exists(), "Should create AGENTS.md by default");
    assert!(!PathBuf::from("CLAUDE.md").exists(), "Should not create CLAUDE.md without config");
    assert!(!PathBuf::from(".cursorrules").exists(), "Should not create .cursorrules without config");
}

#[test]
#[serial]
fn test_build_with_multiple_output_targets() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create cAGENTS setup with multiple output targets
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[output]
targets = ["agents-md", "claude-md", "cursorrules"]
"#).unwrap();

    fs::write(".cAGENTS/templates/main.md", r#"---
name: main
---
# Test Rules

Development guidelines here.
"#).unwrap();

    // Run build
    cagents_core::cmd_build(None, false).unwrap();

    // Should create all configured output files
    assert!(PathBuf::from("AGENTS.md").exists(), "Should create AGENTS.md");
    assert!(PathBuf::from("CLAUDE.md").exists(), "Should create CLAUDE.md");
    assert!(PathBuf::from(".cursorrules").exists(), "Should create .cursorrules");

    // Verify content is the same (merged from templates)
    let agents_content = fs::read_to_string("AGENTS.md").unwrap();
    let claude_content = fs::read_to_string("CLAUDE.md").unwrap();
    let cursor_content = fs::read_to_string(".cursorrules").unwrap();

    assert!(agents_content.contains("# Test Rules"));
    assert!(claude_content.contains("# Test Rules"));
    assert!(cursor_content.contains("# Test Rules"));
}

#[test]
#[serial]
fn test_build_with_only_cursorrules_target() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create cAGENTS setup with only cursorrules target
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[output]
targets = ["cursorrules"]
"#).unwrap();

    fs::write(".cAGENTS/templates/main.md", r#"---
name: main
---
# Cursor Rules
"#).unwrap();

    // Run build
    cagents_core::cmd_build(None, false).unwrap();

    // Should only create .cursorrules
    assert!(!PathBuf::from("AGENTS.md").exists(), "Should NOT create AGENTS.md");
    assert!(!PathBuf::from("CLAUDE.md").exists(), "Should NOT create CLAUDE.md");
    assert!(PathBuf::from(".cursorrules").exists(), "Should create .cursorrules");
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
