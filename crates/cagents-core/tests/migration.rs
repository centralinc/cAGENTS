use cagents_core::init::{migrate_simple, parse_agents_md, ProjectInfo};
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_parse_agents_md_sections() {
    let content = r#"# Project Rules

Owner: Alice

## TypeScript

- Use strict mode

## Backend

- Use PostgreSQL
"#;

    let sections = parse_agents_md(content);

    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].heading, "# Project Rules");
    assert!(sections[0].content.contains("Owner: Alice"));
    assert_eq!(sections[1].heading, "## TypeScript");
    assert_eq!(sections[2].heading, "## Backend");
}

#[test]
#[serial]
fn test_migrate_simple_preserves_content() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create AGENTS.md
    let original_content = "# My Rules\n\nContent here\n";
    fs::write("AGENTS.md", original_content).unwrap();

    let info = ProjectInfo {
        name: "test".to_string(),
        owner: None,
        has_git: false,
        has_agents_md: true,
        has_cagents_dir: false,
        has_claude_md: false,
        has_cursorrules: false,
        has_cursor_rules: false,
        agents_md_locations: vec![],
    };

    migrate_simple(&info, false, false).unwrap();

    // Verify template contains original content (new naming)
    let template = fs::read_to_string(".cAGENTS/templates/agents-root.md").unwrap();
    assert!(template.contains("# My Rules"));
    assert!(template.contains("Content here"));
    assert!(template.contains("---")); // Has frontmatter
}

#[test]
#[serial]
fn test_import_multiple_formats_with_targets() {
    use cagents_core::import::{import_multiple_formats, ImportFormat};

    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create both AGENTS.md and CLAUDE.md
    fs::write("AGENTS.md", "# Agents Content\n\nFor agents.").unwrap();
    fs::write("CLAUDE.md", "# Claude Content\n\nFor claude.").unwrap();

    let formats = vec![ImportFormat::AgentsMd, ImportFormat::ClaudeMd];
    import_multiple_formats(&formats, false).unwrap();

    // Check config has both output targets
    let config_content = fs::read_to_string(".cAGENTS/config.toml").unwrap();
    assert!(config_content.contains("[output]"), "Config should have [output] section");
    assert!(config_content.contains("agents-md"), "Config should have agents-md target");
    assert!(config_content.contains("claude-md"), "Config should have claude-md target");

    // Check AGENTS.md template has target filter for agents-md
    let agents_template = fs::read_to_string(".cAGENTS/templates/agents-from-agentsmd.md").unwrap();
    assert!(agents_template.contains("when:"), "Template should have when section");
    assert!(agents_template.contains("target: [\"agents-md\"]"), "Template should filter to agents-md");
    assert!(agents_template.contains("# Agents Content"), "Template should have original content");

    // Check CLAUDE.md template has target filter for claude-md
    let claude_template = fs::read_to_string(".cAGENTS/templates/agents-from-claudemd.md").unwrap();
    assert!(claude_template.contains("when:"), "Template should have when section");
    assert!(claude_template.contains("target: [\"claude-md\"]"), "Template should filter to claude-md");
    assert!(claude_template.contains("# Claude Content"), "Template should have original content");
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
