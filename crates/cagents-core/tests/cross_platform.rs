// Cross-platform compatibility tests
//
// These tests verify that path handling works correctly across different platforms

mod test_utils;

use std::path::PathBuf;

#[test]
fn test_backslash_conversion() {
    // Even on Unix, we should handle Windows-style paths correctly
    let windows_path = "C:\\Users\\test\\script.py";
    let path = PathBuf::from(windows_path);
    let cmd = test_utils::path_to_command("python3", &path);

    // Should not contain backslashes
    assert!(!cmd.contains('\\'), "Command should not contain backslashes: {}", cmd);

    // Should contain forward slashes (cross-platform compatible)
    assert!(cmd.contains('/'), "Command should contain forward slashes: {}", cmd);

    // Should be quoted
    assert!(cmd.contains('"'), "Path should be quoted: {}", cmd);
}

#[test]
fn test_spaces_in_path() {
    let path = PathBuf::from("/home/user name/my script.py");
    let cmd = test_utils::path_to_command("python3", &path);

    // Should be quoted to handle spaces
    assert!(cmd.starts_with("python3 \""), "Command should start with quoted path: {}", cmd);
    assert!(cmd.ends_with('"'), "Command should end with closing quote: {}", cmd);
}

#[test]
fn test_toml_config_format() {
    let path = PathBuf::from("C:\\Users\\test\\script.py");
    let toml_value = test_utils::format_toml_command("python3", &path);

    // Should use triple-quoted strings to avoid escaping issues
    assert!(toml_value.starts_with("\"\"\""), "TOML value should use triple quotes: {}", toml_value);
    assert!(toml_value.ends_with("\"\"\""), "TOML value should end with triple quotes: {}", toml_value);

    // Should not contain backslashes
    assert!(!toml_value.contains('\\'), "TOML value should not contain backslashes: {}", toml_value);
}

#[test]
fn test_yaml_frontmatter_format() {
    let path = PathBuf::from("C:\\Users\\test\\script.py");
    let yaml_value = test_utils::format_yaml_command("python3", &path);

    // Should use single quotes to avoid escaping double quotes
    assert!(yaml_value.starts_with('\''), "YAML value should use single quotes: {}", yaml_value);
    assert!(yaml_value.ends_with('\''), "YAML value should end with single quote: {}", yaml_value);

    // Should not contain backslashes
    assert!(!yaml_value.contains('\\'), "YAML value should not contain backslashes: {}", yaml_value);

    // Path inside should be quoted
    assert!(yaml_value.contains('"'), "Path should be quoted inside YAML: {}", yaml_value);
}

#[test]
fn test_special_characters_in_path() {
    let path = PathBuf::from("/home/user/file (1).py");
    let cmd = test_utils::path_to_command("python3", &path);

    // Should be quoted to handle parentheses
    assert!(cmd.contains('"'), "Path with special chars should be quoted: {}", cmd);
}

#[test]
fn test_relative_paths() {
    let path = PathBuf::from("./scripts/render.py");
    let cmd = test_utils::path_to_command("python3", &path);

    // Should still be quoted
    assert!(cmd.contains('"'), "Relative path should be quoted: {}", cmd);
}

#[test]
fn test_consistency_across_formats() {
    let path = PathBuf::from("C:\\Users\\test\\script.py");

    let toml = test_utils::format_toml_command("python3", &path);
    let yaml = test_utils::format_yaml_command("python3", &path);
    let cmd = test_utils::path_to_command("python3", &path);

    // All should use forward slashes
    assert!(!toml.contains('\\'), "TOML format inconsistent");
    assert!(!yaml.contains('\\'), "YAML format inconsistent");
    assert!(!cmd.contains('\\'), "Command format inconsistent");

    // All should include the quoted path
    assert!(toml.contains('"'), "TOML missing quotes");
    assert!(yaml.contains('"'), "YAML missing quotes");
    assert!(cmd.contains('"'), "Command missing quotes");
}
