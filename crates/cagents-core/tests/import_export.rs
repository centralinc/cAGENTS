use cagents_core::import::{import_cursorrules, detect_cursor_format, detect_all_formats, CursorFormat, ImportFormat};
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
#[serial]
fn test_import_cursorrules() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create .cursorrules file
    let cursorrules_content = "# My Cursor Rules\n\n- Use TypeScript\n- Write tests\n";
    fs::write(".cursorrules", cursorrules_content).unwrap();

    import_cursorrules(false).unwrap();

    // Verify structure created (new naming)
    assert!(PathBuf::from(".cAGENTS/config.toml").exists());
    assert!(PathBuf::from(".cAGENTS/templates/agents-cursor-root.md").exists());
    assert!(!PathBuf::from(".cursorrules.backup").exists(),
        "Should NOT create backup when backup=false");

    // Verify content preserved
    let template = fs::read_to_string(".cAGENTS/templates/agents-cursor-root.md").unwrap();
    assert!(template.contains("My Cursor Rules"));
    assert!(template.contains("Use TypeScript"));

    // Verify config has cursor settings
    let config = fs::read_to_string(".cAGENTS/config.toml").unwrap();
    assert!(config.contains("cursor"));
}

#[test]
#[serial]
fn test_detect_cursor_format_legacy() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::write(".cursorrules", "rules").unwrap();

    let format = detect_cursor_format();
    assert_eq!(format, CursorFormat::LegacyRootFile);
}

#[test]
#[serial]
fn test_detect_cursor_format_modern() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all(".cursor/rules").unwrap();

    let format = detect_cursor_format();
    assert_eq!(format, CursorFormat::ModernRulesDir);
}

#[test]
#[serial]
fn test_detect_cursor_format_both() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::write(".cursorrules", "rules").unwrap();
    fs::create_dir_all(".cursor/rules").unwrap();

    let format = detect_cursor_format();
    assert_eq!(format, CursorFormat::Both);
}

#[test]
#[serial]
fn test_detect_cursor_format_none() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    let format = detect_cursor_format();
    assert_eq!(format, CursorFormat::None);
}

// Tests for multi-format detection

#[test]
#[serial]
fn test_detect_all_formats_only_cursor() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::write(".cursorrules", "rules").unwrap();

    let formats = detect_all_formats();
    assert_eq!(formats.len(), 1);
    assert!(matches!(formats[0], ImportFormat::CursorLegacy));
}

#[test]
#[serial]
fn test_detect_all_formats_multiple() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::write(".cursorrules", "cursor rules").unwrap();
    fs::write("AGENTS.md", "# Agents").unwrap();
    fs::write("CLAUDE.md", "# Claude").unwrap();

    let formats = detect_all_formats();
    assert_eq!(formats.len(), 3, "Should detect all three formats");
    assert!(formats.iter().any(|f| matches!(f, ImportFormat::CursorLegacy)));
    assert!(formats.iter().any(|f| matches!(f, ImportFormat::AgentsMd)));
    assert!(formats.iter().any(|f| matches!(f, ImportFormat::ClaudeMd)));
}

#[test]
#[serial]
fn test_detect_all_formats_agents_and_claude() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::write("AGENTS.md", "# Agents").unwrap();
    fs::write("CLAUDE.md", "# Claude").unwrap();

    let formats = detect_all_formats();
    assert_eq!(formats.len(), 2);
    assert!(formats.iter().any(|f| matches!(f, ImportFormat::AgentsMd)));
    assert!(formats.iter().any(|f| matches!(f, ImportFormat::ClaudeMd)));
}

#[test]
#[serial]
fn test_detect_all_formats_cursor_modern() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    fs::create_dir_all(".cursor/rules").unwrap();
    fs::write(".cursor/rules/test.md", "test").unwrap();

    let formats = detect_all_formats();
    assert_eq!(formats.len(), 1);
    assert!(matches!(formats[0], ImportFormat::CursorModern));
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
