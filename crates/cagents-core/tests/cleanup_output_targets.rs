// Test cleanup of output files when targets are removed from config

use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use std::env;

#[test]
#[serial]
fn test_cleanup_when_removing_output_targets() {
    let tmp = TempDir::new().unwrap();
    let original = env::current_dir().unwrap();
    env::set_current_dir(tmp.path()).unwrap();

    // Create cAGENTS setup with ALL output targets
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
"#).unwrap();

    // Run first build - should create all 3 files
    cagents_core::cmd_build(None, None, None, None, false).unwrap();

    assert!(PathBuf::from("AGENTS.md").exists(), "Should create AGENTS.md");
    assert!(PathBuf::from("CLAUDE.md").exists(), "Should create CLAUDE.md");
    assert!(PathBuf::from(".cursorrules").exists(), "Should create .cursorrules");

    // Update config to only have agents-md target
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[output]
targets = ["agents-md"]
"#).unwrap();

    // Run second build - should clean up CLAUDE.md and .cursorrules
    cagents_core::cmd_build(None, None, None, None, false).unwrap();

    env::set_current_dir(original).unwrap();

    // AGENTS.md should still exist
    assert!(tmp.path().join("AGENTS.md").exists(), "AGENTS.md should still exist");

    // CLAUDE.md and .cursorrules should be cleaned up
    assert!(!tmp.path().join("CLAUDE.md").exists(), "CLAUDE.md should be removed");
    assert!(!tmp.path().join(".cursorrules").exists(), ".cursorrules should be removed");
}

#[test]
#[serial]
fn test_cleanup_handles_default_targets() {
    let tmp = TempDir::new().unwrap();
    let original = env::current_dir().unwrap();
    env::set_current_dir(tmp.path()).unwrap();

    // Create cAGENTS with explicit targets
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[output]
targets = ["agents-md", "claude-md"]
"#).unwrap();

    fs::write(".cAGENTS/templates/main.md", r#"---
name: main
---
# Rules
"#).unwrap();

    // First build
    cagents_core::cmd_build(None, None, None, None, false).unwrap();
    assert!(PathBuf::from("AGENTS.md").exists());
    assert!(PathBuf::from("CLAUDE.md").exists());

    // Remove [output] section entirely (defaults to agents-md only)
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"
"#).unwrap();

    // Second build - should clean up CLAUDE.md
    cagents_core::cmd_build(None, None, None, None, false).unwrap();

    env::set_current_dir(original).unwrap();

    assert!(tmp.path().join("AGENTS.md").exists());
    assert!(!tmp.path().join("CLAUDE.md").exists(), "CLAUDE.md should be cleaned up when targets defaults to agents-md only");
}
