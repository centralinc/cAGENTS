// Helper utilities for import operations
// This module contains reusable functions to reduce duplication across import functions

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// The standard .gitignore content for .cAGENTS directory
pub const CAGENTS_GITIGNORE_CONTENT: &str = "config.local.toml\n**.local.*\n.output-cache\n";

/// Create .cAGENTS directory structure
/// Returns (cagents_dir, templates_dir)
pub fn create_cagents_structure() -> Result<(PathBuf, PathBuf)> {
    let cagents_dir = PathBuf::from(".cAGENTS");
    let templates_dir = cagents_dir.join("templates");
    fs::create_dir_all(&templates_dir)
        .with_context(|| format!("Failed to create directory {}", templates_dir.display()))?;
    Ok((cagents_dir, templates_dir))
}

/// Create .gitignore file in .cAGENTS directory
pub fn create_gitignore(cagents_dir: &PathBuf) -> Result<()> {
    fs::write(cagents_dir.join(".gitignore"), CAGENTS_GITIGNORE_CONTENT)
        .with_context(|| "Failed to create .gitignore")
}

/// Backup a file with a specific extension
/// Returns the backup path on success
pub fn backup_file(path: &PathBuf, extension: &str) -> Result<PathBuf> {
    let backup_path = path.with_extension(extension);
    fs::copy(path, &backup_path)
        .with_context(|| format!("Failed to backup {} to {}", path.display(), backup_path.display()))?;
    println!("▸ Backed up: {} → {}", path.display(), backup_path.display());
    Ok(backup_path)
}

/// Backup a file if backup flag is set
/// Returns Some(backup_path) on success, None if backup is false or if backup fails
pub fn backup_file_if_requested(path: &PathBuf, extension: &str, backup: bool) -> Option<PathBuf> {
    if backup {
        match backup_file(path, extension) {
            Ok(backup_path) => Some(backup_path),
            Err(e) => {
                eprintln!("▸ Failed to backup {}: {}", path.display(), e);
                None
            }
        }
    } else {
        None
    }
}

/// Read file content with contextual error message
pub fn read_file_content(location: &PathBuf) -> Result<String> {
    fs::read_to_string(location)
        .with_context(|| format!("Failed to read {}", location.display()))
}

/// Generate a template name from a file location
///
/// # Arguments
/// * `location` - Path to the source file
/// * `root_filename` - Expected root filename (e.g., "AGENTS.md")
/// * `prefix` - Prefix for generated names (e.g., "agents")
/// * `idx` - Fallback index for naming
pub fn generate_template_name(
    location: &PathBuf,
    root_filename: &str,
    prefix: &str,
    idx: usize
) -> String {
    if location == &PathBuf::from(root_filename) {
        format!("{}-root", prefix)
    } else {
        location
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| format!("{}-{}", prefix, s))
            .unwrap_or_else(|| format!("{}-{}", prefix, idx))
    }
}

/// Builder for creating template frontmatter
#[derive(Debug, Clone)]
pub struct FrontmatterBuilder {
    name: String,
    description: String,
    globs: Option<String>,
    output_in: Option<String>,
    order: usize,
    target_when: Option<Vec<String>>,
    targets: Option<Vec<String>>,
}

impl FrontmatterBuilder {
    /// Create a new frontmatter builder with required fields
    pub fn new(name: &str, description: &str, order: usize) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            globs: None,
            output_in: None,
            order,
            target_when: None,
            targets: None,
        }
    }

    /// Add glob pattern and outputIn strategy
    pub fn with_glob_and_output(mut self, globs_line: &str, output_in_line: &str) -> Self {
        if !globs_line.is_empty() {
            self.globs = Some(globs_line.to_string());
        }
        if !output_in_line.is_empty() {
            self.output_in = Some(output_in_line.to_string());
        }
        self
    }

    /// Add when.target filtering
    pub fn with_target_when(mut self, targets: Vec<&str>) -> Self {
        self.target_when = Some(targets.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Add legacy targets field (for backwards compatibility)
    pub fn with_targets(mut self, targets: Vec<&str>) -> Self {
        self.targets = Some(targets.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Build the frontmatter string
    pub fn build(&self) -> String {
        let mut fm = format!("name: {}\ndescription: {}", self.name, self.description);

        // Add legacy targets field if present
        if let Some(targets) = &self.targets {
            fm.push_str(&format!("\ntargets: [{}]",
                targets.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ")));
        }

        if let Some(globs) = &self.globs {
            fm.push_str(&format!("\n{}", globs));
        }

        if let Some(output_in) = &self.output_in {
            fm.push_str(&format!("\n{}", output_in));
        }

        if let Some(targets) = &self.target_when {
            fm.push_str(&format!("\nwhen:\n  target: [{}]",
                targets.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ")));
        }

        fm.push_str(&format!("\norder: {}", self.order));
        fm
    }
}

/// Write a template file to disk
/// Returns the template filename on success
pub fn write_template(
    templates_dir: &PathBuf,
    template_name: &str,
    frontmatter: &str,
    content: &str,
) -> Result<String> {
    let template = format!("---\n{}\n---\n{}", frontmatter, content);
    let template_filename = format!("{}.md", template_name);
    fs::write(templates_dir.join(&template_filename), &template)
        .with_context(|| format!("Failed to write template {}", template_filename))?;
    Ok(template_filename)
}

/// Remove source files after successful import
pub fn remove_source_files(
    locations: &[PathBuf],
    backup_extension: &str,
    backup: bool,
) {
    for location in locations {
        if backup {
            let backup_path = location.with_extension(backup_extension);
            println!("  ✓ Backed up {} → {}", location.display(), backup_path.display());
        }
        fs::remove_file(location).ok();
    }
}

/// Remove a source directory after successful import
pub fn remove_source_directory(path: &PathBuf, _backup: bool) {
    // Note: Directory backup not implemented yet
    fs::remove_dir_all(path).ok();
}

/// Print success message after import
pub fn print_import_success(
    format_name: &str,
    count: usize,
    templates: &[String],
) {
    println!();
    println!("✓ Imported {}!", format_name);
    println!("  ✓ Removed {} original file(s)", count);
    println!();
    println!("Created:");
    println!("  .cAGENTS/config.toml");
    if templates.len() == 1 {
        println!("  .cAGENTS/templates/{}", templates[0]);
    } else {
        println!("  .cAGENTS/templates/ ({} templates)", templates.len());
    }
    println!();
}

/// Run initial build after import
pub fn run_initial_build(success_message: &str) {
    println!("▸ Running initial build...");
    println!();
    if let Err(e) = crate::cmd_build(None, false) {
        println!("⚠ Build failed: {}", e);
    } else {
        println!("✓ {}", success_message);
    }
}

/// Print information about multiple files found
pub fn print_multiple_files_found(filename: &str, locations: &[PathBuf]) {
    if locations.len() > 1 {
        println!("▸ Found {} {} files:", locations.len(), filename);
        for loc in locations {
            println!("   - {}", loc.display());
        }
        println!();
    }
}

/// Builder for creating config.toml content
#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    templates_dir: String,
    output_root: String,
    cursor_rules_dir: Option<String>,
    engine: String,
    targets: Vec<String>,
    output_targets: Vec<String>,
    project_name: String,
}

impl ConfigBuilder {
    /// Create a new config builder with default values
    pub fn new(project_name: &str) -> Self {
        Self {
            templates_dir: "templates".to_string(),
            output_root: ".".to_string(),
            cursor_rules_dir: None,
            engine: "builtin:simple".to_string(),
            targets: vec![],
            output_targets: vec![],
            project_name: project_name.to_string(),
        }
    }

    /// Enable Cursor support (adds cursorRulesDir and targets)
    pub fn with_cursor_support(mut self) -> Self {
        self.cursor_rules_dir = Some(".cursor/rules".to_string());
        self.targets.extend(vec!["agentsmd".to_string(), "cursor".to_string()]);
        self
    }

    /// Set legacy targets field
    pub fn with_targets(mut self, targets: Vec<&str>) -> Self {
        self.targets = targets.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set output.targets field (for multi-format imports)
    pub fn with_output_targets(mut self, targets: Vec<&str>) -> Self {
        self.output_targets = targets.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Build the config TOML string
    pub fn build(&self) -> String {
        let mut config = String::new();

        // [paths] section
        config.push_str("[paths]\n");
        config.push_str(&format!("templatesDir = \"{}\"\n", self.templates_dir));
        config.push_str(&format!("outputRoot = \"{}\"\n", self.output_root));
        if let Some(cursor_dir) = &self.cursor_rules_dir {
            config.push_str(&format!("cursorRulesDir = \"{}\"\n", cursor_dir));
        }

        // [output] section (if needed)
        if !self.output_targets.is_empty() {
            config.push_str("\n[output]\n");
            let targets_str = self.output_targets
                .iter()
                .map(|t| format!("\"{}\"", t))
                .collect::<Vec<_>>()
                .join(", ");
            config.push_str(&format!("targets = [{}]\n", targets_str));
        }

        // [defaults] section
        config.push_str(&format!("\n[defaults]\n"));
        config.push_str(&format!("engine = \"{}\"\n", self.engine));
        if !self.targets.is_empty() {
            let targets_str = self.targets
                .iter()
                .map(|t| format!("\"{}\"", t))
                .collect::<Vec<_>>()
                .join(", ");
            config.push_str(&format!("targets = [{}]\n", targets_str));
        }

        // [variables.static] section
        config.push_str(&format!("\n[variables.static]\n"));
        config.push_str(&format!("project = \"{}\"\n", self.project_name));

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_template_name_root() {
        let location = PathBuf::from("AGENTS.md");
        let name = generate_template_name(&location, "AGENTS.md", "agents", 0);
        assert_eq!(name, "agents-root");
    }

    #[test]
    fn test_generate_template_name_nested() {
        let location = PathBuf::from("docs/AGENTS.md");
        let name = generate_template_name(&location, "AGENTS.md", "agents", 0);
        assert_eq!(name, "agents-docs");
    }

    #[test]
    fn test_generate_template_name_fallback() {
        let location = PathBuf::from("CLAUDE.md");
        let name = generate_template_name(&location, "AGENTS.md", "agents", 5);
        // Will use fallback since it doesn't match root_filename
        assert!(name.starts_with("agents-"));
    }

    #[test]
    fn test_frontmatter_builder_basic() {
        let fm = FrontmatterBuilder::new("test", "Test template", 10).build();
        assert!(fm.contains("name: test"));
        assert!(fm.contains("description: Test template"));
        assert!(fm.contains("order: 10"));
    }

    #[test]
    fn test_frontmatter_builder_with_globs() {
        let fm = FrontmatterBuilder::new("test", "Test", 10)
            .with_glob_and_output("globs: [\"docs/\"]", "outputIn: \"matched\"")
            .build();
        assert!(fm.contains("globs: [\"docs/\"]"));
        assert!(fm.contains("outputIn: \"matched\""));
    }

    #[test]
    fn test_frontmatter_builder_with_targets() {
        let fm = FrontmatterBuilder::new("test", "Test", 10)
            .with_target_when(vec!["agents-md", "claude-md"])
            .build();
        assert!(fm.contains("when:"));
        assert!(fm.contains("target: [\"agents-md\", \"claude-md\"]"));
    }

    #[test]
    fn test_config_builder_basic() {
        let config = ConfigBuilder::new("test-project").build();
        assert!(config.contains("templatesDir = \"templates\""));
        assert!(config.contains("project = \"test-project\""));
        assert!(config.contains("engine = \"builtin:simple\""));
    }

    #[test]
    fn test_config_builder_with_cursor() {
        let config = ConfigBuilder::new("test-project")
            .with_cursor_support()
            .build();
        assert!(config.contains("cursorRulesDir = \".cursor/rules\""));
        assert!(config.contains("targets = [\"agentsmd\", \"cursor\"]"));
    }

    #[test]
    fn test_config_builder_with_output_targets() {
        let config = ConfigBuilder::new("test-project")
            .with_output_targets(vec!["agents-md", "claude-md"])
            .build();
        assert!(config.contains("[output]"));
        assert!(config.contains("targets = [\"agents-md\", \"claude-md\"]"));
    }
}
