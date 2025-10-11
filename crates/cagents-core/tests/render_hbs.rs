use assert_fs::prelude::*;
use assert_fs::TempDir;
use cagents_core::adapters::command::render_external;
use serde_json::json;

#[test]
fn test_render_external_receives_template_and_data() {
    let temp = TempDir::new().unwrap();
    let script = temp.child("render.py");
    script
        .write_str(
            r#"import json, sys
payload = json.load(sys.stdin)
template = payload.get("templateSource", "")
data = payload.get("data", {})
for key, value in data.items():
    template = template.replace("{{" + key + "}}", str(value))
print(json.dumps({"content": template}))
"#,
        )
        .unwrap();

    let command = format!("python3 {}", script.path().display());
    let data = json!({"owner": "Jordan", "tone": "concise"});
    let frontmatter = json!({});
    let template = "Hello {{owner}}, tone is {{tone}}!";

    let result = render_external(&command, template, &data, &frontmatter, "rules.md").unwrap();
    assert!(result.contains("Hello Jordan"));
    assert!(result.contains("tone is concise"));
}

#[test]
fn test_render_external_receives_frontmatter() {
    let temp = TempDir::new().unwrap();
    let script = temp.child("render.py");
    script
        .write_str(
            r#"import json, sys
payload = json.load(sys.stdin)
template = payload.get("templateSource", "")
frontmatter = payload.get("frontmatter", {})
name = frontmatter.get("name", "")
template = template.replace("{{front_name}}", name)
print(json.dumps({"content": template}))
"#,
        )
        .unwrap();

    let command = format!("python3 {}", script.path().display());
    let data = json!({});
    let frontmatter = json!({"name": "project-rules"});
    let template = "Template: {{front_name}}";

    let result = render_external(&command, template, &data, &frontmatter, "rules.md").unwrap();
    assert_eq!(result, "Template: project-rules");
}
