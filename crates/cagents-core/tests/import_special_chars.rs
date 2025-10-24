// Test that importing AGENTS.md files with special glob characters in paths
// (like brackets, asterisks, etc.) properly escapes them in generated globs

use anyhow::Result;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_import_with_brackets_in_path() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path();

    // Create AGENTS.md in a path with brackets (common in Next.js routes)
    fs::write(root.join("AGENTS.md"), "Root content")?;
    fs::create_dir_all(root.join("src/pages/o/[orgSlug]/settings"))?;
    fs::write(
        root.join("src/pages/o/[orgSlug]/settings/AGENTS.md"),
        "Org settings content"
    )?;

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(root)?;

    // Import
    cagents_core::import::import_agents_md(false)?;

    // Check template was created with escaped brackets
    let template = fs::read_to_string(".cAGENTS/templates/agents-settings.md")?;
    assert!(template.contains("Org settings content"));

    // Brackets should be escaped with double backslash for YAML (which becomes single backslash after parsing)
    assert!(
        template.contains(r#"globs: ["src/pages/o/\\[orgSlug\\]/settings/"]"#),
        "Glob should have escaped brackets with double backslash, got: {}", template
    );
    assert!(template.contains("outputIn: \"matched\""));

    // Run build to verify structure is recreated
    cagents_core::cmd_build(None, false)?;

    // Verify nested file was recreated at correct location
    assert!(std::path::PathBuf::from("AGENTS.md").exists());
    assert!(std::path::PathBuf::from("src/pages/o/[orgSlug]/settings/AGENTS.md").exists());

    // Verify content is preserved
    let nested_content = fs::read_to_string("src/pages/o/[orgSlug]/settings/AGENTS.md")?;
    assert!(nested_content.contains("Org settings content"));

    // Restore directory
    std::env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
#[serial]
fn test_import_with_multiple_bracket_paths() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path();

    // Create AGENTS.md in paths with brackets (valid filesystem chars)
    fs::write(root.join("AGENTS.md"), "Root")?;

    // Single bracket param
    fs::create_dir_all(root.join("path/[id]"))?;
    fs::write(root.join("path/[id]/AGENTS.md"), "ID bracket content")?;

    // Multiple bracket params in path
    fs::create_dir_all(root.join("app/[locale]/[theme]"))?;
    fs::write(root.join("app/[locale]/[theme]/AGENTS.md"), "Multi bracket content")?;

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(root)?;

    // Import
    cagents_core::import::import_agents_md(false)?;

    // Check templates have escaped globs (double backslash for YAML)
    let id_template = fs::read_to_string(".cAGENTS/templates/agents-[id].md")?;
    assert!(id_template.contains(r#"globs: ["path/\\[id\\]/"]"#));

    let theme_template = fs::read_to_string(".cAGENTS/templates/agents-[theme].md")?;
    assert!(theme_template.contains(r#"globs: ["app/\\[locale\\]/\\[theme\\]/"]"#));

    // Run build
    cagents_core::cmd_build(None, false)?;

    // Verify all files were recreated
    assert!(std::path::PathBuf::from("path/[id]/AGENTS.md").exists());
    assert!(std::path::PathBuf::from("app/[locale]/[theme]/AGENTS.md").exists());

    // Restore directory
    std::env::set_current_dir(original_dir)?;

    Ok(())
}

#[test]
#[serial]
fn test_import_multi_format_with_brackets() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path();

    // Create root-level files (required for detection)
    fs::write(root.join("AGENTS.md"), "Root agents")?;
    fs::write(root.join("CLAUDE.md"), "Root claude")?;

    // Create both AGENTS.md and CLAUDE.md in bracketed path
    fs::create_dir_all(root.join("app/[locale]/dashboard"))?;
    fs::write(root.join("app/[locale]/dashboard/AGENTS.md"), "Agents content")?;
    fs::write(root.join("app/[locale]/dashboard/CLAUDE.md"), "Claude content")?;

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(root)?;

    // Detect and import all formats
    let formats = cagents_core::import::detect_all_formats();
    cagents_core::import::import_multiple_formats(&formats, false)?;

    // Check both templates have properly escaped globs (double backslash for YAML)
    let agents_template = fs::read_to_string(".cAGENTS/templates/agents-dashboard.md")?;
    assert!(agents_template.contains(r#"globs: ["app/\\[locale\\]/dashboard/"]"#));

    let claude_template = fs::read_to_string(".cAGENTS/templates/claude-dashboard.md")?;
    assert!(claude_template.contains(r#"globs: ["app/\\[locale\\]/dashboard/"]"#));

    // Run build to verify both files are recreated
    cagents_core::cmd_build(None, false)?;

    assert!(std::path::PathBuf::from("app/[locale]/dashboard/AGENTS.md").exists());
    assert!(std::path::PathBuf::from("app/[locale]/dashboard/CLAUDE.md").exists());

    // Restore directory
    std::env::set_current_dir(original_dir)?;

    Ok(())
}
