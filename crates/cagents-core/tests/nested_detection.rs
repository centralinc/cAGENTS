// Test nested AGENTS.md detection and migration
use serial_test::serial;

use cagents_core::init::{ProjectInfo, migrate_smart};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use std::env;

#[test]
#[serial]
fn test_detects_multiple_agents_md_files() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create nested AGENTS.md files
    fs::create_dir_all("src").unwrap();
    fs::create_dir_all("packages/app1").unwrap();
    fs::write("AGENTS.md", "Root rules").unwrap();
    fs::write("src/AGENTS.md", "Src rules").unwrap();
    fs::write("packages/app1/AGENTS.md", "App rules").unwrap();

    let info = ProjectInfo::detect().unwrap();

    // Should detect all 3 files
    assert_eq!(info.agents_md_locations.len(), 3);
    assert!(info.agents_md_locations.contains(&PathBuf::from("AGENTS.md")));
    assert!(info.agents_md_locations.contains(&PathBuf::from("src/AGENTS.md")));
    assert!(info.agents_md_locations.contains(&PathBuf::from("packages/app1/AGENTS.md")));
}

#[test]
#[serial]
fn test_migrate_multiple_creates_templates_with_globs() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create nested AGENTS.md files
    fs::create_dir_all("src").unwrap();
    fs::create_dir_all("tests").unwrap();
    fs::write("AGENTS.md", "# Root\nRoot content").unwrap();
    fs::write("src/AGENTS.md", "# Src\nSrc content").unwrap();
    fs::write("tests/AGENTS.md", "# Tests\nTests content").unwrap();

    // Add git for has_git check
    fs::create_dir(".git").unwrap();

    let info = ProjectInfo {
        name: "test".to_string(),
        owner: None,
        has_git: true,
        has_agents_md: true,
        has_cagents_dir: false,
        has_claude_md: false,
        has_cursorrules: false,
        has_cursor_rules: false,
        agents_md_locations: vec![
            PathBuf::from("AGENTS.md"),
            PathBuf::from("src/AGENTS.md"),
            PathBuf::from("tests/AGENTS.md"),
        ],
    };

    // Skip backup prompt with has_git=false to avoid interactive prompt
    let info_no_git = ProjectInfo { has_git: false, ..info };

    migrate_smart(&info_no_git, false, false).unwrap();

    // Should create 3 templates (new naming)
    assert!(PathBuf::from(".cAGENTS/templates/agents-root.md").exists());
    assert!(PathBuf::from(".cAGENTS/templates/agents-src.md").exists());
    assert!(PathBuf::from(".cAGENTS/templates/agents-tests.md").exists());

    // Check root template was created (no when clause = implicitly always applies)
    let root = fs::read_to_string(".cAGENTS/templates/agents-root.md").unwrap();
    assert!(!root.contains("alwaysApply"), "alwaysApply field has been removed");

    // Migrated templates don't have globs or when clauses to avoid nested outputs
    let src = fs::read_to_string(".cAGENTS/templates/agents-src.md").unwrap();
    assert!(!src.contains("alwaysApply"), "alwaysApply field has been removed");
    assert!(!src.contains("globs:"), "Should not have globs - no when clause means always apply");
    assert!(!src.contains("when:"), "Should not have when clause - implicitly always apply");

    // Check templates preserve content
    assert!(root.contains("Root content"));
    assert!(src.contains("Src content"));
}

#[test]
#[serial]
fn test_migrate_respects_order() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all("a").unwrap();
    fs::create_dir_all("b").unwrap();
    fs::write("AGENTS.md", "Root").unwrap();
    fs::write("a/AGENTS.md", "A").unwrap();
    fs::write("b/AGENTS.md", "B").unwrap();

    let info = ProjectInfo {
        name: "test".to_string(),
        owner: None,
        has_git: false,
        has_agents_md: true,
        has_cagents_dir: false,
        has_claude_md: false,
        has_cursorrules: false,
        has_cursor_rules: false,
        agents_md_locations: vec![
            PathBuf::from("AGENTS.md"),
            PathBuf::from("a/AGENTS.md"),
            PathBuf::from("b/AGENTS.md"),
        ],
    };

    migrate_smart(&info, false, false).unwrap();

    // Check order values (new naming)
    let root = fs::read_to_string(".cAGENTS/templates/agents-root.md").unwrap();
    let a = fs::read_to_string(".cAGENTS/templates/agents-a.md").unwrap();
    let b = fs::read_to_string(".cAGENTS/templates/agents-b.md").unwrap();

    assert!(root.contains("order: 10"));
    assert!(a.contains("order: 20"));
    assert!(b.contains("order: 30"));
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
