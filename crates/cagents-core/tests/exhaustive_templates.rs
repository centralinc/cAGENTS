use anyhow::Result;
use serial_test::serial;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use cagents_core::cmd_build;
use std::fs;

fn write_renderer(temp: &TempDir, name: &str, python_body: &str) -> String {
    let script = temp.child(name);
    script.write_str(python_body).unwrap();
    // Quote path and use forward slashes for cross-platform compatibility
    let path = script.path().display().to_string().replace('\\', "/");
    format!("python3 \"{}\"", path)
}

#[test]
#[serial]
fn test_multiple_templates_use_distinct_commands() -> Result<()> {
    let temp = TempDir::new()?;

    // Uppercase renderer
    let upper_command = write_renderer(
        &temp,
        "upper.py",
        r#"import json, sys
payload = json.load(sys.stdin)
content = payload.get("templateSource", "").upper()
print(json.dumps({"content": content}))
"#,
    );

    // Lowercase renderer
    let lower_command = write_renderer(
        &temp,
        "lower.py",
        r#"import json, sys
payload = json.load(sys.stdin)
content = payload.get("templateSource", "").lower()
print(json.dumps({"content": content}))
"#,
    );

    // Config
    temp.child(".cAGENTS/config.toml").write_str(
        r#"[paths]
templatesDir = "templates"
outputRoot = "."
"#,
    )?;

    // Templates - use single quotes in YAML to avoid escaping double quotes
    temp.child(".cAGENTS/templates/upper.md").write_str(&format!(
        "---\nname: upper\nengine: 'command:{}'\norder: 1\n---\nMixedCase\n",
        upper_command
    ))?;

    temp.child(".cAGENTS/templates/lower.md").write_str(&format!(
        "---\nname: lower\nengine: 'command:{}'\norder: 2\n---\nMixedCase\n",
        lower_command
    ))?;

    let original = std::env::current_dir()?;
    std::env::set_current_dir(temp.path())?;
    let result = cmd_build(None, false);
    std::env::set_current_dir(original)?;

    result?;

    let output = fs::read_to_string(temp.child("AGENTS.md").path())?;
    assert!(output.contains("MIXEDCASE"), "expected uppercase renderer output");
    assert!(output.contains("mixedcase"), "expected lowercase renderer output");

    Ok(())
}

#[test]
#[serial]
fn test_defaults_engine_applied_when_missing() -> Result<()> {
    let temp = TempDir::new()?;

    let command = write_renderer(
        &temp,
        "default.py",
        r#"import json, sys
payload = json.load(sys.stdin)
data = payload.get("data", {})
content = payload.get("templateSource", "")
for key, value in data.items():
    content = content.replace("{{" + key + "}}", str(value))
print(json.dumps({"content": content}))
"#,
    );

    // Use triple-quoted TOML string to avoid escaping quotes
    temp.child(".cAGENTS/config.toml").write_str(&format!(
        "[paths]\ntemplatesDir = \"templates\"\noutputRoot = \".\"\n\n[defaults]\nengine = \"\"\"command:{}\"\"\"\n\n[variables.static]\nproject = \"Defaults\"\n",
        command
    ))?;

    temp.child(".cAGENTS/templates/rule.md").write_str(
        "---\nname: defaulted\norder: 1\n---\nProject: {{project}}\n",
    )?;

    let original = std::env::current_dir()?;
    std::env::set_current_dir(temp.path())?;
    let result = cmd_build(None, false);
    std::env::set_current_dir(original)?;

    result?;

    let output = fs::read_to_string(temp.child("AGENTS.md").path())?;
    assert!(output.contains("Project: Defaults"));

    Ok(())
}

#[test]
#[serial]
fn test_frontmatter_vars_merge_into_data() -> Result<()> {
    let temp = TempDir::new()?;

    let command = write_renderer(
        &temp,
        "vars.py",
        r#"import json, sys
payload = json.load(sys.stdin)
content = payload.get("templateSource", "")
data = payload.get("data", {})
for key, value in data.items():
    content = content.replace("{{" + key + "}}", str(value))
print(json.dumps({"content": content}))
"#,
    );

    // Use triple-quoted TOML string to avoid escaping quotes
    temp.child(".cAGENTS/config.toml").write_str(&format!(
        "[paths]\ntemplatesDir = \"templates\"\noutputRoot = \".\"\n\n[defaults]\nengine = \"\"\"command:{}\"\"\"\n",
        command
    ))?;

    temp.child(".cAGENTS/templates/rule.md").write_str(
        "---\nname: vars\norder: 1\nvars:\n  audience: frontend\n---\nAudience: {{audience}}\n",
    )?;

    let original = std::env::current_dir()?;
    std::env::set_current_dir(temp.path())?;
    let result = cmd_build(None, false);
    std::env::set_current_dir(original)?;

    result?;

    let output = fs::read_to_string(temp.child("AGENTS.md").path())?;
    assert!(output.contains("Audience: frontend"));

    Ok(())
}

#[test]
#[serial]
fn test_failing_command_surfaces_error() -> Result<()> {
    let temp = TempDir::new()?;

    temp.child(".cAGENTS/config.toml").write_str(
        r#"[paths]
templatesDir = "templates"
outputRoot = "."
"#,
    )?;

    temp.child(".cAGENTS/templates/broken.md").write_str(
        "---\nname: broken\nengine: \"command:false\"\n---\nThis will fail\n",
    )?;

    let original = std::env::current_dir()?;
    std::env::set_current_dir(temp.path())?;
    let result = cmd_build(None, false);
    std::env::set_current_dir(original)?;

    assert!(result.is_err(), "expected command failure to propagate");

    Ok(())
}
