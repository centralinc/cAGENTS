use cagents_core::import::import_cursor_rules;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_import_cursor_rules_recursively() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create nested .cursor/rules structure
    fs::create_dir_all(".cursor/rules/backend").unwrap();
    fs::create_dir_all(".cursor/rules/frontend").unwrap();

    // Create root-level rule
    fs::write(
        ".cursor/rules/general.md",
        "# General Rules\n\nAlways write tests."
    ).unwrap();

    // Create nested rules
    fs::write(
        ".cursor/rules/backend/api.md",
        "# API Rules\n\nUse REST conventions."
    ).unwrap();

    fs::write(
        ".cursor/rules/frontend/react.md",
        "# React Rules\n\nUse functional components."
    ).unwrap();

    // Run import
    import_cursor_rules(false).unwrap();

    // Verify all templates were created
    assert!(
        std::path::Path::new(".cAGENTS/templates/agents-general.md").exists(),
        "Root level rule should be imported"
    );

    assert!(
        std::path::Path::new(".cAGENTS/templates/agents-backend-api.md").exists(),
        "Nested backend rule should be imported with flattened name"
    );

    assert!(
        std::path::Path::new(".cAGENTS/templates/agents-frontend-react.md").exists(),
        "Nested frontend rule should be imported with flattened name"
    );

    // Verify content is preserved
    let general_content = fs::read_to_string(".cAGENTS/templates/agents-general.md").unwrap();
    assert!(general_content.contains("Always write tests"));

    let api_content = fs::read_to_string(".cAGENTS/templates/agents-backend-api.md").unwrap();
    assert!(api_content.contains("Use REST conventions"));

    let react_content = fs::read_to_string(".cAGENTS/templates/agents-frontend-react.md").unwrap();
    assert!(react_content.contains("Use functional components"));
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
