// Test that init creates correct glob patterns to avoid nested AGENTS.md files
use serial_test::serial;
use cagents_core::init::ProjectInfo;
use cagents_core::cmd_build;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use std::env;

#[test]
#[serial]
fn test_init_glob_creates_single_output_per_directory() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create nested structure with AGENTS.md in packages/
    fs::create_dir_all("packages/app1/src").unwrap();
    fs::create_dir_all("packages/app2/lib").unwrap();
    fs::write("AGENTS.md", "# Root").unwrap();
    fs::write("packages/AGENTS.md", "# Packages").unwrap();

    // Add actual files so glob matching works
    fs::write("packages/app1/index.ts", "// app1").unwrap();
    fs::write("packages/app1/src/main.ts", "// main").unwrap();
    fs::write("packages/app2/index.ts", "// app2").unwrap();
    fs::write("packages/app2/lib/util.ts", "// util").unwrap();

    // Create renderer script
    fs::write("render.py", r#"import json, sys
payload = json.load(sys.stdin)
template = payload.get("templateSource", "")
print(json.dumps({"content": template}))
"#).unwrap();

    let info = ProjectInfo {
        name: "test".to_string(),
        owner: None,
        has_git: false,
        has_agents_md: true,
        has_cagents_dir: false,
        has_claude_md: false,
        has_cursorrules: false,
        has_cursor_rules: false,
        agents_md_locations: vec![
            PathBuf::from("AGENTS.md"),
            PathBuf::from("packages/AGENTS.md"),
        ],
    };

    cagents_core::init::migrate_smart(&info, false, false).unwrap();

    // Update config to use test renderer (config now has builtin:simple by default, replace it)
    let config = fs::read_to_string(".cAGENTS/config.toml").unwrap();
    let config_with_test_engine = config.replace("builtin:simple", "command:python3 render.py");
    fs::write(".cAGENTS/config.toml", config_with_test_engine).unwrap();

    // Build
    cmd_build(None, None, None, None, false).unwrap();

    // Debug: Show what was created and what glob pattern was used
    println!("\n=== Glob pattern in template ===");
    let packages_template = fs::read_to_string(".cAGENTS/templates/agents-packages.md").unwrap();
    for line in packages_template.lines() {
        if line.contains("glob") || line.contains("packages") {
            println!("  {}", line);
        }
    }
    println!("\n=== Files created ===");
    for entry in walkdir::WalkDir::new(".").max_depth(4) {
        if let Ok(e) = entry {
            if e.file_name() == "AGENTS.md" {
                println!("  {}", e.path().display());
            }
        }
    }
    println!("====================\n");

    // Verify: With new migration strategy, we use alwaysApply (no globs)
    // This means AGENTS.md is only created at the root after migration
    assert!(PathBuf::from("AGENTS.md").exists(), "Root AGENTS.md should exist");

    // After migration, nested AGENTS.md files are removed and converted to templates with alwaysApply
    // So they don't get recreated in their original locations
    assert!(!PathBuf::from("packages/AGENTS.md").exists(),
        "packages/AGENTS.md should be removed after migration");

    // These should NOT exist either
    assert!(!PathBuf::from("packages/app1/AGENTS.md").exists(),
        "Should NOT create AGENTS.md in packages/app1/");
    assert!(!PathBuf::from("packages/app1/src/AGENTS.md").exists(),
        "Should NOT create AGENTS.md in packages/app1/src/");
    assert!(!PathBuf::from("packages/app2/AGENTS.md").exists(),
        "Should NOT create AGENTS.md in packages/app2/");
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

