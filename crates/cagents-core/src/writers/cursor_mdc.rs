// emit .cursor/rules/*.mdc mirroring front-matter and body

use crate::loader::Rule;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Write rules to Cursor .mdc format
pub fn write_cursor_rules(cursor_dir: &Path, rules: &[Rule], rendered_bodies: &[String]) -> Result<()> {
    // Create .cursor/rules directory
    let rules_dir = cursor_dir.join("rules");
    fs::create_dir_all(&rules_dir)
        .with_context(|| format!("Failed to create {}", rules_dir.display()))?;

    // Write each rule that targets cursor
    for (rule, rendered) in rules.iter().zip(rendered_bodies.iter()) {
        // Check if this rule targets cursor
        let targets_cursor = rule.frontmatter.targets.as_ref()
            .map(|t| t.contains(&"cursor".to_string()))
            .unwrap_or(false);

        if !targets_cursor {
            continue;
        }

        // Generate filename
        let name = rule.frontmatter.name.as_deref().unwrap_or("rule");
        let filename = format!("{}.md", name);

        // Cursor .mdc format is just markdown with optional metadata comments
        let mut content = String::new();

        // Add metadata comment if useful
        if let Some(desc) = &rule.frontmatter.description {
            content.push_str(&format!("<!-- Description: {} -->\n\n", desc));
        }

        // Add rendered content
        content.push_str(rendered);

        fs::write(rules_dir.join(&filename), content)
            .with_context(|| format!("Failed to write {}", filename))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::RuleFrontmatter;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_write_cursor_rules() {
        let tmp = TempDir::new().unwrap();

        let rule = Rule {
            frontmatter: RuleFrontmatter {
                name: Some("test-rule".to_string()),
                description: Some("Test description".to_string()),
                targets: Some(vec!["cursor".to_string()]),
                ..Default::default()
            },
            body: "# Test\nContent".to_string(),
            path: PathBuf::from("test.md"),
        };

        let rendered = vec!["# Test\nRendered content".to_string()];

        write_cursor_rules(&tmp.path().join(".cursor"), &[rule], &rendered).unwrap();

        let output = fs::read_to_string(tmp.path().join(".cursor/rules/test-rule.md")).unwrap();
        assert!(output.contains("Test description"));
        assert!(output.contains("Rendered content"));
    }
}
