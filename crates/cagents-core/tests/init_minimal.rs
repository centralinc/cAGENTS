// Test that init creates minimal empty config without auto-migration

use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use std::env;

#[test]
#[serial]
fn test_init_creates_empty_config() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Run init
    cagents_core::cmd_init("basic", false, false, false).unwrap();

    // Should create .cAGENTS structure
    assert!(PathBuf::from(".cAGENTS").exists());
    assert!(PathBuf::from(".cAGENTS/config.toml").exists());
    assert!(PathBuf::from(".cAGENTS/templates").exists());

    // Config should be minimal
    let config = fs::read_to_string(".cAGENTS/config.toml").unwrap();

    // Should have basic structure
    assert!(config.contains("[paths]"));
    assert!(config.contains("templatesDir"));
    assert!(config.contains("outputRoot"));

    // Should NOT have templates directory pre-populated
    let templates_dir = PathBuf::from(".cAGENTS/templates");
    let template_files: Vec<_> = fs::read_dir(templates_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .collect();

    assert_eq!(template_files.len(), 0, "Should not create any template files by default");
}

#[test]
#[serial]
fn test_init_does_not_auto_migrate_agents_md() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create existing AGENTS.md
    fs::write("AGENTS.md", "# Existing Rules\nContent here").unwrap();

    // Run init
    cagents_core::cmd_init("basic", false, false, false).unwrap();

    // Should create .cAGENTS
    assert!(PathBuf::from(".cAGENTS").exists());

    // Should NOT migrate or remove AGENTS.md
    assert!(PathBuf::from("AGENTS.md").exists(), "Should NOT remove AGENTS.md during init");
    let agents_content = fs::read_to_string("AGENTS.md").unwrap();
    assert_eq!(agents_content, "# Existing Rules\nContent here", "AGENTS.md should be unchanged");

    // Should NOT create any templates from migration
    let templates_dir = PathBuf::from(".cAGENTS/templates");
    let template_files: Vec<_> = fs::read_dir(templates_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert_eq!(template_files.len(), 0, "Should not auto-create templates from AGENTS.md");
}

#[test]
#[serial]
fn test_init_with_claude_md_does_not_auto_migrate() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create existing CLAUDE.md
    fs::write("CLAUDE.md", "# Claude Rules\nContent here").unwrap();

    // Run init
    cagents_core::cmd_init("basic", false, false, false).unwrap();

    // Should create .cAGENTS
    assert!(PathBuf::from(".cAGENTS").exists());

    // Should NOT migrate or remove CLAUDE.md
    assert!(PathBuf::from("CLAUDE.md").exists(), "Should NOT remove CLAUDE.md during init");

    // Should still show detection message (checked manually in output)
    // but should NOT auto-migrate
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
