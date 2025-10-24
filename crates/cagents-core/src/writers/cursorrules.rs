// Write .cursorrules output format

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Write merged content to .cursorrules at the specified path
pub fn write_cursorrules(output_dir: &Path, content: &str) -> Result<()> {
    let output_path = output_dir.join(".cursorrules");

    // Prepend auto-update header
    let header = r#"# IMPORTANT: This project uses cAGENTS to provide generated context and instructions for AI coding agents.
# This file is auto-generated. Do not edit it directly.

"#;

    let full_content = format!("{}{}", header, content);

    fs::write(&output_path, full_content)
        .with_context(|| format!("Failed to write .cursorrules to {}", output_path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_cursorrules() {
        let temp_dir = TempDir::new().unwrap();
        let content = "# Cursor Rules\n\nTest content.";

        write_cursorrules(temp_dir.path(), content).unwrap();

        let written = fs::read_to_string(temp_dir.path().join(".cursorrules")).unwrap();

        // Should include auto-update header
        assert!(written.contains("IMPORTANT: This project uses cAGENTS"));
        assert!(written.contains("This file is auto-generated. Do not edit it directly."));
        // Should include original content
        assert!(written.contains("# Cursor Rules"));
        assert!(written.contains("Test content."));
    }
}
