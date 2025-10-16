use cagents_core::import::import_cursorrules;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_import_cursorrules_recursively() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create nested structure with .cursorrules files
    fs::create_dir_all("backend").unwrap();
    fs::create_dir_all("frontend").unwrap();

    // Create root-level .cursorrules
    fs::write(
        ".cursorrules",
        "# General Rules\n\nAlways write tests."
    ).unwrap();

    // Create nested .cursorrules files
    fs::write(
        "backend/.cursorrules",
        "# API Rules\n\nUse REST conventions."
    ).unwrap();

    fs::write(
        "frontend/.cursorrules",
        "# React Rules\n\nUse functional components."
    ).unwrap();

    // Run import
    import_cursorrules(false).unwrap();

    // Verify all templates were created
    assert!(
        std::path::Path::new(".cAGENTS/templates/agents-cursor-root.md").exists(),
        "Root level .cursorrules should be imported as agents-cursor-root"
    );

    assert!(
        std::path::Path::new(".cAGENTS/templates/agents-cursor-backend.md").exists(),
        "Nested backend .cursorrules should be imported with directory name"
    );

    assert!(
        std::path::Path::new(".cAGENTS/templates/agents-cursor-frontend.md").exists(),
        "Nested frontend .cursorrules should be imported with directory name"
    );

    // Verify content is preserved
    let root_content = fs::read_to_string(".cAGENTS/templates/agents-cursor-root.md").unwrap();
    assert!(root_content.contains("Always write tests"));

    let api_content = fs::read_to_string(".cAGENTS/templates/agents-cursor-backend.md").unwrap();
    assert!(api_content.contains("Use REST conventions"));

    let react_content = fs::read_to_string(".cAGENTS/templates/agents-cursor-frontend.md").unwrap();
    assert!(react_content.contains("Use functional components"));

    // Verify nested files were removed
    assert!(!std::path::Path::new("backend/.cursorrules").exists(), "Nested backend .cursorrules should be removed");
    assert!(!std::path::Path::new("frontend/.cursorrules").exists(), "Nested frontend .cursorrules should be removed");

    // Root .cursorrules is removed and NOT regenerated (unlike AGENTS.md)
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
