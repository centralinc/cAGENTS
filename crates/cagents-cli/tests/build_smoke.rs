use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_build_creates_agents_md() {
    // Create a temporary copy of examples/basic
    let temp = assert_fs::TempDir::new().unwrap();
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let basic_src = workspace_root.join("examples/basic");

    // Copy cAGENTS directory
    let cagents_src = basic_src.join(".cAGENTS");
    let cagents_dst = temp.path().join(".cAGENTS");
    copy_dir_recursive(&cagents_src, &cagents_dst).expect("Failed to copy cAGENTS");

    // Copy source files so glob-based templates have files to match
    let src_dir = basic_src.join("src");
    if src_dir.exists() {
        let dst_src = temp.path().join("src");
        fs::create_dir_all(&dst_src).unwrap();
        for entry in fs::read_dir(&src_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                fs::copy(entry.path(), dst_src.join(entry.file_name())).unwrap();
            }
        }
    }

    let tests_dir = basic_src.join("tests");
    if tests_dir.exists() {
        let dst_tests = temp.path().join("tests");
        fs::create_dir_all(&dst_tests).unwrap();
        for entry in fs::read_dir(&tests_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                fs::copy(entry.path(), dst_tests.join(entry.file_name())).unwrap();
            }
        }
    }

    // Run cagents build
    let mut cmd = Command::cargo_bin("cagents").unwrap();
    cmd.current_dir(temp.path())
        .arg("build")
        .assert()
        .success();

    // Check that AGENTS.md was created
    let agents_md = temp.child("AGENTS.md");
    agents_md.assert(predicate::path::exists());

    // Read and verify content
    let content = fs::read_to_string(agents_md.path()).unwrap();

    // Should contain content from project-meta template
    assert!(content.contains("Project Context"), "Should contain Project Context section");
    assert!(content.contains("Communication Style"), "Should contain template content");

    // Snapshot test for exact format
    insta::assert_snapshot!(content);
}

// Helper to recursively copy directories
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
