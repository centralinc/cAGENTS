use cagents_core::init::{init_basic, ProjectInfo};
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
#[serial]
fn test_init_basic_creates_structure() {
    let tmp = TempDir::new().unwrap();

    let info = ProjectInfo {
        name: "test-project".to_string(),
        owner: Some("Alice".to_string()),
        has_git: false,
        has_agents_md: false,
        has_cagents_dir: false,
        has_claude_md: false,
        has_cursorrules: false,
        has_cursor_rules: false,
        agents_md_locations: vec![],
    };

    // Change to temp directory before running init
    let _guard = ChangeDir::new(tmp.path());
    init_basic(&info, false, false).unwrap();

    // Verify structure (files are in current dir, which is tmp.path())
    assert!(PathBuf::from(".cAGENTS").exists(), "cAGENTS dir should exist");
    assert!(PathBuf::from(".cAGENTS/config.toml").exists());
    assert!(PathBuf::from(".cAGENTS/templates").exists());
    assert!(PathBuf::from(".cAGENTS/.gitignore").exists());

    // Verify config content is minimal (no pre-filled variables)
    let config = fs::read_to_string(".cAGENTS/config.toml").unwrap();
    assert!(config.contains("[paths]"));
    assert!(config.contains("templatesDir"));
    assert!(config.contains("[variables.static]"));
    assert!(config.contains("[defaults]"));

    // Should NOT create any templates by default
    let templates_count = fs::read_dir(".cAGENTS/templates")
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .count();
    assert_eq!(templates_count, 0, "Should not create template files");

    // Verify AGENTS.md was NOT auto-generated
    assert!(!PathBuf::from("AGENTS.md").exists(), "Should NOT auto-generate AGENTS.md")
}

#[test]
#[serial]
fn test_init_fails_when_exists() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    let info = ProjectInfo {
        name: "test".to_string(),
        owner: None,
        has_git: false,
        has_agents_md: false,
        has_cagents_dir: false,
        has_claude_md: false,
        has_cursorrules: false,
        has_cursor_rules: false,
        agents_md_locations: vec![],
    };

    // First init should succeed
    init_basic(&info, false, false).unwrap();

    // Second init should fail
    let info2 = ProjectInfo {
        has_cagents_dir: true,
        ..info
    };
    let result = init_basic(&info2, false, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[test]
#[serial]
fn test_init_force_overwrites() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    let info = ProjectInfo {
        name: "test".to_string(),
        owner: None,
        has_git: false,
        has_agents_md: false,
        has_cagents_dir: false,
        has_claude_md: false,
        has_cursorrules: false,
        has_cursor_rules: false,
        agents_md_locations: vec![],
    };

    // First init
    init_basic(&info, false, false).unwrap();

    // Second init with force should succeed
    let info2 = ProjectInfo {
        has_cagents_dir: true,
        ..info
    };
    init_basic(&info2, true, false).unwrap();
}

#[test]
#[serial]
fn test_init_no_backup_without_flag() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create existing AGENTS.md
    fs::write("AGENTS.md", "# Existing").unwrap();

    let info = ProjectInfo {
        name: "test".to_string(),
        owner: None,
        has_git: false,
        has_agents_md: true,
        has_cagents_dir: false,
        has_claude_md: false,
        has_cursorrules: false,
        has_cursor_rules: false,
        agents_md_locations: vec![PathBuf::from("AGENTS.md")],
    };

    // Run migration without backup flag
    cagents_core::init::migrate_simple(&info, false, false).unwrap();

    // Verify no backup was created
    assert!(!PathBuf::from("AGENTS.md.backup").exists(),
        "Should NOT create backup without --backup flag");
}

#[test]
#[serial]
fn test_init_creates_backup_with_flag() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create existing AGENTS.md
    fs::write("AGENTS.md", "# Existing").unwrap();

    let info = ProjectInfo {
        name: "test".to_string(),
        owner: None,
        has_git: false,
        has_agents_md: true,
        has_cagents_dir: false,
        has_claude_md: false,
        has_cursorrules: false,
        has_cursor_rules: false,
        agents_md_locations: vec![PathBuf::from("AGENTS.md")],
    };

    // Run migration WITH backup flag
    cagents_core::init::migrate_simple(&info, false, true).unwrap();

    // Verify backup was created
    assert!(PathBuf::from("AGENTS.md.backup").exists(),
        "Should create backup WITH --backup flag");
}

/// Helper to change directory and restore on drop
struct ChangeDir {
    original: std::path::PathBuf,
}

impl ChangeDir {
    fn new(path: &std::path::Path) -> Self {
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();
        Self { original }
    }
}

impl Drop for ChangeDir {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}
