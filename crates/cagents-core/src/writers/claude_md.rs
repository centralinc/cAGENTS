// Write CLAUDE.md output format

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Write merged content to CLAUDE.md at the specified path
pub fn write_claude_md(output_dir: &Path, content: &str, _is_root: bool) -> Result<()> {
    let output_path = output_dir.join("CLAUDE.md");

    // Prepend auto-update header
    let header = r#"<!--
**IMPORTANT**: This project uses **cAGENTS** to provide generated context and instructions for AI coding agents.
This file is auto-generated. Do not edit it directly.
-->

"#;

    let full_content = format!("{}{}", header, content);

    fs::write(&output_path, full_content)
        .with_context(|| format!("Failed to write CLAUDE.md to {}", output_path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_claude_md() {
        let temp_dir = TempDir::new().unwrap();
        let content = "# Claude Code Rules\n\nTest content.";

        write_claude_md(temp_dir.path(), content, true).unwrap();

        let written = fs::read_to_string(temp_dir.path().join("CLAUDE.md")).unwrap();

        // Should include auto-update header
        assert!(written.contains("**IMPORTANT**: This project uses **cAGENTS**"));
        assert!(written.contains("This file is auto-generated. Do not edit it directly."));
        // Should include original content
        assert!(written.contains("# Claude Code Rules"));
        assert!(written.contains("Test content."));
    }
}
