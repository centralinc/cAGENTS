use cagents_core::import::import_claude_md;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_import_claude_md_recursively() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create nested structure with CLAUDE.md files
    fs::create_dir_all("backend").unwrap();
    fs::create_dir_all("frontend").unwrap();

    // Create root-level CLAUDE.md
    fs::write(
        "CLAUDE.md",
        "# General Rules\n\nAlways write tests."
    ).unwrap();

    // Create nested CLAUDE.md files
    fs::write(
        "backend/CLAUDE.md",
        "# API Rules\n\nUse REST conventions."
    ).unwrap();

    fs::write(
        "frontend/CLAUDE.md",
        "# React Rules\n\nUse functional components."
    ).unwrap();

    // Run import
    import_claude_md(false).unwrap();

    // Verify all templates were created
    assert!(
        std::path::Path::new(".cAGENTS/templates/agents-root.md").exists(),
        "Root level CLAUDE.md should be imported as agents-root"
    );

    assert!(
        std::path::Path::new(".cAGENTS/templates/agents-backend.md").exists(),
        "Nested backend CLAUDE.md should be imported with directory name"
    );

    assert!(
        std::path::Path::new(".cAGENTS/templates/agents-frontend.md").exists(),
        "Nested frontend CLAUDE.md should be imported with directory name"
    );

    // Verify content is preserved
    let root_content = fs::read_to_string(".cAGENTS/templates/agents-root.md").unwrap();
    assert!(root_content.contains("Always write tests"));

    let api_content = fs::read_to_string(".cAGENTS/templates/agents-backend.md").unwrap();
    assert!(api_content.contains("Use REST conventions"));

    let react_content = fs::read_to_string(".cAGENTS/templates/agents-frontend.md").unwrap();
    assert!(react_content.contains("Use functional components"));

    // Verify nested files were removed (root might get regenerated)
    assert!(!std::path::Path::new("backend/CLAUDE.md").exists(), "Nested backend CLAUDE.md should be removed");
    assert!(!std::path::Path::new("frontend/CLAUDE.md").exists(), "Nested frontend CLAUDE.md should be removed");

    // Note: Root CLAUDE.md is removed and NOT regenerated (unlike AGENTS.md)
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
