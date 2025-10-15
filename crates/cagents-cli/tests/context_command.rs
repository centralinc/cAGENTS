use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

// Import cross-platform path utilities from cagents-core tests
// Note: This requires adding the path to test dependencies
use std::path::Path;

fn path_to_command(cmd: &str, path: &Path) -> String {
    let path_str = path.display().to_string().replace('\\', "/");
    format!("{} \"{}\"", cmd, path_str)
}

// Helper to create a simple pass-through renderer
fn write_passthrough_renderer(dir: &assert_fs::TempDir) -> String {
    let script = dir.child("render.py");
    script
        .write_str(
            r#"import json, sys
payload = json.load(sys.stdin)
template = payload.get("templateSource", "")
print(json.dumps({"content": template}))
"#,
        )
        .unwrap();
    // Use cross-platform path utility
    path_to_command("python3", script.path())
}

#[test]
fn test_context_markdown_output() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create cAGENTS setup
    let cagents_dir = temp.child(".cAGENTS");
    cagents_dir.create_dir_all().unwrap();

    let renderer_cmd = write_passthrough_renderer(&temp);

    let config = cagents_dir.child("config.toml");
    config.write_str(&format!(r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = """command:{}"""

[variables.static]
project = "test-project"
owner = "Jordan"
"#, renderer_cmd)).unwrap();

    // Create templates
    let templates_dir = cagents_dir.child("templates");
    templates_dir.create_dir_all().unwrap();

    let rust_template = templates_dir.child("agents-rust.md");
    rust_template.write_str(r#"---
name: "rust-rules"
globs:
  - "**/*.rs"
order: 10
---
# Rust Rules
Use idiomatic Rust.
"#).unwrap();

    let global_template = templates_dir.child("agents-root.md");
    global_template.write_str(r#"---
name: "global-rules"
alwaysApply: true
order: 1
---
# Global Rules
General project rules.
"#).unwrap();

    // Create test file
    let src_dir = temp.child("src");
    src_dir.create_dir_all().unwrap();
    let test_file = src_dir.child("main.rs");
    test_file.write_str("fn main() {}").unwrap();

    // Run context command (default Markdown output)
    let mut cmd = Command::cargo_bin("cagents").unwrap();
    cmd.current_dir(temp.path())
        .arg("context")
        .arg("src/main.rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Context for"))
        .stdout(predicate::str::contains("Matched Rules"))
        .stdout(predicate::str::contains("global-rules"))
        .stdout(predicate::str::contains("rust-rules"))
        .stdout(predicate::str::contains("Available Variables"))
        .stdout(predicate::str::contains("File Info"));
}

#[test]
fn test_context_json_output() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create cAGENTS setup
    let cagents_dir = temp.child(".cAGENTS");
    cagents_dir.create_dir_all().unwrap();

    let renderer_cmd = write_passthrough_renderer(&temp);

    let config = cagents_dir.child("config.toml");
    config.write_str(&format!(r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = """command:{}"""

[variables.static]
project = "test-project"
"#, renderer_cmd)).unwrap();

    // Create templates
    let templates_dir = cagents_dir.child("templates");
    templates_dir.create_dir_all().unwrap();

    let template = templates_dir.child("agents-root.md");
    template.write_str(r#"---
name: "global"
alwaysApply: true
---
# Rules
"#).unwrap();

    // Create test file
    let test_file = temp.child("test.txt");
    test_file.write_str("test").unwrap();

    // Run context command with --json
    let mut cmd = Command::cargo_bin("cagents").unwrap();
    let output = cmd.current_dir(temp.path())
        .arg("context")
        .arg("test.txt")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify JSON structure
    assert!(json.get("file").is_some());
    assert!(json.get("matched_rules").is_some());
    assert!(json.get("variables").is_some());
    assert!(json.get("file_info").is_some());
    assert!(json.get("rendered_content").is_some());
}

#[test]
fn test_context_with_variables() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create cAGENTS setup
    let cagents_dir = temp.child(".cAGENTS");
    cagents_dir.create_dir_all().unwrap();

    let renderer_cmd = write_passthrough_renderer(&temp);

    let config = cagents_dir.child("config.toml");
    config.write_str(&format!(r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = """command:{}"""
"#, renderer_cmd)).unwrap();

    // Create templates
    let templates_dir = cagents_dir.child("templates");
    templates_dir.create_dir_all().unwrap();

    let template = templates_dir.child("agents-root.md");
    template.write_str(r#"---
name: "test"
alwaysApply: true
---
# Rules
"#).unwrap();

    let test_file = temp.child("test.txt");
    test_file.write_str("test").unwrap();

    // Run with custom variables
    let mut cmd = Command::cargo_bin("cagents").unwrap();
    cmd.current_dir(temp.path())
        .arg("context")
        .arg("test.txt")
        .arg("--var")
        .arg("env=prod")
        .arg("--var")
        .arg("team=backend")
        .assert()
        .success()
        .stdout(predicate::str::contains("env"))
        .stdout(predicate::str::contains("prod"))
        .stdout(predicate::str::contains("team"))
        .stdout(predicate::str::contains("backend"));
}
