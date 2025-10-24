// Test that importing nested AGENTS.md/CLAUDE.md files creates templates
// with globs that rebuild to the same nested structure

use anyhow::Result;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_import_agents_md_preserves_nested_structure() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path();

    // Create nested AGENTS.md files
    fs::write(root.join("AGENTS.md"), "Root content")?;
    fs::create_dir_all(root.join("docs"))?;
    fs::write(root.join("docs/AGENTS.md"), "Docs content")?;
    fs::create_dir_all(root.join("src/api"))?;
    fs::write(root.join("src/api/AGENTS.md"), "API content")?;

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(root)?;

    // Import
    cagents_core::import::import_agents_md(false)?;

    // Check templates were created with proper globs (use relative paths now that we're in root)
    let templates_dir = std::path::PathBuf::from(".cAGENTS/templates");
    assert!(templates_dir.exists());

    // Check root template (no glob)
    let root_template = fs::read_to_string(templates_dir.join("agents-root.md"))?;
    assert!(root_template.contains("Root content"));
    assert!(!root_template.contains("globs:"));
    assert!(!root_template.contains("outputIn:"));

    // Check docs template (has glob)
    let docs_template = fs::read_to_string(templates_dir.join("agents-docs.md"))?;
    assert!(docs_template.contains("Docs content"));
    assert!(docs_template.contains("globs: [\"docs/\"]"));
    assert!(docs_template.contains("outputIn: \"matched\""));

    // Check api template (has glob)
    let api_template = fs::read_to_string(templates_dir.join("agents-api.md"))?;
    assert!(api_template.contains("API content"));
    assert!(api_template.contains("globs: [\"src/api/\"]"));
    assert!(api_template.contains("outputIn: \"matched\""));

    // Run build to verify structure is recreated
    cagents_core::cmd_build(None, false)?;

    // Verify all AGENTS.md files were recreated at correct locations (relative paths)
    assert!(std::path::PathBuf::from("AGENTS.md").exists());
    assert!(std::path::PathBuf::from("docs/AGENTS.md").exists());
    assert!(std::path::PathBuf::from("src/api/AGENTS.md").exists());

    // Verify content is preserved
    let root_content = fs::read_to_string("AGENTS.md")?;
    assert!(root_content.contains("Root content"));

    let docs_content = fs::read_to_string("docs/AGENTS.md")?;
    assert!(docs_content.contains("Docs content"));

    let api_content = fs::read_to_string("src/api/AGENTS.md")?;
    assert!(api_content.contains("API content"));

    // Restore directory
    std::env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
#[serial]
fn test_import_claude_md_preserves_nested_structure() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path();

    // Create nested CLAUDE.md files
    fs::write(root.join("CLAUDE.md"), "Root content")?;
    fs::create_dir_all(root.join("backend"))?;
    fs::write(root.join("backend/CLAUDE.md"), "Backend content")?;

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(root)?;

    // Import
    cagents_core::import::import_claude_md(false)?;

    // Check templates were created with proper globs (use relative paths)
    let templates_dir = std::path::PathBuf::from(".cAGENTS/templates");

    // Check root template (no glob)
    let root_template = fs::read_to_string(templates_dir.join("agents-root.md"))?;
    assert!(root_template.contains("Root content"));
    assert!(!root_template.contains("globs:"));

    // Check backend template (has glob)
    let backend_template = fs::read_to_string(templates_dir.join("agents-backend.md"))?;
    assert!(backend_template.contains("Backend content"));
    assert!(backend_template.contains("globs: [\"backend/\"]"));
    assert!(backend_template.contains("outputIn: \"matched\""));

    // Run build to verify structure is recreated
    // Need to set output target to claude-md
    let config = fs::read_to_string(".cAGENTS/config.toml")?;
    let config_with_target = format!("{}\n\n[output]\ntargets = [\"claude-md\"]\n", config);
    fs::write(".cAGENTS/config.toml", config_with_target)?;

    cagents_core::cmd_build(None, false)?;

    // Verify all CLAUDE.md files were recreated (relative paths)
    assert!(std::path::PathBuf::from("CLAUDE.md").exists());
    assert!(std::path::PathBuf::from("backend/CLAUDE.md").exists());

    // Restore directory
    std::env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
#[serial]
fn test_import_multiple_formats_preserves_nested_structure() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path();

    // Create nested files for multiple formats
    fs::write(root.join("AGENTS.md"), "Agents root")?;
    fs::create_dir_all(root.join("docs"))?;
    fs::write(root.join("docs/AGENTS.md"), "Agents docs")?;

    fs::write(root.join("CLAUDE.md"), "Claude root")?;
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join("src/CLAUDE.md"), "Claude src")?;

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(root)?;

    // Detect and import all formats
    let formats = cagents_core::import::detect_all_formats();
    cagents_core::import::import_multiple_formats(&formats, false)?;

    // Check templates were created with proper globs (relative paths)
    let templates_dir = std::path::PathBuf::from(".cAGENTS/templates");

    // Check AGENTS.md templates
    let agents_root = fs::read_to_string(templates_dir.join("agents-root.md"))?;
    assert!(agents_root.contains("Agents root"));
    assert!(!agents_root.contains("globs:"));

    let agents_docs = fs::read_to_string(templates_dir.join("agents-docs.md"))?;
    assert!(agents_docs.contains("Agents docs"));
    assert!(agents_docs.contains("globs: [\"docs/\"]"));
    assert!(agents_docs.contains("outputIn: \"matched\""));
    assert!(agents_docs.contains("target: [\"agents-md\"]"));

    // Check CLAUDE.md templates
    let claude_root = fs::read_to_string(templates_dir.join("claude-root.md"))?;
    assert!(claude_root.contains("Claude root"));
    assert!(!claude_root.contains("globs:"));
    assert!(claude_root.contains("target: [\"claude-md\"]"));

    let claude_src = fs::read_to_string(templates_dir.join("claude-src.md"))?;
    assert!(claude_src.contains("Claude src"));
    assert!(claude_src.contains("globs: [\"src/\"]"));
    assert!(claude_src.contains("outputIn: \"matched\""));
    assert!(claude_src.contains("target: [\"claude-md\"]"));

    // Run build to verify both structures are recreated
    cagents_core::cmd_build(None, false)?;

    // Verify AGENTS.md files (relative paths)
    assert!(std::path::PathBuf::from("AGENTS.md").exists());
    assert!(std::path::PathBuf::from("docs/AGENTS.md").exists());

    // Verify CLAUDE.md files (relative paths)
    assert!(std::path::PathBuf::from("CLAUDE.md").exists());
    assert!(std::path::PathBuf::from("src/CLAUDE.md").exists());

    // Restore directory
    std::env::set_current_dir(original_dir)?;

    Ok(())
}
