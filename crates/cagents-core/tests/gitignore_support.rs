// Test .gitignore support and unlimited depth
use serial_test::serial;

use cagents_core::init::ProjectInfo;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use std::env;

#[test]
#[serial]
fn test_unlimited_depth_detection() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create deeply nested structure (10 levels)
    fs::create_dir_all("a/b/c/d/e/f/g/h/i/j").unwrap();
    fs::write("AGENTS.md", "Root").unwrap();
    fs::write("a/AGENTS.md", "L1").unwrap();
    fs::write("a/b/c/AGENTS.md", "L3").unwrap();
    fs::write("a/b/c/d/e/f/AGENTS.md", "L6").unwrap();
    fs::write("a/b/c/d/e/f/g/h/i/j/AGENTS.md", "L10").unwrap();

    let info = ProjectInfo::detect().unwrap();

    // Should find all files regardless of depth
    assert!(info.agents_md_locations.len() >= 5, "Should find all AGENTS.md files at any depth");
    assert!(info.agents_md_locations.contains(&PathBuf::from("AGENTS.md")));
    assert!(info.agents_md_locations.contains(&PathBuf::from("a/AGENTS.md")));
    assert!(info.agents_md_locations.contains(&PathBuf::from("a/b/c/AGENTS.md")));
    assert!(info.agents_md_locations.contains(&PathBuf::from("a/b/c/d/e/f/AGENTS.md")));
    assert!(info.agents_md_locations.contains(&PathBuf::from("a/b/c/d/e/f/g/h/i/j/AGENTS.md")));
}

#[test]
#[serial]
fn test_respects_gitignore() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Initialize git (required for gitignore to work)
    std::process::Command::new("git").args(["init"]).output().ok();

    // Create .gitignore
    fs::write(".gitignore", "ignored/\n").unwrap();

    // Create files
    fs::create_dir_all("src").unwrap();
    fs::create_dir_all("ignored").unwrap();
    fs::write("AGENTS.md", "Root").unwrap();
    fs::write("src/AGENTS.md", "Src").unwrap();
    fs::write("ignored/AGENTS.md", "Ignored").unwrap();

    let info = ProjectInfo::detect().unwrap();

    // Should find root and src, but NOT ignored
    assert!(info.agents_md_locations.contains(&PathBuf::from("AGENTS.md")));
    assert!(info.agents_md_locations.contains(&PathBuf::from("src/AGENTS.md")));
    assert!(!info.agents_md_locations.contains(&PathBuf::from("ignored/AGENTS.md")),
        "Should not find AGENTS.md in .gitignored directory");
}

#[test]
#[serial]
fn test_respects_gitignore_patterns() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Initialize git
    std::process::Command::new("git").args(["init"]).output().ok();

    // Create .gitignore with pattern
    fs::write(".gitignore", "*.tmp\nbuild/\n").unwrap();

    fs::create_dir_all("src").unwrap();
    fs::create_dir_all("build").unwrap();
    fs::write("AGENTS.md", "Root").unwrap();
    fs::write("src/AGENTS.md", "Src").unwrap();
    fs::write("build/AGENTS.md", "Build").unwrap();

    let info = ProjectInfo::detect().unwrap();

    assert!(info.agents_md_locations.contains(&PathBuf::from("AGENTS.md")));
    assert!(info.agents_md_locations.contains(&PathBuf::from("src/AGENTS.md")));
    assert!(!info.agents_md_locations.contains(&PathBuf::from("build/AGENTS.md")),
        "Should not find AGENTS.md in build/ (gitignored)");
}

#[test]
#[serial]
fn test_respects_nested_gitignore() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Initialize git
    std::process::Command::new("git").args(["init"]).output().ok();

    // Root .gitignore
    fs::write(".gitignore", "*.log\n").unwrap();

    // Nested .gitignore in src/
    fs::create_dir_all("src/private").unwrap();
    fs::write("src/.gitignore", "private/\n").unwrap();

    fs::write("AGENTS.md", "Root").unwrap();
    fs::write("src/AGENTS.md", "Src").unwrap();
    fs::write("src/private/AGENTS.md", "Private").unwrap();

    let info = ProjectInfo::detect().unwrap();

    assert!(info.agents_md_locations.contains(&PathBuf::from("AGENTS.md")));
    assert!(info.agents_md_locations.contains(&PathBuf::from("src/AGENTS.md")));
    // Should respect src/.gitignore
    assert!(!info.agents_md_locations.contains(&PathBuf::from("src/private/AGENTS.md")),
        "Should respect nested .gitignore in src/");
}

#[test]
#[serial]
fn test_cagentsignore_custom_file() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Initialize git
    std::process::Command::new("git").args(["init"]).output().ok();

    // Create .cagentsignore
    fs::write(".cagentsignore", "experimental/\n").unwrap();

    fs::create_dir_all("src").unwrap();
    fs::create_dir_all("experimental").unwrap();
    fs::write("AGENTS.md", "Root").unwrap();
    fs::write("src/AGENTS.md", "Src").unwrap();
    fs::write("experimental/AGENTS.md", "Experimental").unwrap();

    let info = ProjectInfo::detect().unwrap();

    assert!(info.agents_md_locations.contains(&PathBuf::from("AGENTS.md")));
    assert!(info.agents_md_locations.contains(&PathBuf::from("src/AGENTS.md")));
    assert!(!info.agents_md_locations.contains(&PathBuf::from("experimental/AGENTS.md")),
        "Should respect .cagentsignore");
}

#[test]
#[serial]
fn test_no_depth_limit() {
    let tmp = TempDir::new().unwrap();
    let _guard = ChangeDir::new(tmp.path());

    // Create deep nesting (10 levels on Windows due to MAX_PATH, 20 on Unix)
    // Windows has a 260 character path limit by default, and temp dirs can be long
    let (deep_path, expected_depth) = if cfg!(windows) {
        ("a/b/c/d/e/f/g/h/i/j", 10)
    } else {
        ("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t", 20)
    };

    eprintln!("Test environment:");
    eprintln!("  OS: {}", std::env::consts::OS);
    eprintln!("  Temp dir: {:?}", tmp.path());
    eprintln!("  Creating nested path: {}", deep_path);
    eprintln!("  Expected depth: {} levels", expected_depth);

    fs::create_dir_all(deep_path).unwrap();
    fs::write("AGENTS.md", "Root").unwrap();
    fs::write(format!("{}/AGENTS.md", deep_path), "Deep").unwrap();

    let info = ProjectInfo::detect().unwrap();

    eprintln!("Detection results:");
    eprintln!("  Found {} AGENTS.md files", info.agents_md_locations.len());
    for (i, path) in info.agents_md_locations.iter().enumerate() {
        eprintln!("  [{}] {}", i, path.display());
    }

    // Should find both, even at significant depth
    assert_eq!(info.agents_md_locations.len(), 2,
        "Should find both root and nested AGENTS.md files (OS: {}, depth: {} levels)",
        std::env::consts::OS, expected_depth);

    assert!(info.agents_md_locations.contains(&PathBuf::from("AGENTS.md")),
        "Should find root AGENTS.md (found: {:?})", info.agents_md_locations);

    // Check for nested path - need to handle both / and \ path separators
    let has_nested = info.agents_md_locations.iter().any(|p| {
        let path_str = p.to_string_lossy();
        path_str.contains("a/b/c/d") || path_str.contains("a\\b\\c\\d")
    });
    assert!(has_nested,
        "Should find nested AGENTS.md at depth {}+ (OS: {}, found paths: {:?})",
        expected_depth, std::env::consts::OS, info.agents_md_locations);
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
