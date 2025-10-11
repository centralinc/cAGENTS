use cagents_core::lint::{validate_config, validate_templates, lint_all};
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_validate_config_missing() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    let result = validate_config().unwrap();
    assert!(result.has_errors());
    assert!(result.error_count() > 0);
}

#[test]
#[serial]
fn test_validate_config_valid() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create valid config
    fs::create_dir_all(".cAGENTS/templates").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."
"#).unwrap();

    let result = validate_config().unwrap();
    assert!(!result.has_errors());
}

#[test]
#[serial]
fn test_validate_config_invalid_toml() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all(".cAGENTS").unwrap();
    fs::write(".cAGENTS/config.toml", "invalid {{ toml").unwrap();

    let result = validate_config().unwrap();
    assert!(result.has_errors());
    assert!(result.issues[0].message.contains("Failed to parse"));
}

#[test]
#[serial]
fn test_validate_templates_missing_dir() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Config exists but templates dir doesn't
    fs::create_dir_all(".cAGENTS").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."
"#).unwrap();

    let result = validate_templates().unwrap();
    // Should not error (config validation catches missing dir)
    assert!(!result.has_errors());
}

#[test]
#[serial]
fn test_lint_all_combines_results() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // No config at all
    let result = lint_all().unwrap();
    assert!(result.has_errors());
}

/// Helper to change directory and restore on drop
struct ChangeDir {
    original: std::path::PathBuf,
}

impl ChangeDir {
    fn new(path: &std::path::Path) -> Self {
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();
        Self { original }
    }
}

impl Drop for ChangeDir {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}
