use assert_fs::prelude::*;
use assert_fs::TempDir;
use cagents_core::adapters::command::render_external;
use serde_json::json;

#[test]
fn test_render_external_receives_template_path() {
    let temp = TempDir::new().unwrap();
    let script = temp.child("render.py");
    script
        .write_str(
            r#"import json, sys
payload = json.load(sys.stdin)
path = payload.get("templatePath")
print(json.dumps({"content": f"path={path}"}))
"#,
        )
        .unwrap();

    // Quote path and use forward slashes for cross-platform compatibility
    let path = script.path().display().to_string().replace('\\', "/");
    let command = format!("python3 \"{}\"", path);
    let output = render_external(&command, "template", &json!({}), &json!({}), "templates/rule.md").unwrap();
    assert_eq!(output, "path=templates/rule.md");
}

#[test]
fn test_render_external_invalid_json_is_error() {
    let temp = TempDir::new().unwrap();
    let script = temp.child("render.sh");
    script
        .write_str("printf 'not-json'\n")
        .unwrap();

    // Quote path and use forward slashes for cross-platform compatibility
    let path = script.path().display().to_string().replace('\\', "/");
    let command = format!("sh \"{}\"", path);
    let err = render_external(&command, "", &json!({}), &json!({}), "rule.md").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Failed to parse compiler output") || msg.contains("External compiler failed"));
}
