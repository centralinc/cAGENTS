// Test git ignore handling for multiple output types

use serial_test::serial;
use std::fs;
use tempfile::TempDir;
use std::env;

#[test]
#[serial]
fn test_ignore_outputs_with_config() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create .cAGENTS with output targets config
    fs::create_dir_all(".cAGENTS").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[output]
targets = ["agents-md", "claude-md", "cursorrules"]
"#).unwrap();

    // Run ignore_outputs
    cagents_core::helpers::git::ignore_outputs().unwrap();

    let gitignore = fs::read_to_string(".gitignore").unwrap();

    // Should add patterns for all configured output types
    assert!(gitignore.contains("AGENTS.md"), "Should ignore AGENTS.md");
    assert!(gitignore.contains("CLAUDE.md"), "Should ignore CLAUDE.md");
    assert!(gitignore.contains(".cursorrules"), "Should ignore .cursorrules");
}

#[test]
#[serial]
fn test_ignore_outputs_defaults_to_agents_md_only() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create .cAGENTS with no output.targets
    fs::create_dir_all(".cAGENTS").unwrap();
    fs::write(".cAGENTS/config.toml", r#"
[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"
"#).unwrap();

    // Run ignore_outputs
    cagents_core::helpers::git::ignore_outputs().unwrap();

    let gitignore = fs::read_to_string(".gitignore").unwrap();

    // Should only add AGENTS.md by default
    assert!(gitignore.contains("AGENTS.md"), "Should ignore AGENTS.md");
    assert!(!gitignore.contains("CLAUDE.md"), "Should NOT ignore CLAUDE.md if not configured");
    assert!(!gitignore.contains(".cursorrules"), "Should NOT ignore .cursorrules if not configured");
}

#[test]
#[serial]
fn test_unignore_outputs_removes_all_types() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create .gitignore with all output types
    fs::write(".gitignore", r#"node_modules
target

# cAGENTS generated files
AGENTS.md
**/AGENTS.md
CLAUDE.md
**/CLAUDE.md
.cursorrules
**/.cursorrules
"#).unwrap();

    // Run unignore_outputs
    cagents_core::helpers::git::unignore_outputs().unwrap();

    let gitignore = fs::read_to_string(".gitignore").unwrap();

    // Should remove all output patterns
    assert!(!gitignore.contains("AGENTS.md"));
    assert!(!gitignore.contains("CLAUDE.md"));
    assert!(!gitignore.contains(".cursorrules"));
    assert!(!gitignore.contains("cAGENTS generated"));

    // Should preserve other patterns
    assert!(gitignore.contains("node_modules"));
    assert!(gitignore.contains("target"));
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
