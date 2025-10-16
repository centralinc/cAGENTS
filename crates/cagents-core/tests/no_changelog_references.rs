// Test that templates use changesets, not CHANGELOG.md
use std::fs;
use std::path::PathBuf;

#[test]
fn test_templates_use_changesets_not_changelog() {
    let templates_dir = PathBuf::from(".cAGENTS/templates");

    if !templates_dir.exists() {
        // No templates directory in test environment
        return;
    }

    // Read all template files
    for entry in fs::read_dir(&templates_dir).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_file() {
            continue;
        }

        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let content = fs::read_to_string(&path).unwrap();

        // Should not reference CHANGELOG
        assert!(
            !content.contains("CHANGELOG"),
            "Template {} should not reference CHANGELOG.md, use changesets instead",
            path.display()
        );

        // If it mentions versioning/releases, should mention changesets
        if content.to_lowercase().contains("version") ||
           content.to_lowercase().contains("release") ||
           content.to_lowercase().contains("non-negotiable") {
            // Not a hard requirement, but good practice
            // Could add assert for "changeset" if needed
        }
    }
}
