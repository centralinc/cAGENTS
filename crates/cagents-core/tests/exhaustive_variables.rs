// Exhaustive test suite for variable handling
use serial_test::serial;

use serde_json::{json, Value};
use tempfile::TempDir;
use std::env;
use std::fs;

const RENDER_SCRIPT: &str = r#"
import json
import sys


def format_value(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if value is None:
        return ""
    if isinstance(value, (int, float)):
        return str(value)
    if isinstance(value, (list, dict)):
        return json.dumps(value, ensure_ascii=False)
    return str(value)


def render(template, data):
    result = template
    if isinstance(data, dict):
        for key, value in data.items():
            placeholder = "{{" + key + "}}"
            result = result.replace(placeholder, format_value(value))
    return result


def main():
    payload = json.load(sys.stdin)
    template = payload.get("templateSource", "")
    data = payload.get("data", {})
    content = render(template, data)
    print(json.dumps({"content": content}, ensure_ascii=False))


if __name__ == "__main__":
    main()
"#;

fn render_with_byob(template: &str, data: Value) -> String {
    let tmp = TempDir::new().expect("temp dir");
    let script_path = tmp.path().join("render.py");
    fs::write(&script_path, RENDER_SCRIPT).expect("write renderer");
    let command = format!("python3 {}", script_path.display());
    cagents_core::adapters::command::render_external(
        &command,
        template,
        &data,
        &json!({}),
        "inline",
    )
    .expect("external renderer failed")
}

// Test variable types and scenarios

#[test]
#[serial]
fn test_static_variables_basic() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create config with static variables
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[variables.static]
name = "Alice"
age = 30
active = true
"#).unwrap();

    let config = cagents_core::config::load_config_with_precedence().unwrap();
    let vars = config.variables.unwrap();
    let static_vars = vars.static_.unwrap();

    assert_eq!(static_vars["name"], "Alice");
    assert_eq!(static_vars["age"], 30);
    assert_eq!(static_vars["active"], true);
}

#[test]
#[serial]
fn test_static_variables_complex_types() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[variables.static]
simple = "string"
number = 42
float = 3.14
bool = true
array = ["a", "b", "c"]

[variables.static.nested]
key1 = "value1"
key2 = "value2"
"#).unwrap();

    let config = cagents_core::config::load_config_with_precedence().unwrap();
    let vars = config.variables.unwrap();
    let static_vars = vars.static_.unwrap();

    assert_eq!(static_vars["simple"], "string");
    assert_eq!(static_vars["number"], 42);
    assert!(static_vars["array"].is_array());
    assert!(static_vars["nested"].is_object());
}

#[test]
#[serial]
fn test_static_variables_special_characters() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[variables.static]
special = "Hello & <world> \"quoted\""
emoji = "🎉 party"
multiline = """
Line 1
Line 2
Line 3"""
"#).unwrap();

    let config = cagents_core::config::load_config_with_precedence().unwrap();
    let vars = config.variables.unwrap();
    let static_vars = vars.static_.unwrap();

    assert!(static_vars["special"].as_str().unwrap().contains("&"));
    assert!(static_vars["emoji"].as_str().unwrap().contains("🎉"));
    assert!(static_vars["multiline"].as_str().unwrap().contains("Line 1"));
}

#[test]
#[serial]
fn test_variable_substitution_in_template() {
    let template = r#"# {{project}}

Owner: {{owner}}
Version: {{version}}
Active: {{active}}
"#;

    let data = json!({
        "project": "TestApp",
        "owner": "Alice",
        "version": "1.0.0",
        "active": true
    });

    let result = render_with_byob(template, data);
    assert!(result.contains("# TestApp"));
    assert!(result.contains("Owner: Alice"));
    assert!(result.contains("Version: 1.0.0"));
    assert!(result.contains("Active: true"));
}

#[test]
#[serial]
fn test_variable_in_markdown_headings() {
    let template = r#"# {{level1}}
## {{level2}}
### {{level3}}
"#;
    let data = json!({
        "level1": "Title",
        "level2": "Subtitle",
        "level3": "Section"
    });

    let result = render_with_byob(template, data);
    assert!(result.contains("# Title"));
    assert!(result.contains("## Subtitle"));
    assert!(result.contains("### Section"));
}

#[test]
#[serial]
fn test_variable_in_code_blocks() {
    let template = r#"```bash
{{command}}
```"#;
    let data = json!({"command": "npm install"});

    let result = render_with_byob(template, data);
    assert!(result.contains("npm install"));
}

#[test]
#[serial]
fn test_variable_in_lists() {
    let template = r#"- {{item1}}
- {{item2}}
- {{item3}}"#;
    let data = json!({"item1": "First", "item2": "Second", "item3": "Third"});

    let result = render_with_byob(template, data);
    assert!(result.contains("- First"));
    assert!(result.contains("- Second"));
    assert!(result.contains("- Third"));
}

#[test]
#[serial]
fn test_variable_in_links() {
    let template = "[{{text}}]({{url}})";
    let data = json!({"text": "Click here", "url": "https://example.com"});

    let result = render_with_byob(template, data);
    assert_eq!(result, "[Click here](https://example.com)");
}

#[test]
#[serial]
fn test_empty_string_variable() {
    let template = "Value: '{{empty}}'";
    let data = json!({"empty": ""});

    let result = render_with_byob(template, data);
    assert_eq!(result, "Value: ''");
}

#[test]
#[serial]
fn test_null_variable() {
    let template = "Value: {{nothing}}";
    let data = json!({"nothing": null});

    let result = render_with_byob(template, data);
    // Handlebars renders null as empty
    assert_eq!(result, "Value: ");
}

#[test]
#[serial]
fn test_number_as_string() {
    let template = "Port: {{port}}";
    let data = json!({"port": 8080});

    let result = render_with_byob(template, data);
    assert_eq!(result, "Port: 8080");
}

#[test]
#[serial]
fn test_boolean_as_string() {
    let template = "Enabled: {{enabled}}";
    let data = json!({"enabled": true});

    let result = render_with_byob(template, data);
    assert_eq!(result, "Enabled: true");
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
