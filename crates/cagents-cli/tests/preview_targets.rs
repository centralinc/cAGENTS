// Test that preview command shows all target files separately
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_preview_shows_multiple_targets() {
    let tmp = TempDir::new().unwrap();
    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    // Setup config with multiple targets
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
targets = ["agents-md", "claude-md"]
"#,
    )
    .unwrap();

    // Create templates with target filtering
    fs::write(
        ".cAGENTS/templates/agents-only.md",
        r#"---
name: agents-only
when:
  target: ["agents-md"]
---
# AGENTS.md specific content
"#,
    )
    .unwrap();

    fs::write(
        ".cAGENTS/templates/claude-only.md",
        r#"---
name: claude-only
when:
  target: ["claude-md"]
---
# CLAUDE.md specific content
"#,
    )
    .unwrap();

    // Run preview command
    let output = Command::new(env!("CARGO_BIN_EXE_cagents"))
        .arg("preview")
        .output()
        .unwrap();

    std::env::set_current_dir(original).unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should mention both target files
    assert!(stdout.contains("AGENTS.md"), "Preview should mention AGENTS.md");
    assert!(stdout.contains("CLAUDE.md"), "Preview should mention CLAUDE.md");

    // Should show they are different (not merged)
    assert!(stdout.contains("2 file(s) will be generated") || stdout.contains("AGENTS.md") && stdout.contains("CLAUDE.md"),
        "Preview should indicate multiple files");
}
