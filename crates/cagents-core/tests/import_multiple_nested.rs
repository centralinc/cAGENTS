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

    // Verify separate templates were created for each nested file

    // Check AGENTS.md templates
    let agents_root = std::fs::read_to_string(
        temp.child(".cAGENTS/templates/agents-root.md").path()
    ).unwrap();
    assert!(agents_root.contains("Root level agents"));
    assert!(!agents_root.contains("globs:"), "Root template should not have globs");

    let agents_api = std::fs::read_to_string(
        temp.child(".cAGENTS/templates/agents-api.md").path()
    ).unwrap();
    assert!(agents_api.contains("API level agents"));
    assert!(agents_api.contains("globs: [\"src/api/\"]"), "Nested template should have directory glob");
    assert!(agents_api.contains("outputIn: \"matched\""), "Nested template should have outputIn: matched");

    let agents_client = std::fs::read_to_string(
        temp.child(".cAGENTS/templates/agents-client.md").path()
    ).unwrap();
    assert!(agents_client.contains("Client level agents"));
    assert!(agents_client.contains("globs: [\"src/client/\"]"));
    assert!(agents_client.contains("outputIn: \"matched\""));

    // Check CLAUDE.md templates
    let claude_root = std::fs::read_to_string(
        temp.child(".cAGENTS/templates/claude-root.md").path()
    ).unwrap();
    assert!(claude_root.contains("Root level claude"));
    assert!(!claude_root.contains("globs:"));

    let claude_api = std::fs::read_to_string(
        temp.child(".cAGENTS/templates/claude-api.md").path()
    ).unwrap();
    assert!(claude_api.contains("API level claude"));
    assert!(claude_api.contains("globs: [\"src/api/\"]"));
    assert!(claude_api.contains("outputIn: \"matched\""));

    let claude_client = std::fs::read_to_string(
        temp.child(".cAGENTS/templates/claude-client.md").path()
    ).unwrap();
    assert!(claude_client.contains("Client level claude"));
    assert!(claude_client.contains("globs: [\"src/client/\"]"));
    assert!(claude_client.contains("outputIn: \"matched\""));
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

    // Check that separate templates were created for non-ignored files
    let agents_root = std::fs::read_to_string(
        temp.child(".cAGENTS/templates/agents-root.md").path()
    ).unwrap();
    assert!(agents_root.contains("Root AGENTS"));

    let agents_src = std::fs::read_to_string(
        temp.child(".cAGENTS/templates/agents-src.md").path()
    ).unwrap();
    assert!(agents_src.contains("Src AGENTS"));

    // Verify ignored files were NOT imported as templates
    assert!(!temp.child(".cAGENTS/templates/agents-node_modules.md").exists());
    assert!(!temp.child(".cAGENTS/templates/agents-target.md").exists());
}
