// Project initialization and AGENTS.md migration

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Detect project information from git and filesystem
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub owner: Option<String>,
    pub has_git: bool,
    pub has_agents_md: bool,
    pub has_cagents_dir: bool,
    pub has_claude_md: bool,
    pub has_cursorrules: bool,
    pub has_cursor_rules: bool,
    pub agents_md_locations: Vec<PathBuf>,
}

impl ProjectInfo {
    /// Detect project information from current directory
    pub fn detect() -> Result<Self> {
        let name = detect_project_name()?;
        let owner = detect_owner().ok();
        let has_git = PathBuf::from(".git").exists();
        let has_agents_md = PathBuf::from("AGENTS.md").exists();
        let has_cagents_dir = PathBuf::from(".cAGENTS").exists();
        let has_claude_md = PathBuf::from("CLAUDE.md").exists();
        let has_cursorrules = PathBuf::from(".cursorrules").exists();
        let cursor_rules_path = PathBuf::from(".cursor/rules");
        let has_cursor_rules = cursor_rules_path.exists() && cursor_rules_path.is_dir();
        let agents_md_locations = find_all_agents_md();

        Ok(Self {
            name,
            owner,
            has_git,
            has_agents_md,
            has_cagents_dir,
            has_claude_md,
            has_cursorrules,
            has_cursor_rules,
            agents_md_locations,
        })
    }
}

/// Detect project name from various sources
fn detect_project_name() -> Result<String> {
    // 1. Try git remote
    if let Ok(output) = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
    {
        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout);
            if let Some(name) = extract_repo_name(&url) {
                return Ok(name);
            }
        }
    }

    // 2. Try package.json
    if let Ok(content) = fs::read_to_string("package.json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                return Ok(name.to_string());
            }
        }
    }

    // 3. Try Cargo.toml
    if let Ok(content) = fs::read_to_string("Cargo.toml") {
        if let Ok(cargo) = toml::from_str::<toml::Value>(&content) {
            if let Some(name) = cargo.get("package")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
            {
                return Ok(name.to_string());
            }
        }
    }

    // 4. Use current directory name
    let cwd = std::env::current_dir()?;
    let dir_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project");

    Ok(dir_name.to_string())
}

/// Extract repository name from git URL
fn extract_repo_name(url: &str) -> Option<String> {
    let url = url.trim();

    // Handle git@github.com:user/repo.git
    if let Some(idx) = url.rfind('/') {
        let name = &url[idx + 1..];
        let name = name.trim_end_matches(".git");
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }

    None
}

/// Detect owner from git config
fn detect_owner() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout);
        Ok(name.trim().to_string())
    } else {
        anyhow::bail!("Failed to get git user.name")
    }
}

/// Initialize a fresh cAGENTS project with basic preset
pub fn init_basic(info: &ProjectInfo, force: bool, _backup: bool) -> Result<()> {
    // 1. Check if cAGENTS already exists
    if info.has_cagents_dir && !force {
        anyhow::bail!(
            ".cAGENTS/ already exists. Use --force to overwrite, or run 'cagents build' to use existing setup."
        );
    }

    // 2. Create directory structure
    let cagents_dir = PathBuf::from(".cAGENTS");
    let templates_dir = cagents_dir.join("templates");

    fs::create_dir_all(&templates_dir)
        .context("Failed to create .cAGENTS/templates directory")?;

    // 3. Generate minimal config.toml
    let config_content = generate_minimal_config(info);
    fs::write(cagents_dir.join("config.toml"), config_content)
        .context("Failed to write config.toml")?;

    // 4. Generate .gitignore
    let gitignore_content = "# cAGENTS local config\nconfig.local.toml\n**.local.*\n.output-cache\n";
    fs::write(cagents_dir.join(".gitignore"), gitignore_content)
        .context("Failed to write .gitignore")?;

    use owo_colors::OwoColorize;

    println!("✓ cAGENTS initialized!");
    println!();
    println!("Created:");
    println!("  .cAGENTS/config.toml");
    println!("  .cAGENTS/templates/ (empty)");
    println!("  .cAGENTS/.gitignore");
    println!();
    println!("Next steps:");
    println!("  1. Create templates in .cAGENTS/templates/");
    println!("  2. Or migrate existing rules: {}", "cagents migrate".bright_white());
    println!("  3. Run: {}", "cagents build".bright_white());

    Ok(())
}

/// Generate basic config.toml (used by migration)
fn generate_basic_config(info: &ProjectInfo) -> String {
    let owner = info.owner.as_deref().unwrap_or("Your Name");

    format!(r#"[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[variables.static]
project = "{}"
owner = "{}"

[variables.command]
branch = "git rev-parse --abbrev-ref HEAD"

[execution]
shell = "bash"
timeoutMs = 3000
"#, info.name, owner)
}

/// Generate minimal config.toml (for clean init)
fn generate_minimal_config(_info: &ProjectInfo) -> String {
    r#"[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[variables.static]

[execution]
shell = "bash"
timeoutMs = 3000
"#.to_string()
}

/// Find all AGENTS.md files in the project
/// Respects .gitignore and has no depth limit
fn find_all_agents_md() -> Vec<PathBuf> {
    use ignore::WalkBuilder;

    let mut locations = Vec::new();

    // Use ignore crate which respects .gitignore
    let walker = WalkBuilder::new(".")
        .hidden(false) // Don't auto-skip hidden files (let .gitignore decide)
        .git_ignore(true) // Respect .gitignore
        .git_global(true) // Respect global gitignore
        .git_exclude(true) // Respect .git/info/exclude
        .add_custom_ignore_filename(".cagentsignore") // Custom ignore file
        .filter_entry(|e| {
            // Keep root
            if e.path() == std::path::Path::new(".") {
                return true;
            }

            let name = e.file_name().to_string_lossy();
            // Only skip our own directory and common build dirs
            name != ".cAGENTS"
                && name != "node_modules"
                && name != "target"
                && name != "dist"
                && !name.starts_with('.') // Skip hidden dirs
        })
        .build();

    for entry in walker.flatten() {
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
            && entry.file_name() == "AGENTS.md"
        {
            if let Ok(rel_path) = entry.path().strip_prefix(".") {
                locations.push(rel_path.to_path_buf());
            }
        }
    }

    locations
}

/// Represents a parsed section from AGENTS.md
#[derive(Debug, Clone)]
pub struct AgentsSection {
    pub heading: String,
    pub level: usize,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Parse AGENTS.md into sections
pub fn parse_agents_md(content: &str) -> Vec<AgentsSection> {
    let mut sections = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut current_heading = String::from("_preamble");
    let mut current_level = 0;
    let mut current_content = String::new();
    let mut current_start = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with('#') {
            // Save previous section
            if !current_content.is_empty() || !sections.is_empty() {
                sections.push(AgentsSection {
                    heading: current_heading.clone(),
                    level: current_level,
                    content: current_content.trim().to_string(),
                    start_line: current_start,
                    end_line: i,
                });
            }

            // Start new section
            let level = line.chars().take_while(|&c| c == '#').count();
            current_heading = line.to_string();
            current_level = level;
            current_content = line.to_string() + "\n";
            current_start = i;
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last section
    if !current_content.is_empty() {
        sections.push(AgentsSection {
            heading: current_heading,
            level: current_level,
            content: current_content.trim().to_string(),
            start_line: current_start,
            end_line: lines.len(),
        });
    }

    sections
}

/// Backup AGENTS.md before migration
pub fn backup_agents_md(should_backup: bool) -> Result<()> {
    if !should_backup {
        return Ok(()); // No backup requested
    }

    let source = PathBuf::from("AGENTS.md");
    let backup = PathBuf::from("AGENTS.md.backup");

    if !source.exists() {
        return Ok(()); // Nothing to backup
    }

    fs::copy(&source, &backup)
        .with_context(|| "Failed to backup AGENTS.md")?;

    println!("▸ Backed up: AGENTS.md → AGENTS.md.backup");
    Ok(())
}

/// Simple migration: convert entire AGENTS.md to a single template
pub fn migrate_simple(info: &ProjectInfo, force: bool, backup: bool) -> Result<()> {
    if info.has_cagents_dir && !force {
        anyhow::bail!(".cAGENTS/ already exists. Use --force to overwrite.");
    }

    if !info.has_agents_md {
        anyhow::bail!("No AGENTS.md found to migrate.");
    }

    // Backup original (if requested)
    backup_agents_md(backup)?;

    // Read existing AGENTS.md
    let content = fs::read_to_string("AGENTS.md")?;

    // Create structure
    let cagents_dir = PathBuf::from(".cAGENTS");
    let templates_dir = cagents_dir.join("templates");
    fs::create_dir_all(&templates_dir)?;

    // Generate config
    let config = generate_basic_config(info);
    fs::write(cagents_dir.join("config.toml"), config)?;

    // Convert AGENTS.md to template (new naming)
    let template = format!(r#"---
name: agents-root
description: Migrated from existing AGENTS.md
order: 1
---
{}
"#, content);

    fs::write(templates_dir.join("agents-root.md"), template)?;
    fs::write(cagents_dir.join(".gitignore"), "config.local.toml\n**.local.*\n.output-cache\n")?;

    println!("✓ Migrated AGENTS.md to cAGENTS!");
    println!();
    println!("Created:");
    println!("  .cAGENTS/config.toml");
    println!("  .cAGENTS/templates/agents-root.md");
    println!();

    // Remove original AGENTS.md file after successful migration
    if backup {
        println!("  ✓ Backed up AGENTS.md → AGENTS.md.backup");
    }
    fs::remove_file("AGENTS.md").ok(); // Ignore error if already deleted
    println!("  ✓ Removed original AGENTS.md");
    println!();

    // Auto-run build
    println!("▸ Running initial build...");
    println!();
    if let Err(e) = crate::cmd_build(None, false) {
        println!("⚠ Initial build failed: {}", e);
        println!();
        println!("Next steps:");
        println!("  1. Configure engine in .cAGENTS/config.toml");
        println!("  2. Run: cagents build");
    } else {
        println!("✓ AGENTS.md regenerated from template!");
        println!();
        println!("Next steps:");
        println!("  1. Compare: diff AGENTS.md.backup AGENTS.md");
        println!("  2. Edit templates to add variables and split sections");
    }

    Ok(())
}


/// Smart migration: split sections and suggest templates
pub fn migrate_smart(info: &ProjectInfo, force: bool, backup: bool) -> Result<()> {
    if info.has_cagents_dir && !force {
        anyhow::bail!(".cAGENTS/ already exists. Use --force to overwrite.");
    }

    if !info.has_agents_md {
        anyhow::bail!("No AGENTS.md found to migrate.");
    }

    // M3 Slice 2: Handle multiple AGENTS.md files
    if info.agents_md_locations.len() > 1 {
        println!("▸ Found {} AGENTS.md files:", info.agents_md_locations.len());
        for loc in &info.agents_md_locations {
            println!("   - {}", loc.display());
        }
        println!();
        return migrate_multiple_agents_md(info, force, backup);
    }

    // Simply migrate the existing AGENTS.md as-is
    // Users can manually organize into multiple templates after migration if needed
    migrate_simple(info, force, backup)?;

    Ok(())
}

/// Migrate multiple AGENTS.md files found in nested directories
fn migrate_multiple_agents_md(info: &ProjectInfo, force: bool, backup: bool) -> Result<()> {
    if info.has_cagents_dir && !force {
        anyhow::bail!(".cAGENTS/ already exists. Use --force to overwrite.");
    }

    println!("▸ Migrating {} AGENTS.md files...", info.agents_md_locations.len());
    println!();

    // Create structure
    let cagents_dir = PathBuf::from(".cAGENTS");
    let templates_dir = cagents_dir.join("templates");
    fs::create_dir_all(&templates_dir)?;

    // Generate config
    let config = generate_basic_config(info);
    fs::write(cagents_dir.join("config.toml"), config)?;

    // Backup and convert each AGENTS.md file
    let mut created_templates = Vec::new();

    for (idx, location) in info.agents_md_locations.iter().enumerate() {
        // Backup (if requested)
        if backup {
            let backup_path = location.with_extension("md.backup");
            if let Err(e) = fs::copy(location, &backup_path) {
                eprintln!("▸  Failed to backup {}: {}", location.display(), e);
            } else {
                println!("▸ Backed up: {} → {}", location.display(), backup_path.display());
            }
        }

        // Read content
        let content = fs::read_to_string(location)
            .with_context(|| format!("Failed to read {}", location.display()))?;

        // Generate template name from location (new naming)
        let template_name = if location == &PathBuf::from("AGENTS.md") {
            "agents-root".to_string()
        } else {
            location
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| format!("agents-{}", s))
                .unwrap_or_else(|| format!("agents-{}", idx))
        };

        // Generate glob pattern from location
        // For migrated files, don't use globs to avoid nested outputs
        // The user can manually add globs after migration if they want directory-specific rules
        let glob_pattern: Option<Vec<String>> = None;

        // Create template
        let mut frontmatter = format!(
            "name: {}\ndescription: Migrated from {}\n",
            template_name,
            location.display()
        );

        if let Some(globs) = glob_pattern {
            frontmatter.push_str("globs:\n");
            for glob in &globs {
                frontmatter.push_str(&format!("  - \"{}\"\n", glob));
            }
        }
        // Note: No when clause = implicitly always apply everywhere

        frontmatter.push_str(&format!("order: {}\n", (idx + 1) * 10));

        let template_content = format!("---\n{}---\n{}", frontmatter, content);
        let template_filename = format!("{}.md", template_name);

        fs::write(templates_dir.join(&template_filename), template_content)?;
        created_templates.push(template_filename);

        println!("   ✓ Created template: {} (from {})", template_name, location.display());
    }

    fs::write(cagents_dir.join(".gitignore"), "config.local.toml\n**.local.*\n.output-cache\n")?;

    // Remove original AGENTS.md files after successful migration
    for location in &info.agents_md_locations {
        if backup {
            let backup_path = location.with_extension("md.backup");
            println!("  ✓ Backed up {} → {}", location.display(), backup_path.display());
        }
        fs::remove_file(location).ok(); // Ignore errors
    }

    println!();
    println!("✓ Migrated {} AGENTS.md files!", info.agents_md_locations.len());
    println!("  ✓ Removed {} original AGENTS.md file(s)", info.agents_md_locations.len());
    println!();
    println!("Created:");
    println!("  .cAGENTS/config.toml");
    for template in &created_templates {
        println!("  .cAGENTS/templates/{}", template);
    }
    println!();
    println!("Next steps:");
    println!("  1. Run: cagents build");
    println!("  2. Compare outputs with originals");
    println!("  3. Edit templates to add variables");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repo_name() {
        assert_eq!(
            extract_repo_name("git@github.com:user/my-repo.git"),
            Some("my-repo".to_string())
        );
        assert_eq!(
            extract_repo_name("https://github.com/user/my-repo.git"),
            Some("my-repo".to_string())
        );
        assert_eq!(
            extract_repo_name("https://github.com/user/my-repo"),
            Some("my-repo".to_string())
        );
    }

    #[test]
    fn test_generate_basic_config() {
        let info = ProjectInfo {
            name: "test-project".to_string(),
            owner: Some("Alice".to_string()),
            has_git: true,
            has_agents_md: false,
            has_cagents_dir: false,
            has_claude_md: false,
            has_cursorrules: false,
            has_cursor_rules: false,
            agents_md_locations: vec![],
        };

        let config = generate_basic_config(&info);
        assert!(config.contains("test-project"));
        assert!(config.contains("Alice"));
        assert!(config.contains("[paths]"));
        assert!(config.contains("[variables.static]"));
    }
}
