// Test filtering rules by target (agents-md, claude-md, cursorrules)

use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use std::env;

#[test]
#[serial]
fn test_filter_rule_by_target() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create cAGENTS with multiple output targets
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

    // Create template that only applies to claude-md
    fs::write(".cAGENTS/templates/claude-specific.md", r#"---
name: claude-specific
alwaysApply: true
when:
  target: ["claude-md"]
---
# Claude-Specific Rules

This should only appear in CLAUDE.md
"#).unwrap();

    // Create template that applies to both
    fs::write(".cAGENTS/templates/common.md", r#"---
name: common
alwaysApply: true
---
# Common Rules

This should appear in both files
"#).unwrap();

    // Run build
    cagents_core::cmd_build(None, None, None, None, false).unwrap();

    // Check AGENTS.md does NOT have claude-specific content
    let agents = fs::read_to_string("AGENTS.md").unwrap();
    assert!(!agents.contains("Claude-Specific Rules"),
        "AGENTS.md should NOT contain claude-specific rules");
    assert!(agents.contains("Common Rules"),
        "AGENTS.md should contain common rules");

    // Check CLAUDE.md has both
    let claude = fs::read_to_string("CLAUDE.md").unwrap();
    assert!(claude.contains("Claude-Specific Rules"),
        "CLAUDE.md should contain claude-specific rules");
    assert!(claude.contains("Common Rules"),
        "CLAUDE.md should contain common rules");
}

#[test]
#[serial]
fn test_filter_rule_for_cursorrules_only() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create cAGENTS with all output targets
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

    // Create template that only applies to cursorrules
    fs::write(".cAGENTS/templates/cursor-only.md", r#"---
name: cursor-only
alwaysApply: true
when:
  target: ["cursorrules"]
---
# Cursor-Only Rules

Cursor IDE specific configuration
"#).unwrap();

    // Run build
    cagents_core::cmd_build(None, None, None, None, false).unwrap();

    // Check only .cursorrules has the content
    assert!(PathBuf::from(".cursorrules").exists(), ".cursorrules should be created");
    let cursorrules = fs::read_to_string(".cursorrules").unwrap();
    assert!(cursorrules.contains("Cursor-Only Rules"));

    // Other files should not contain it (or might not exist if no rules apply)
    if PathBuf::from("AGENTS.md").exists() {
        let agents = fs::read_to_string("AGENTS.md").unwrap();
        assert!(!agents.contains("Cursor-Only Rules"));
    }

    if PathBuf::from("CLAUDE.md").exists() {
        let claude = fs::read_to_string("CLAUDE.md").unwrap();
        assert!(!claude.contains("Cursor-Only Rules"));
    }
}

#[test]
#[serial]
fn test_filter_rule_for_multiple_targets() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create cAGENTS with all output targets
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

    // Create template that applies to agents-md and claude-md but not cursorrules
    fs::write(".cAGENTS/templates/markdown-only.md", r#"---
name: markdown-only
alwaysApply: true
when:
  target: ["agents-md", "claude-md"]
---
# Markdown Rules

Rich markdown formatting here
"#).unwrap();

    // Run build
    cagents_core::cmd_build(None, None, None, None, false).unwrap();

    // Check agents-md and claude-md have it
    assert!(PathBuf::from("AGENTS.md").exists());
    let agents = fs::read_to_string("AGENTS.md").unwrap();
    assert!(agents.contains("Markdown Rules"));

    assert!(PathBuf::from("CLAUDE.md").exists());
    let claude = fs::read_to_string("CLAUDE.md").unwrap();
    assert!(claude.contains("Markdown Rules"));

    // cursorrules should not have the markdown-only content (or might not exist)
    if PathBuf::from(".cursorrules").exists() {
        let cursorrules = fs::read_to_string(".cursorrules").unwrap();
        assert!(!cursorrules.contains("Markdown Rules"));
    }
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
