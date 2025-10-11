// discover templates, parse front-matter (YAML), return in-memory rules

use crate::model::{ProjectConfig, RuleFrontmatter};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a rule with its parsed frontmatter, body, and source path
#[derive(Debug, Clone)]
pub struct Rule {
    pub frontmatter: RuleFrontmatter,
    pub body: String,
    pub path: PathBuf,
}

/// Load project config from a TOML file
pub fn load_config(config_path: &Path) -> Result<ProjectConfig> {
    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let config: ProjectConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    Ok(config)
}

/// Discover all rule files in the templates directory
pub fn discover_rules(config: &ProjectConfig, base_dir: &Path) -> Result<Vec<Rule>> {
    let templates_dir = base_dir.join(&config.paths.templates_dir);

    if !templates_dir.exists() {
        anyhow::bail!(
            "Templates directory does not exist: {}",
            templates_dir.display()
        );
    }

    let mut rules = Vec::new();

    // Read all .md files in the templates directory
    for entry in fs::read_dir(&templates_dir)
        .with_context(|| format!("Failed to read templates dir: {}", templates_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        // Check if filename ends with .md (handles .hbs.md, .liquid.md, .j2.md, etc.)
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.ends_with(".md") {
                    let rule = parse_rule_file(&path)?;
                    rules.push(rule);
                }
            }
        }
    }

    // Sort by order (lower numbers first)
    rules.sort_by_key(|r| r.frontmatter.order.unwrap_or(50));

    Ok(rules)
}

/// Parse a single rule file, extracting YAML frontmatter and body
fn parse_rule_file(path: &Path) -> Result<Rule> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read rule file: {}", path.display()))?;

    let (frontmatter, body) = split_frontmatter(&content)?;

    let frontmatter: RuleFrontmatter = serde_yaml::from_str(&frontmatter)
        .with_context(|| format!("Failed to parse frontmatter in: {}", path.display()))?;

    Ok(Rule {
        frontmatter,
        body,
        path: path.to_path_buf(),
    })
}

/// Split a document into YAML frontmatter and body
/// Expects frontmatter to be surrounded by --- markers
fn split_frontmatter(content: &str) -> Result<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0].trim() != "---" {
        anyhow::bail!("Missing frontmatter delimiter at start");
    }

    // Find the closing ---
    let end_idx = lines
        .iter()
        .skip(1)
        .position(|line| line.trim() == "---")
        .context("Missing closing frontmatter delimiter")?
        + 1;

    let frontmatter = lines[1..end_idx].join("\n");
    let body = if end_idx + 1 < lines.len() {
        lines[end_idx + 1..].join("\n")
    } else {
        String::new()
    };

    Ok((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter() {
        let content = r#"---
name: test
order: 10
---
## Body

Content here"#;

        let (fm, body) = split_frontmatter(content).unwrap();
        assert!(fm.contains("name: test"));
        assert!(body.contains("## Body"));
    }
}
