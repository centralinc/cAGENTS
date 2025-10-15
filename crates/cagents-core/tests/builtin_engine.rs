use anyhow::Result;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use cagents_core::cmd_build;
use serial_test::serial;
use std::fs;

#[test]
#[serial]
fn test_builtin_simple_engine_renders() -> Result<()> {
    let temp = TempDir::new()?;

    // Write config with builtin:simple engine
    let config = temp.child(".cAGENTS/config.toml");
    config.write_str(
        r#"[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[variables.static]
project = "TestProject"
owner = "Alice"
"#,
    )?;

    // Write template using {{variable}} syntax
    let template = temp.child(".cAGENTS/templates/agents-root.md");
    template.write_str(
        r#"---
name: test-template
---
# {{project}} - Rules

Owner: {{owner}}

This is a test project.
"#,
    )?;

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp.path())?;

    // Run build
    let result = cmd_build(None, None, None, None, false);

    // Restore directory
    std::env::set_current_dir(&original_dir)?;

    result?;

    // Verify AGENTS.md was created with interpolated values
    let agents_md = fs::read_to_string(temp.child("AGENTS.md").path())?;
    assert!(agents_md.contains("# TestProject - Rules"));
    assert!(agents_md.contains("Owner: Alice"));
    assert!(agents_md.contains("This is a test project"));

    // Should NOT contain template syntax
    assert!(!agents_md.contains("{{project}}"));
    assert!(!agents_md.contains("{{owner}}"));

    Ok(())
}

#[test]
#[serial]
fn test_builtin_engine_fails_on_undefined_variable() -> Result<()> {
    let temp = TempDir::new()?;

    // Write config
    let config = temp.child(".cAGENTS/config.toml");
    config.write_str(
        r#"[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[variables.static]
project = "TestProject"
"#,
    )?;

    // Write template with UNDEFINED variable
    let template = temp.child(".cAGENTS/templates/agents-root.md");
    template.write_str(
        r#"---
name: test
---
# {{project}}
Owner: {{undefined_var}}
"#,
    )?;

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp.path())?;

    // Run build - should FAIL
    let result = cmd_build(None, None, None, None, false);

    // Restore directory
    std::env::set_current_dir(&original_dir)?;

    // Verify it failed
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{:#}", err); // Full error chain

    // Should contain error about builtin engine failure
    assert!(err_msg.contains("Builtin engine failed"),
        "Error should mention builtin engine. Got: {}", err_msg);

    Ok(())
}

#[test]
#[serial]
fn test_builtin_engine_with_numbers_and_booleans() -> Result<()> {
    let temp = TempDir::new()?;

    // Write config
    let config = temp.child(".cAGENTS/config.toml");
    config.write_str(
        r#"[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[variables.static]
project = "Test"
version = 42
active = true
"#,
    )?;

    // Write template
    let template = temp.child(".cAGENTS/templates/agents-root.md");
    template.write_str(
        r#"---
name: test
---
Project: {{project}}
Version: {{version}}
Active: {{active}}
"#,
    )?;

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp.path())?;

    // Run build
    let result = cmd_build(None, None, None, None, false);

    // Restore directory
    std::env::set_current_dir(&original_dir)?;

    result?;

    // Verify output
    let agents_md = fs::read_to_string(temp.child("AGENTS.md").path())?;
    assert!(agents_md.contains("Project: Test"));
    assert!(agents_md.contains("Version: 42"));
    assert!(agents_md.contains("Active: true"));

    Ok(())
}
