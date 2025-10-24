// Test auto-update headers in rendered output files

use serial_test::serial;
use std::fs;
use tempfile::TempDir;
use std::env;

#[test]
#[serial]
fn test_agents_md_has_update_comment() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create minimal cAGENTS setup
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

    // Check AGENTS.md has the auto-generated header
    let agents_content = fs::read_to_string("AGENTS.md").unwrap();

    assert!(agents_content.contains("**IMPORTANT**: This project uses **cAGENTS**"),
        "Should contain cAGENTS notice");
    assert!(agents_content.contains("This file is auto-generated. Do not edit it directly."),
        "Should contain warning about editing");

    // Header should be at the top
    let first_lines = agents_content.lines().take(5).collect::<Vec<_>>().join("\n");
    assert!(first_lines.contains("**IMPORTANT**"),
        "Header should be at the top of the file");
}

#[test]
#[serial]
fn test_claude_md_has_update_comment() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create cAGENTS with claude-md output
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[output]
targets = ["claude-md"]
"#).unwrap();

    fs::write(".cAGENTS/templates/main.md", r#"---
name: main
---
# Claude Rules
"#).unwrap();

    // Run build
    cagents_core::cmd_build(None, false).unwrap();

    // Check CLAUDE.md has the auto-generated header
    let claude_content = fs::read_to_string("CLAUDE.md").unwrap();

    assert!(claude_content.contains("**IMPORTANT**: This project uses **cAGENTS**"),
        "CLAUDE.md should contain cAGENTS notice");
    assert!(claude_content.contains("This file is auto-generated. Do not edit it directly."),
        "CLAUDE.md should contain warning about editing");
}

#[test]
#[serial]
fn test_cursorrules_has_update_comment() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create cAGENTS with cursorrules output
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

    // Check .cursorrules has the auto-generated header
    let cursor_content = fs::read_to_string(".cursorrules").unwrap();

    assert!(cursor_content.contains("IMPORTANT: This project uses cAGENTS"),
        ".cursorrules should contain cAGENTS notice");
    assert!(cursor_content.contains("This file is auto-generated. Do not edit it directly."),
        ".cursorrules should contain warning about editing");
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
