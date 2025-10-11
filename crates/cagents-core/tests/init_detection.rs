// Test detection of existing rule files during init

use assert_fs::prelude::*;
use serial_test::serial;

#[test]
#[serial]
fn test_detect_claude_md_during_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create CLAUDE.md file
    temp.child("CLAUDE.md").write_str("# Claude Rules\nTest rules").unwrap();

    // Save current dir and change to temp
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp).unwrap();

    // Detect project info
    let info = cagents_core::init::ProjectInfo::detect().unwrap();

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    // Should detect CLAUDE.md
    assert!(info.has_claude_md, "Should detect CLAUDE.md");
}

#[test]
#[serial]
fn test_detect_cursorrules_during_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create .cursorrules file
    temp.child(".cursorrules").write_str("# Cursor Rules\nTest rules").unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp).unwrap();

    // Detect project info
    let info = cagents_core::init::ProjectInfo::detect().unwrap();

    std::env::set_current_dir(original_dir).unwrap();

    // Should detect .cursorrules
    assert!(info.has_cursorrules, "Should detect .cursorrules");
}

#[test]
#[serial]
fn test_detect_cursor_rules_dir_during_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create .cursor/rules directory with files
    temp.child(".cursor/rules").create_dir_all().unwrap();
    temp.child(".cursor/rules/main.md").write_str("# Main Rules").unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp).unwrap();

    // Detect project info
    let info = cagents_core::init::ProjectInfo::detect().unwrap();

    std::env::set_current_dir(original_dir).unwrap();

    // Should detect .cursor/rules
    assert!(info.has_cursor_rules, "Should detect .cursor/rules");
}

#[test]
#[serial]
fn test_detect_multiple_formats_during_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create multiple format files
    temp.child("AGENTS.md").write_str("# Agents").unwrap();
    temp.child("CLAUDE.md").write_str("# Claude").unwrap();
    temp.child(".cursorrules").write_str("# Cursor").unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp).unwrap();

    // Detect project info
    let info = cagents_core::init::ProjectInfo::detect().unwrap();

    std::env::set_current_dir(original_dir).unwrap();

    // Should detect all formats
    assert!(info.has_agents_md);
    assert!(info.has_claude_md);
    assert!(info.has_cursorrules);
}

#[test]
#[serial]
fn test_init_prompts_when_claude_md_exists() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create CLAUDE.md
    temp.child("CLAUDE.md").write_str("# Claude Rules\nExisting rules").unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp).unwrap();

    // Run init - in non-interactive mode, should detect and offer import
    // This is a smoke test - interactive behavior tested separately
    let info = cagents_core::init::ProjectInfo::detect().unwrap();

    std::env::set_current_dir(original_dir).unwrap();

    assert!(info.has_claude_md);

    // The actual prompting logic will be implemented
}
