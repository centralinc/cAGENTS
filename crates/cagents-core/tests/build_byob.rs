use anyhow::Result;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use cagents_core::cmd_build;
use serial_test::serial;
use std::fs;

fn write_renderer_script(dir: &assert_fs::TempDir) -> String {
    let script = dir.child("render.py");
    script
        .write_str(
            r#"import json, sys
payload = json.load(sys.stdin)
template = payload.get("templateSource", "")
data = payload.get("data", {})
for key, value in data.items():
    template = template.replace("{{" + key + "}}", str(value) if not isinstance(value, bool) else ("true" if value else "false"))
print(json.dumps({"content": template}))
"#,
        )

        .unwrap();
    // Quote path and use forward slashes for cross-platform compatibility
    let path = script.path().display().to_string().replace('\\', "/");
    format!("python3 \"{}\"", path)
}

#[test]
#[serial]
fn build_with_command_engine_renders_content() -> Result<()> {
    let temp = TempDir::new()?;

    // Write config
    let config = temp.child(".cAGENTS/config.toml");
    config.write_str(
        r#"[paths]
templatesDir = "templates"
outputRoot = "."

[variables.static]
project = "Demo"
owner = "Jordan"
"#,
    )?;

    // Write renderer
    let command = write_renderer_script(&temp);

    // Write template
    let template = temp.child(".cAGENTS/templates/project.md");
    // Use single quotes in YAML to avoid escaping double quotes
    template.write_str(&format!(
        "---\nname: project\nengine: 'command:{}'\norder: 1\n---\n# {{{{project}}}}\nOwner: {{{{owner}}}}\n",
        command
    ))?;

    // Change directory
    let original = std::env::current_dir()?;
    std::env::set_current_dir(temp.path())?;

    let result = cmd_build(None, None, None, None, false);
    std::env::set_current_dir(original)?;

    result?;

    let content = fs::read_to_string(temp.child("AGENTS.md").path())?;
    assert!(content.contains("# Demo"));
    assert!(content.contains("Owner: Jordan"));

    Ok(())
}

#[test]
#[serial]
fn build_without_command_engine_errors() -> Result<()> {
    let temp = TempDir::new()?;

    let config = temp.child(".cAGENTS/config.toml");
    config.write_str(
        r#"[paths]
templatesDir = "templates"
outputRoot = "."
"#,
    )?;

    let template = temp.child(".cAGENTS/templates/rule.hbs.md");
    template.write_str(
        "---\nname: legacy\nengine: \"handlebars\"\n---\n# Legacy\n",
    )?;

    let original = std::env::current_dir()?;
    std::env::set_current_dir(temp.path())?;
    let result = cmd_build(None, None, None, None, false);
    std::env::set_current_dir(original)?;

    assert!(result.is_err(), "expected error when using built-in engine");

    Ok(())
}
