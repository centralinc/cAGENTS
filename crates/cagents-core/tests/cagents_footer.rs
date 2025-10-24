// Test that cAGENTS footer is appended to all output targets
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_agents_md_has_cagents_footer() {
    let tmp = TempDir::new().unwrap();
    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    // Setup minimal config
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(
        ".cAGENTS/config.toml",
        r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"
"#,
    )
    .unwrap();

    // Create simple template
    fs::write(
        ".cAGENTS/templates/test.md",
        r#"---
name: test
---
# Test Content
"#,
    )
    .unwrap();

    // Build
    cagents_core::cmd_build(None, false).unwrap();

    // Read generated AGENTS.md
    let content = fs::read_to_string("AGENTS.md").unwrap();

    std::env::set_current_dir(original).unwrap();

    // Verify header exists (footer removed)
    assert!(content.contains("**IMPORTANT**: This project uses **cAGENTS**"), "Should have cAGENTS header");
    assert!(content.contains("This file is auto-generated. Do not edit it directly."), "Should have warning");
}

#[test]
#[serial]
fn test_claude_md_has_cagents_footer() {
    let tmp = TempDir::new().unwrap();
    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    // Setup minimal config with claude-md target
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(
        ".cAGENTS/config.toml",
        r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[output]
targets = ["claude-md"]
"#,
    )
    .unwrap();

    // Create simple template
    fs::write(
        ".cAGENTS/templates/test.md",
        r#"---
name: test
---
# Test Content
"#,
    )
    .unwrap();

    // Build
    cagents_core::cmd_build(None, false).unwrap();

    // Read generated CLAUDE.md
    let content = fs::read_to_string("CLAUDE.md").unwrap();

    std::env::set_current_dir(original).unwrap();

    // Verify header exists (footer removed)
    assert!(content.contains("**IMPORTANT**: This project uses **cAGENTS**"), "Should have cAGENTS header");
    assert!(content.contains("This file is auto-generated. Do not edit it directly."), "Should have warning");
}
