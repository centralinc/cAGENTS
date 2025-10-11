// Test lint validation for config and template values

use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use std::env;

#[test]
#[serial]
fn test_lint_catches_invalid_output_target() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create config with invalid output target
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[output]
targets = ["agents-md", "invalid-target", "claude-md"]
"#).unwrap();

    // Run lint
    let result = cagents_core::lint::lint_all().unwrap();

    // Should have error for invalid target
    assert!(result.has_errors(), "Should have errors for invalid target");
    let error_msg = result.issues.iter()
        .find(|i| i.message.contains("invalid-target"))
        .expect("Should have error mentioning 'invalid-target'");

    assert!(error_msg.message.contains("Unknown output target") ||
            error_msg.message.contains("Invalid target"),
        "Error should mention invalid target type");
}

#[test]
#[serial]
fn test_lint_catches_invalid_when_target() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create config and template with invalid when.target
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"
"#).unwrap();

    fs::write(".cAGENTS/templates/bad.md", r#"---
name: bad-target
alwaysApply: true
when:
  target: ["invalid-target", "another-bad"]
---
# Content
"#).unwrap();

    // Run lint
    let result = cagents_core::lint::lint_all().unwrap();

    // Should have error for invalid when.target
    assert!(result.has_errors(), "Should have errors for invalid when.target values");

    let has_target_error = result.issues.iter().any(|i| {
        i.message.contains("invalid-target") || i.message.contains("when.target")
    });

    assert!(has_target_error, "Should have error about invalid when.target value");
}

#[test]
#[serial]
fn test_lint_accepts_valid_targets() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create config with all valid targets
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[output]
targets = ["agents-md", "claude-md", "cursorrules"]
"#).unwrap();

    fs::write(".cAGENTS/templates/good.md", r#"---
name: good-target
alwaysApply: true
when:
  target: ["agents-md", "claude-md"]
---
# Content
"#).unwrap();

    // Run lint
    let result = cagents_core::lint::lint_all().unwrap();

    // Should have no errors related to targets
    let has_target_error = result.issues.iter().any(|i| {
        i.severity == cagents_core::lint::Severity::Error &&
        (i.message.contains("target") || i.message.contains("Invalid"))
    });

    assert!(!has_target_error, "Should not have target-related errors for valid targets");
}

#[test]
#[serial]
fn test_lint_validates_all_output_targets() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create config with mix of valid and invalid targets
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[output]
targets = ["agents-md", "foo", "claude-md", "bar", "cursorrules"]
"#).unwrap();

    // Run lint
    let result = cagents_core::lint::lint_all().unwrap();

    // Should catch both invalid targets
    assert!(result.has_errors(), "Should have errors");

    let errors: Vec<_> = result.issues.iter()
        .filter(|i| i.severity == cagents_core::lint::Severity::Error)
        .collect();

    let has_foo = errors.iter().any(|e| e.message.contains("foo"));
    let has_bar = errors.iter().any(|e| e.message.contains("bar"));

    assert!(has_foo, "Should catch 'foo' as invalid target");
    assert!(has_bar, "Should catch 'bar' as invalid target");
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
