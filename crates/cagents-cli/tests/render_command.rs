use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

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
    // Quote path and use forward slashes for cross-platform compatibility
    let path = script.path().display().to_string().replace('\\', "/");
    format!("python3 \"{}\"", path)
}

#[test]
fn test_render_outputs_to_stdout() {
    // Create a temporary directory with a simple cAGENTS setup
    let temp = assert_fs::TempDir::new().unwrap();

    // Create cAGENTS directory and config
    let cagents_dir = temp.child(".cAGENTS");
    cagents_dir.create_dir_all().unwrap();

    // Create renderer script
    let renderer_cmd = write_passthrough_renderer(&temp);

    let config = cagents_dir.child("config.toml");
    config.write_str(&format!(r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = """command:{}"""
"#, renderer_cmd)).unwrap();

    // Create templates directory
    let templates_dir = cagents_dir.child("templates");
    templates_dir.create_dir_all().unwrap();

    // Create a template that matches *.rs files
    let rust_template = templates_dir.child("rust-rules.md");
    rust_template.write_str(r#"---
name: "Rust Rules"
globs:
  - "**/*.rs"
---
# Rust Development Rules

Use idiomatic Rust patterns.
"#).unwrap();

    // Create a test Rust file
    let src_dir = temp.child("src");
    src_dir.create_dir_all().unwrap();
    let test_file = src_dir.child("main.rs");
    test_file.write_str("fn main() {}").unwrap();

    // Run cagents render for the Rust file
    let mut cmd = Command::cargo_bin("cagents").unwrap();
    cmd.current_dir(temp.path())
        .arg("render")
        .arg("src/main.rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust Development Rules"))
        .stdout(predicate::str::contains("idiomatic Rust patterns"));
}

#[test]
fn test_render_with_variables() {
    // Create a temporary directory with a simple cAGENTS setup
    let temp = assert_fs::TempDir::new().unwrap();

    // Create cAGENTS directory and config
    let cagents_dir = temp.child(".cAGENTS");
    cagents_dir.create_dir_all().unwrap();

    // Create renderer script
    let renderer_cmd = write_passthrough_renderer(&temp);

    let config = cagents_dir.child("config.toml");
    config.write_str(&format!(r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = """command:{}"""
"#, renderer_cmd)).unwrap();

    // Create templates directory
    let templates_dir = cagents_dir.child("templates");
    templates_dir.create_dir_all().unwrap();

    // Create a template with a variable placeholder (cat will just pass it through)
    let template = templates_dir.child("project-rules.md");
    template.write_str(r#"---
name: "Project Rules"
---
# Project: {{projectName}}

Developer: {{developer}}
"#).unwrap();

    // Create a test file
    let test_file = temp.child("test.txt");
    test_file.write_str("test").unwrap();

    // Run cagents render with variables
    let mut cmd = Command::cargo_bin("cagents").unwrap();
    cmd.current_dir(temp.path())
        .arg("render")
        .arg("test.txt")
        .arg("--var")
        .arg("projectName=MyProject")
        .arg("--var")
        .arg("developer=Jordan")
        .assert()
        .success()
        .stdout(predicate::str::contains("{{projectName}}"))
        .stdout(predicate::str::contains("{{developer}}"));

    // Note: Since we're using 'cat' as the engine, variables won't be rendered
    // In a real scenario with a proper engine, the output would contain:
    // "Project: MyProject" and "Developer: Jordan"
}

#[test]
fn test_render_no_matching_rules() {
    // Create a temporary directory with a simple cAGENTS setup
    let temp = assert_fs::TempDir::new().unwrap();

    // Create cAGENTS directory and config
    let cagents_dir = temp.child(".cAGENTS");
    cagents_dir.create_dir_all().unwrap();

    // Create renderer script
    let renderer_cmd = write_passthrough_renderer(&temp);

    let config = cagents_dir.child("config.toml");
    config.write_str(&format!(r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = """command:{}"""
"#, renderer_cmd)).unwrap();

    // Create templates directory but no templates
    let templates_dir = cagents_dir.child("templates");
    templates_dir.create_dir_all().unwrap();

    // Create a template that matches *.rs files only
    let rust_template = templates_dir.child("rust-rules.md");
    rust_template.write_str(r#"---
name: "Rust Rules"
globs:
  - "**/*.rs"
---
# Rust Rules
"#).unwrap();

    // Create a TypeScript file (won't match)
    let test_file = temp.child("test.ts");
    test_file.write_str("console.log('test')").unwrap();

    // Run cagents render - should output warning to stderr
    let mut cmd = Command::cargo_bin("cagents").unwrap();
    cmd.current_dir(temp.path())
        .arg("render")
        .arg("test.ts")
        .assert()
        .success()
        .stderr(predicate::str::contains("No rules match file"));
}

#[test]
fn test_render_rules_without_when_clause() {
    // Create a temporary directory with a simple cAGENTS setup
    let temp = assert_fs::TempDir::new().unwrap();

    // Create cAGENTS directory and config
    let cagents_dir = temp.child(".cAGENTS");
    cagents_dir.create_dir_all().unwrap();

    // Create renderer script
    let renderer_cmd = write_passthrough_renderer(&temp);

    let config = cagents_dir.child("config.toml");
    config.write_str(&format!(r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = """command:{}"""
"#, renderer_cmd)).unwrap();

    // Create templates directory
    let templates_dir = cagents_dir.child("templates");
    templates_dir.create_dir_all().unwrap();

    // Create a template without when clause (implicitly applies to all files)
    let global_template = templates_dir.child("global-rules.md");
    global_template.write_str(r#"---
name: "Global Rules"
---
# Global Rules

These rules apply to all files.
"#).unwrap();

    // Create any file
    let test_file = temp.child("anyfile.xyz");
    test_file.write_str("test").unwrap();

    // Run cagents render - should include rule without when clause
    let mut cmd = Command::cargo_bin("cagents").unwrap();
    cmd.current_dir(temp.path())
        .arg("render")
        .arg("anyfile.xyz")
        .assert()
        .success()
        .stdout(predicate::str::contains("Global Rules"))
        .stdout(predicate::str::contains("apply to all files"));
}
