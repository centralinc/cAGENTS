// Cross-platform test utilities for handling file paths in configuration strings
//
// Windows paths contain backslashes which cause issues when embedded in TOML/YAML:
// 1. Backslashes are escape sequences in strings
// 2. Unquoted paths with spaces/special chars break shell interpretation
//
// This module provides utilities to ensure consistent, correct path handling.

use std::path::Path;

/// Convert a path to a cross-platform command string
///
/// This function:
/// 1. Converts backslashes to forward slashes (Windows accepts both)
/// 2. Wraps the path in quotes to handle spaces
///
/// # Example
/// ```
/// let path = std::path::Path::new("C:\\Users\\test\\script.py");
/// let cmd = path_to_command("python3", path);
/// assert_eq!(cmd, "python3 \"C:/Users/test/script.py\"");
/// ```
pub fn path_to_command(command: &str, path: &Path) -> String {
    let path_str = path.display().to_string().replace('\\', "/");
    format!("{} \"{}\"", command, path_str)
}

/// Convert a path to a TOML-safe string for use in config values
///
/// Uses TOML's triple-quoted string syntax to avoid escaping issues
///
/// # Example
/// ```
/// let path = std::path::Path::new("C:\\Users\\test\\script.py");
/// let toml = path_to_toml_string(path);
/// assert_eq!(toml, "\"\"\"C:/Users/test/script.py\"\"\"");
/// ```
pub fn path_to_toml_string(path: &Path) -> String {
    let path_str = path.display().to_string().replace('\\', "/");
    format!("\"\"\"{}\"\"\"", path_str)
}

/// Convert a path to a YAML-safe string for use in frontmatter
///
/// Uses YAML's single-quoted syntax to avoid escaping double quotes
///
/// # Example
/// ```
/// let path = std::path::Path::new("C:\\Users\\test\\script.py");
/// let yaml = path_to_yaml_string(path);
/// assert_eq!(yaml, "'C:/Users/test/script.py'");
/// ```
pub fn path_to_yaml_string(path: &Path) -> String {
    let path_str = path.display().to_string().replace('\\', "/");
    format!("'{}'", path_str)
}

/// Format a command string for use in TOML config
///
/// Combines command and path into a TOML-safe triple-quoted string
///
/// # Example
/// ```
/// let path = std::path::Path::new("C:\\Users\\test\\script.py");
/// let toml = format_toml_command("python3", path);
/// assert_eq!(toml, "\"\"\"python3 \"C:/Users/test/script.py\"\"\"\"");
/// ```
pub fn format_toml_command(command: &str, path: &Path) -> String {
    let cmd = path_to_command(command, path);
    format!("\"\"\"{}\"\"\"", cmd)
}

/// Format a command string for use in YAML frontmatter
///
/// Combines command and path into a YAML-safe single-quoted string
///
/// # Example
/// ```
/// let path = std::path::Path::new("C:\\Users\\test\\script.py");
/// let yaml = format_yaml_command("python3", path);
/// assert_eq!(yaml, "'python3 \"C:/Users/test/script.py\"'");
/// ```
pub fn format_yaml_command(command: &str, path: &Path) -> String {
    let cmd = path_to_command(command, path);
    format!("'{}'", cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_to_command_unix() {
        let path = PathBuf::from("/home/user/script.py");
        let cmd = path_to_command("python3", &path);
        assert_eq!(cmd, "python3 \"/home/user/script.py\"");
    }

    #[test]
    fn test_path_to_command_windows() {
        // Simulates Windows path with backslashes
        let path_str = "C:\\Users\\test\\script.py";
        let path = PathBuf::from(path_str);
        let cmd = path_to_command("python3", &path);

        // Should convert backslashes to forward slashes and quote
        assert!(cmd.contains("python3"));
        assert!(cmd.contains("\""));
        assert!(!cmd.contains("\\")); // No backslashes
        assert!(cmd.contains("/")); // Forward slashes
    }

    #[test]
    fn test_path_with_spaces() {
        let path = PathBuf::from("/home/user name/my script.py");
        let cmd = path_to_command("python3", &path);
        assert!(cmd.starts_with("python3 \""));
        assert!(cmd.ends_with("\""));
        assert!(cmd.contains("my script.py"));
    }

    #[test]
    fn test_toml_string_format() {
        let path = PathBuf::from("/home/user/script.py");
        let toml = path_to_toml_string(&path);
        assert!(toml.starts_with("\"\"\""));
        assert!(toml.ends_with("\"\"\""));
        assert!(toml.contains("/home/user/script.py"));
    }

    #[test]
    fn test_yaml_string_format() {
        let path = PathBuf::from("/home/user/script.py");
        let yaml = path_to_yaml_string(&path);
        assert!(yaml.starts_with("'"));
        assert!(yaml.ends_with("'"));
        assert!(yaml.contains("/home/user/script.py"));
    }

    #[test]
    fn test_format_toml_command() {
        let path = PathBuf::from("/usr/bin/python");
        let toml = format_toml_command("python3", &path);
        assert!(toml.starts_with("\"\"\"python3"));
        assert!(toml.ends_with("\"\"\""));
        assert!(toml.contains("\"/usr/bin/python\""));
    }

    #[test]
    fn test_format_yaml_command() {
        let path = PathBuf::from("/usr/bin/python");
        let yaml = format_yaml_command("python3", &path);
        assert!(yaml.starts_with("'python3"));
        assert!(yaml.ends_with("'"));
        assert!(yaml.contains("\"/usr/bin/python\""));
    }

    #[test]
    fn test_no_double_escaping() {
        // Ensure we don't double-escape already normalized paths
        let path = PathBuf::from("C:/Users/test/script.py");
        let cmd = path_to_command("python3", &path);
        assert_eq!(cmd.matches('"').count(), 2); // Only opening and closing quotes
    }
}
