use assert_fs::prelude::*;
use assert_fs::TempDir;
use cagents_core::import::{ImportFormat, import_multiple_formats};
use serial_test::serial;
use std::env;

#[test]
#[serial]
fn test_import_multiple_formats_finds_nested_files() {
    // Create test directory structure with nested AGENTS.md and CLAUDE.md files
    let temp = TempDir::new().unwrap();

    // Root level files
    temp.child("AGENTS.md").write_str("# Root AGENTS\nRoot level agents").unwrap();
    temp.child("CLAUDE.md").write_str("# Root CLAUDE\nRoot level claude").unwrap();

    // Nested files
    temp.child("src/api/AGENTS.md").write_str("# API AGENTS\nAPI level agents").unwrap();
    temp.child("src/api/CLAUDE.md").write_str("# API CLAUDE\nAPI level claude").unwrap();
    temp.child("src/client/AGENTS.md").write_str("# Client AGENTS\nClient level agents").unwrap();
    temp.child("src/client/CLAUDE.md").write_str("# Client CLAUDE\nClient level claude").unwrap();

    // Change to temp directory
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp.path()).unwrap();

    // Test the merge functionality
    let formats = vec![
        ImportFormat::AgentsMd,
        ImportFormat::ClaudeMd,
    ];

    // This should find and merge ALL nested files, not just root-level
    import_multiple_formats(&formats, false).unwrap();

    // Restore original directory
    env::set_current_dir(&original_dir).unwrap();

    // Verify .cAGENTS structure was created
    assert!(temp.child(".cAGENTS/config.toml").exists());
    assert!(temp.child(".cAGENTS/templates").exists());

    // Read the generated templates
    let agents_template = std::fs::read_to_string(
        temp.child(".cAGENTS/templates/agents-from-agentsmd.md").path()
    ).unwrap();

    let claude_template = std::fs::read_to_string(
        temp.child(".cAGENTS/templates/agents-from-claudemd.md").path()
    ).unwrap();

    // BUG: Currently these only contain root-level content
    // After fix, they should contain ALL nested content

    // Verify AGENTS template contains content from all AGENTS.md files
    assert!(
        agents_template.contains("Root level agents"),
        "Should contain root AGENTS.md content"
    );
    assert!(
        agents_template.contains("API level agents"),
        "Should contain nested src/api/AGENTS.md content"
    );
    assert!(
        agents_template.contains("Client level agents"),
        "Should contain nested src/client/AGENTS.md content"
    );

    // Verify CLAUDE template contains content from all CLAUDE.md files
    assert!(
        claude_template.contains("Root level claude"),
        "Should contain root CLAUDE.md content"
    );
    assert!(
        claude_template.contains("API level claude"),
        "Should contain nested src/api/CLAUDE.md content"
    );
    assert!(
        claude_template.contains("Client level claude"),
        "Should contain nested src/client/CLAUDE.md content"
    );
}

#[test]
#[serial]
fn test_import_multiple_formats_respects_gitignore() {
    let temp = TempDir::new().unwrap();

    // Create files that should be found
    temp.child("AGENTS.md").write_str("# Root AGENTS").unwrap();
    temp.child("src/AGENTS.md").write_str("# Src AGENTS").unwrap();

    // Create files that should be ignored
    temp.child("node_modules/AGENTS.md").write_str("# Node AGENTS").unwrap();
    temp.child(".git/AGENTS.md").write_str("# Git AGENTS").unwrap();
    temp.child("target/AGENTS.md").write_str("# Target AGENTS").unwrap();

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp.path()).unwrap();

    let formats = vec![ImportFormat::AgentsMd];
    import_multiple_formats(&formats, false).unwrap();

    env::set_current_dir(&original_dir).unwrap();

    let template = std::fs::read_to_string(
        temp.child(".cAGENTS/templates/agents-from-agentsmd.md").path()
    ).unwrap();

    // Should contain non-ignored files
    assert!(template.contains("Root AGENTS"));
    assert!(template.contains("Src AGENTS"));

    // Should NOT contain ignored files
    assert!(!template.contains("Node AGENTS"));
    assert!(!template.contains("Git AGENTS"));
    assert!(!template.contains("Target AGENTS"));
}
