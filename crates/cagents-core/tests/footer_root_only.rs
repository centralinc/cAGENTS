use std::fs;
use tempfile::TempDir;

#[test]
fn test_agents_md_footer_only_in_root() {
    let temp_dir = TempDir::new().unwrap();
    let root_content = "# Root Rules\n\nThis is root.";
    let nested_content = "# Nested Rules\n\nThis is nested.";

    // Write root AGENTS.md
    cagents_core::writers::agents_md::write_agents_md(temp_dir.path(), root_content, true)
        .unwrap();

    // Write nested AGENTS.md
    let nested_dir = temp_dir.path().join("src");
    fs::create_dir_all(&nested_dir).unwrap();
    cagents_core::writers::agents_md::write_agents_md(&nested_dir, nested_content, false)
        .unwrap();

    // Read root AGENTS.md - should have footer
    let root_written = fs::read_to_string(temp_dir.path().join("AGENTS.md")).unwrap();
    assert!(
        root_written.contains("Working with cAGENTS"),
        "Root AGENTS.md should contain footer"
    );
    assert!(root_written.contains("# Root Rules"));

    // Read nested AGENTS.md - should NOT have footer
    let nested_written = fs::read_to_string(nested_dir.join("AGENTS.md")).unwrap();
    assert!(
        !nested_written.contains("Working with cAGENTS"),
        "Nested AGENTS.md should NOT contain footer"
    );
    assert!(nested_written.contains("# Nested Rules"));
}

#[test]
fn test_claude_md_footer_only_in_root() {
    let temp_dir = TempDir::new().unwrap();
    let root_content = "# Root Claude Rules\n\nThis is root.";
    let nested_content = "# Nested Claude Rules\n\nThis is nested.";

    // Write root CLAUDE.md
    cagents_core::writers::claude_md::write_claude_md(temp_dir.path(), root_content, true)
        .unwrap();

    // Write nested CLAUDE.md
    let nested_dir = temp_dir.path().join("src");
    fs::create_dir_all(&nested_dir).unwrap();
    cagents_core::writers::claude_md::write_claude_md(&nested_dir, nested_content, false)
        .unwrap();

    // Read root CLAUDE.md - should have footer
    let root_written = fs::read_to_string(temp_dir.path().join("CLAUDE.md")).unwrap();
    assert!(
        root_written.contains("Working with cAGENTS"),
        "Root CLAUDE.md should contain footer"
    );
    assert!(root_written.contains("# Root Claude Rules"));

    // Read nested CLAUDE.md - should NOT have footer
    let nested_written = fs::read_to_string(nested_dir.join("CLAUDE.md")).unwrap();
    assert!(
        !nested_written.contains("Working with cAGENTS"),
        "Nested CLAUDE.md should NOT contain footer"
    );
    assert!(nested_written.contains("# Nested Claude Rules"));
}
