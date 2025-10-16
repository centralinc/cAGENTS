// Import from various rule formats (Cursor, Claude)

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Import from .cursorrules file(s) (legacy Cursor format) - supports nested files
pub fn import_cursorrules(backup: bool) -> Result<()> {
    // Find all .cursorrules files
    let locations = find_all_files_named(".cursorrules");

    if locations.is_empty() {
        anyhow::bail!("No .cursorrules files found");
    }

    // Handle multiple files
    if locations.len() > 1 {
        println!("▸ Found {} .cursorrules files:", locations.len());
        for loc in &locations {
            println!("   - {}", loc.display());
        }
        println!();
    }

    // Create cAGENTS structure
    let cagents_dir = PathBuf::from(".cAGENTS");
    let templates_dir = cagents_dir.join("templates");
    fs::create_dir_all(&templates_dir)?;

    // Create basic config
    let config = r#"[paths]
templatesDir = "templates"
outputRoot = "."
cursorRulesDir = ".cursor/rules"

[defaults]
engine = "builtin:simple"
targets = ["agentsmd", "cursor"]

[variables.static]
project = "imported-from-cursor"
"#;
    fs::write(cagents_dir.join("config.toml"), config)?;

    // Import each .cursorrules file
    let mut created_templates = Vec::new();

    for (idx, location) in locations.iter().enumerate() {
        // Backup (if requested)
        if backup {
            let backup_path = location.with_extension("cursorrules.backup");
            if let Err(e) = fs::copy(location, &backup_path) {
                eprintln!("▸ Failed to backup {}: {}", location.display(), e);
            } else {
                println!("▸ Backed up: {} → {}", location.display(), backup_path.display());
            }
        }

        // Read content
        let content = fs::read_to_string(location)
            .with_context(|| format!("Failed to read {}", location.display()))?;

        // Generate template name from location
        let template_name = if location == &PathBuf::from(".cursorrules") {
            "agents-cursor-root".to_string()
        } else {
            location
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| format!("agents-cursor-{}", s))
                .unwrap_or_else(|| format!("agents-cursor-{}", idx))
        };

        // Create template
        let template = format!(
            "---\nname: {}\ndescription: Imported from {}\ntargets: [\"agentsmd\", \"cursor\"]\norder: {}\n---\n{}",
            template_name,
            location.display(),
            (idx + 1) * 10,
            content
        );

        let template_filename = format!("{}.md", template_name);
        fs::write(templates_dir.join(&template_filename), template)?;
        created_templates.push(template_filename);

        println!("   ✓ Created template: {} (from {})", template_name, location.display());
    }

    fs::write(cagents_dir.join(".gitignore"), "config.local.toml\n**.local.*\n.output-cache\n")?;

    // Remove original files after successful import
    for location in &locations {
        if backup {
            let backup_path = location.with_extension("cursorrules.backup");
            println!("  ✓ Backed up {} → {}", location.display(), backup_path.display());
        }
        fs::remove_file(location).ok();
    }

    println!();
    println!("✓ Imported {} .cursorrules file(s)!", locations.len());
    println!("  ✓ Removed {} original .cursorrules file(s)", locations.len());
    println!();
    println!("Created:");
    println!("  .cAGENTS/config.toml");
    for template in &created_templates {
        println!("  .cAGENTS/templates/{}", template);
    }
    println!();

    // Auto-run build
    println!("▸ Running initial build...");
    println!();
    if let Err(e) = crate::cmd_build(None, false) {
        println!("⚠ Build failed: {}", e);
    } else {
        println!("✓ AGENTS.md generated!");
    }

    Ok(())
}

/// Import from .cursor/rules/ directory (modern Cursor format)
pub fn import_cursor_rules(_backup: bool) -> Result<()> {
    let rules_dir = PathBuf::from(".cursor/rules");

    if !rules_dir.exists() {
        anyhow::bail!(".cursor/rules directory not found");
    }

    // Create cAGENTS structure
    let cagents_dir = PathBuf::from(".cAGENTS");
    let templates_dir = cagents_dir.join("templates");
    fs::create_dir_all(&templates_dir)?;

    // Create config
    let config = r#"[paths]
templatesDir = "templates"
outputRoot = "."
cursorRulesDir = ".cursor/rules"

[defaults]
engine = "builtin:simple"
targets = ["agentsmd", "cursor"]
"#;
    fs::write(cagents_dir.join("config.toml"), config)?;

    // Convert each .md file to a template (recursively)
    let mut count = 0;
    let mut simplified_rules = Vec::new();

    let md_files = collect_md_files_recursive(&rules_dir)?;

    for path in md_files {
        let content = fs::read_to_string(&path)?;

        // Generate a unique name that preserves subdirectory structure
        let name = if let Ok(rel_path) = path.strip_prefix(&rules_dir) {
            // Convert path like "subdir/file.md" to "subdir-file"
            rel_path.with_extension("")
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "-")
        } else {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("rule")
                .to_string()
        };

        // Try to parse Cursor frontmatter to extract globs
        let (cursor_globs, body) = parse_cursor_rule(&content);

        // Build our template frontmatter
        let mut frontmatter = format!(
            "name: {}\ndescription: Imported from Cursor\n",
            name
        );

        // If Cursor rule had globs, preserve them and enable simplification
        if let Some(globs) = cursor_globs {
            if !globs.is_empty() {
                frontmatter.push_str("globs:\n");
                for glob in &globs {
                    frontmatter.push_str(&format!("  - \"{}\"\n", glob));
                }
                frontmatter.push_str("outputIn: common-parent\n");

                // Track for warning
                let common_parent = find_common_parent(&globs);
                if common_parent != globs {
                    simplified_rules.push((name.to_string(), globs.clone(), common_parent));
                }
            }
            // Note: empty globs or no globs = no when clause = implicitly always apply
        }

        frontmatter.push_str("targets: [\"agentsmd\", \"cursor\"]\n");
        frontmatter.push_str(&format!("order: {}\n", (count + 1) * 10));

        let template = format!("---\n{}---\n{}", frontmatter, body);

        fs::write(
            templates_dir.join(format!("agents-{}.md", name)),
            template
        )?;
        count += 1;
    }

    // Print warnings for simplified globs
    if !simplified_rules.is_empty() {
        println!();
        println!("⚠ Glob patterns simplified for {} rule(s):", simplified_rules.len());
        for (rule_name, original, simplified) in &simplified_rules {
            println!("  • {}", rule_name);
            println!("    Original: {}", original.join(", "));
            println!("    Simplified to parent: {}", simplified.join(", "));
        }
        println!();
        println!("  Templates preserve original patterns but outputIn: common-parent");
        println!("  This generates AGENTS.md at common parent directory.");
    }

    fs::write(cagents_dir.join(".gitignore"), "config.local.toml\n**.local.*\n.output-cache\n")?;

    // Remove original .cursor/rules directory after successful import
    if PathBuf::from(".cursor/rules").exists() {
        fs::remove_dir_all(".cursor/rules").ok();
        println!("  ✓ Removed original .cursor/rules directory");
        println!();
    }

    println!("✓ Imported {} Cursor rules to cAGENTS!", count);
    println!();
    println!("Created:");
    println!("  .cAGENTS/config.toml");
    println!("  .cAGENTS/templates/ ({} templates)", count);
    println!();

    // Auto-run build
    println!("▸ Running initial build...");
    println!();
    if let Err(e) = crate::cmd_build(None, false) {
        println!("⚠ Build failed: {}", e);
    } else {
        println!("✓ AGENTS.md generated!");
    }

    Ok(())
}

/// Detect which Cursor format is present
#[derive(Debug, PartialEq)]
pub enum CursorFormat {
    LegacyRootFile,      // .cursorrules
    ModernRulesDir,      // .cursor/rules/
    Both,
    None,
}

pub fn detect_cursor_format() -> CursorFormat {
    let has_cursorrules = PathBuf::from(".cursorrules").exists();
    let has_rules_dir = PathBuf::from(".cursor/rules").exists();

    match (has_cursorrules, has_rules_dir) {
        (true, true) => CursorFormat::Both,
        (true, false) => CursorFormat::LegacyRootFile,
        (false, true) => CursorFormat::ModernRulesDir,
        (false, false) => CursorFormat::None,
    }
}

/// Represents all possible import formats
#[derive(Debug, PartialEq, Clone)]
pub enum ImportFormat {
    CursorLegacy,    // .cursorrules
    CursorModern,    // .cursor/rules/
    AgentsMd,        // AGENTS.md
    ClaudeMd,        // CLAUDE.md
}

impl ImportFormat {
    pub fn display_name(&self) -> &str {
        match self {
            ImportFormat::CursorLegacy => ".cursorrules (Cursor legacy)",
            ImportFormat::CursorModern => ".cursor/rules/ (Cursor modern)",
            ImportFormat::AgentsMd => "AGENTS.md",
            ImportFormat::ClaudeMd => "CLAUDE.md",
        }
    }

    pub fn file_path(&self) -> PathBuf {
        match self {
            ImportFormat::CursorLegacy => PathBuf::from(".cursorrules"),
            ImportFormat::CursorModern => PathBuf::from(".cursor/rules"),
            ImportFormat::AgentsMd => PathBuf::from("AGENTS.md"),
            ImportFormat::ClaudeMd => PathBuf::from("CLAUDE.md"),
        }
    }
}

/// Detect all available import formats in the current directory
pub fn detect_all_formats() -> Vec<ImportFormat> {
    let mut formats = Vec::new();

    if PathBuf::from(".cursorrules").exists() {
        formats.push(ImportFormat::CursorLegacy);
    }

    if PathBuf::from(".cursor/rules").exists() {
        formats.push(ImportFormat::CursorModern);
    }

    if PathBuf::from("AGENTS.md").exists() {
        formats.push(ImportFormat::AgentsMd);
    }

    if PathBuf::from("CLAUDE.md").exists() {
        formats.push(ImportFormat::ClaudeMd);
    }

    formats
}

/// Import from AGENTS.md file(s) - supports nested files
pub fn import_agents_md(backup: bool) -> Result<()> {
    // Find all AGENTS.md files
    let locations = find_all_files_named("AGENTS.md");

    if locations.is_empty() {
        anyhow::bail!("No AGENTS.md files found");
    }

    // Handle multiple files
    if locations.len() > 1 {
        println!("▸ Found {} AGENTS.md files:", locations.len());
        for loc in &locations {
            println!("   - {}", loc.display());
        }
        println!();
    }

    // Create cAGENTS structure
    let cagents_dir = PathBuf::from(".cAGENTS");
    let templates_dir = cagents_dir.join("templates");
    fs::create_dir_all(&templates_dir)?;

    // Create basic config
    let config = r#"[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[variables.static]
project = "imported-from-agents-md"
"#;
    fs::write(cagents_dir.join("config.toml"), config)?;

    // Import each AGENTS.md file
    let mut created_templates = Vec::new();

    for (idx, location) in locations.iter().enumerate() {
        // Backup (if requested)
        if backup {
            let backup_path = location.with_extension("md.backup");
            if let Err(e) = fs::copy(location, &backup_path) {
                eprintln!("▸ Failed to backup {}: {}", location.display(), e);
            } else {
                println!("▸ Backed up: {} → {}", location.display(), backup_path.display());
            }
        }

        // Read content
        let content = fs::read_to_string(location)
            .with_context(|| format!("Failed to read {}", location.display()))?;

        // Generate template name from location
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

        // Create template
        let template = format!(
            "---\nname: {}\ndescription: Imported from {}\norder: {}\n---\n{}",
            template_name,
            location.display(),
            (idx + 1) * 10,
            content
        );

        let template_filename = format!("{}.md", template_name);
        fs::write(templates_dir.join(&template_filename), template)?;
        created_templates.push(template_filename);

        println!("   ✓ Created template: {} (from {})", template_name, location.display());
    }

    fs::write(cagents_dir.join(".gitignore"), "config.local.toml\n**.local.*\n.output-cache\n")?;

    // Remove original files after successful import
    for location in &locations {
        if backup {
            let backup_path = location.with_extension("md.backup");
            println!("  ✓ Backed up {} → {}", location.display(), backup_path.display());
        }
        fs::remove_file(location).ok();
    }

    println!();
    println!("✓ Imported {} AGENTS.md file(s)!", locations.len());
    println!("  ✓ Removed {} original AGENTS.md file(s)", locations.len());
    println!();
    println!("Created:");
    println!("  .cAGENTS/config.toml");
    for template in &created_templates {
        println!("  .cAGENTS/templates/{}", template);
    }
    println!();

    // Auto-run build
    println!("▸ Running initial build...");
    println!();
    if let Err(e) = crate::cmd_build(None, false) {
        println!("⚠ Build failed: {}", e);
    } else {
        println!("✓ AGENTS.md regenerated from template!");
    }

    Ok(())
}

/// Import from CLAUDE.md file(s) - supports nested files
pub fn import_claude_md(backup: bool) -> Result<()> {
    // Find all CLAUDE.md files
    let locations = find_all_files_named("CLAUDE.md");

    if locations.is_empty() {
        anyhow::bail!("No CLAUDE.md files found");
    }

    // Handle multiple files
    if locations.len() > 1 {
        println!("▸ Found {} CLAUDE.md files:", locations.len());
        for loc in &locations {
            println!("   - {}", loc.display());
        }
        println!();
    }

    // Create cAGENTS structure
    let cagents_dir = PathBuf::from(".cAGENTS");
    let templates_dir = cagents_dir.join("templates");
    fs::create_dir_all(&templates_dir)?;

    // Create basic config
    let config = r#"[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[variables.static]
project = "imported-from-claude-md"
"#;
    fs::write(cagents_dir.join("config.toml"), config)?;

    // Import each CLAUDE.md file
    let mut created_templates = Vec::new();

    for (idx, location) in locations.iter().enumerate() {
        // Backup (if requested)
        if backup {
            let backup_path = location.with_extension("md.backup");
            if let Err(e) = fs::copy(location, &backup_path) {
                eprintln!("▸ Failed to backup {}: {}", location.display(), e);
            } else {
                println!("▸ Backed up: {} → {}", location.display(), backup_path.display());
            }
        }

        // Read content
        let content = fs::read_to_string(location)
            .with_context(|| format!("Failed to read {}", location.display()))?;

        // Generate template name from location
        let template_name = if location == &PathBuf::from("CLAUDE.md") {
            "agents-root".to_string()
        } else {
            location
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| format!("agents-{}", s))
                .unwrap_or_else(|| format!("agents-{}", idx))
        };

        // Create template
        let template = format!(
            "---\nname: {}\ndescription: Imported from {}\norder: {}\n---\n{}",
            template_name,
            location.display(),
            (idx + 1) * 10,
            content
        );

        let template_filename = format!("{}.md", template_name);
        fs::write(templates_dir.join(&template_filename), template)?;
        created_templates.push(template_filename);

        println!("   ✓ Created template: {} (from {})", template_name, location.display());
    }

    fs::write(cagents_dir.join(".gitignore"), "config.local.toml\n**.local.*\n.output-cache\n")?;

    // Remove original files after successful import
    for location in &locations {
        if backup {
            let backup_path = location.with_extension("md.backup");
            println!("  ✓ Backed up {} → {}", location.display(), backup_path.display());
        }
        fs::remove_file(location).ok();
    }

    println!();
    println!("✓ Imported {} CLAUDE.md file(s)!", locations.len());
    println!("  ✓ Removed {} original CLAUDE.md file(s)", locations.len());
    println!();
    println!("Created:");
    println!("  .cAGENTS/config.toml");
    for template in &created_templates {
        println!("  .cAGENTS/templates/{}", template);
    }
    println!();

    // Auto-run build
    println!("▸ Running initial build...");
    println!();
    if let Err(e) = crate::cmd_build(None, false) {
        println!("⚠ Build failed: {}", e);
    } else {
        println!("✓ AGENTS.md regenerated from template!");
    }

    Ok(())
}

/// Import and merge multiple formats into separate templates
pub fn import_multiple_formats(formats: &[ImportFormat], backup: bool) -> Result<()> {
    if formats.is_empty() {
        anyhow::bail!("No formats provided to import");
    }

    // Create cAGENTS structure
    let cagents_dir = PathBuf::from(".cAGENTS");
    let templates_dir = cagents_dir.join("templates");
    fs::create_dir_all(&templates_dir)?;

    // Determine which output targets we need based on imported formats
    let mut output_targets = Vec::new();
    for format in formats {
        match format {
            ImportFormat::AgentsMd => {
                if !output_targets.contains(&"agents-md") {
                    output_targets.push("agents-md");
                }
            }
            ImportFormat::ClaudeMd => {
                if !output_targets.contains(&"claude-md") {
                    output_targets.push("claude-md");
                }
            }
            ImportFormat::CursorLegacy | ImportFormat::CursorModern => {
                if !output_targets.contains(&"agents-md") {
                    output_targets.push("agents-md");
                }
                if !output_targets.contains(&"cursorrules") {
                    output_targets.push("cursorrules");
                }
            }
        }
    }

    // Build output.targets config section
    let targets_config = if output_targets.is_empty() {
        String::new()
    } else {
        let targets_str = output_targets
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect::<Vec<_>>()
            .join(", ");
        format!("\n[output]\ntargets = [{}]\n", targets_str)
    };

    // Create config with appropriate output targets
    let config = format!(r#"[paths]
templatesDir = "templates"
outputRoot = "."
{0}
[defaults]
engine = "builtin:simple"

[variables.static]
project = "imported-merged"
"#, targets_config);
    fs::write(cagents_dir.join("config.toml"), config)?;

    // Import each format as a separate template
    let mut imported_count = 0;
    for (idx, format) in formats.iter().enumerate() {
        let path = format.file_path();

        let (content, backup_path) = match format {
            ImportFormat::CursorLegacy => {
                let content = fs::read_to_string(&path)?;
                (content, PathBuf::from(".cursorrules.backup"))
            }
            ImportFormat::CursorModern => {
                // For modern cursor, concatenate all files (recursively)
                let mut combined = String::new();
                let md_files = collect_md_files_recursive(&path)?;
                for file_path in md_files {
                    let file_content = fs::read_to_string(&file_path)?;
                    combined.push_str(&file_content);
                    combined.push_str("\n\n");
                }
                (combined, PathBuf::from(".cursor/rules.backup"))
            }
            ImportFormat::AgentsMd => {
                let content = fs::read_to_string(&path)?;
                (content, PathBuf::from("AGENTS.md.backup"))
            }
            ImportFormat::ClaudeMd => {
                let content = fs::read_to_string(&path)?;
                (content, PathBuf::from("CLAUDE.md.backup"))
            }
        };

        // Create template with unique name and target filter
        let (template_name, target_filter) = match format {
            ImportFormat::CursorLegacy | ImportFormat::CursorModern => {
                ("agents-cursor", "when:\n  target: [\"agents-md\", \"cursorrules\"]")
            }
            ImportFormat::AgentsMd => {
                ("agents-from-agentsmd", "when:\n  target: [\"agents-md\"]")
            }
            ImportFormat::ClaudeMd => {
                ("agents-from-claudemd", "when:\n  target: [\"claude-md\"]")
            }
        };

        let template = format!(r#"---
name: {}
description: Imported from {}
{}
order: {}
---
{}
"#, template_name, format.display_name(), target_filter, (idx + 1) * 10, content);

        fs::write(templates_dir.join(format!("{}.md", template_name)), template)?;

        // Backup original (if requested)
        if backup {
            if let Err(e) = fs::copy(&path, &backup_path) {
                eprintln!("Warning: Could not backup {}: {}", path.display(), e);
            }
        }

        imported_count += 1;
    }

    fs::write(cagents_dir.join(".gitignore"), "config.local.toml\n**.local.*\n.output-cache\n")?;

    // Remove original files after successful import
    for format in formats {
        match format {
            ImportFormat::CursorLegacy => {
                fs::remove_file(".cursorrules").ok();
                println!("  ✓ Removed original .cursorrules");
            }
            ImportFormat::CursorModern => {
                fs::remove_dir_all(".cursor/rules").ok();
                println!("  ✓ Removed original .cursor/rules");
            }
            ImportFormat::AgentsMd => {
                fs::remove_file("AGENTS.md").ok();
                println!("  ✓ Removed original AGENTS.md");
            }
            ImportFormat::ClaudeMd => {
                fs::remove_file("CLAUDE.md").ok();
                println!("  ✓ Removed original CLAUDE.md");
            }
        }
    }
    println!();

    println!("✓ Imported and merged {} formats!", imported_count);
    println!();
    println!("Created:");
    println!("  .cAGENTS/config.toml");
    println!("  .cAGENTS/templates/ ({} templates)", imported_count);
    println!();

    // Auto-run build
    println!("▸ Running initial build...");
    println!();
    if let Err(e) = crate::cmd_build(None, false) {
        println!("⚠ Build failed: {}", e);
    } else {
        println!("✓ AGENTS.md generated from merged templates!");
    }

    Ok(())
}

/// Parse a Cursor rule file to extract globs from frontmatter
/// Returns (globs, body without frontmatter)
fn parse_cursor_rule(content: &str) -> (Option<Vec<String>>, String) {
    let lines: Vec<&str> = content.lines().collect();

    // Check for frontmatter
    if lines.is_empty() || lines[0].trim() != "---" {
        return (None, content.to_string());
    }

    // Find closing ---
    let end_idx = lines.iter().skip(1).position(|line| line.trim() == "---");
    if end_idx.is_none() {
        return (None, content.to_string());
    }
    let end_idx = end_idx.unwrap() + 1;

    // Extract frontmatter
    let frontmatter = lines[1..end_idx].join("\n");

    // Parse globs from frontmatter
    let globs = extract_globs_from_yaml(&frontmatter);

    // Extract body
    let body = if end_idx + 1 < lines.len() {
        lines[end_idx + 1..].join("\n")
    } else {
        String::new()
    };

    (globs, body)
}

/// Extract globs from YAML frontmatter
/// Handles both comma-separated and array formats
fn extract_globs_from_yaml(yaml: &str) -> Option<Vec<String>> {
    // Try to parse as YAML and extract globs field
    if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(yaml) {
        if let Some(globs_value) = value.get("globs") {
            // Handle array format: ["pattern1", "pattern2"]
            if let Some(arr) = globs_value.as_sequence() {
                let globs: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                if !globs.is_empty() {
                    return Some(globs);
                }
            }
            // Handle comma-separated string: "pattern1, pattern2"
            else if let Some(s) = globs_value.as_str() {
                let globs: Vec<String> = s
                    .split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect();
                if !globs.is_empty() {
                    return Some(globs);
                }
            }
        }
    }

    None
}

/// Recursively collect all .md files in a directory
fn collect_md_files_recursive(dir: &std::path::Path) -> Result<Vec<PathBuf>> {
    let mut md_files = Vec::new();

    if !dir.is_dir() {
        return Ok(md_files);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively process subdirectories
            md_files.extend(collect_md_files_recursive(&path)?);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            md_files.push(path);
        }
    }

    Ok(md_files)
}

/// Find common parent directory from glob patterns
/// Returns simplified patterns using deepest common parent
fn find_common_parent(globs: &[String]) -> Vec<String> {
    if globs.is_empty() {
        return vec![];
    }

    if globs.len() == 1 {
        return globs.to_vec();
    }

    // Extract directory paths from patterns (before **)
    let dir_parts: Vec<Vec<&str>> = globs
        .iter()
        .map(|g| {
            // Split on /** and take first part
            let dir_part = g.split("/**").next().unwrap_or(g);
            dir_part.split('/').filter(|s| !s.is_empty()).collect()
        })
        .collect();

    // Find common prefix
    if dir_parts.is_empty() {
        return globs.to_vec();
    }

    let mut common_prefix = Vec::new();
    let first = &dir_parts[0];

    for (i, part) in first.iter().enumerate() {
        if dir_parts.iter().all(|p| p.get(i) == Some(part)) {
            common_prefix.push(*part);
        } else {
            break;
        }
    }

    // If we found a common parent, create simplified pattern
    if !common_prefix.is_empty() {
        let parent_path = common_prefix.join("/");
        vec![format!("{}/**", parent_path)]
    } else {
        // No common parent, return original
        globs.to_vec()
    }
}

/// Find all files with a specific name recursively
fn find_all_files_named(filename: &str) -> Vec<PathBuf> {
    use ignore::WalkBuilder;

    let mut locations = Vec::new();
    let is_hidden_file = filename.starts_with('.');

    let walker = WalkBuilder::new(".")
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .add_custom_ignore_filename(".cagentsignore")
        .filter_entry(move |e| {
            if e.path() == std::path::Path::new(".") {
                return true;
            }

            let name = e.file_name().to_string_lossy();
            // Skip specific directories but allow hidden files if we're searching for them
            if name == ".cAGENTS" || name == "node_modules" || name == "target" || name == "dist" {
                return false;
            }
            // If we're not looking for a hidden file, skip hidden directories
            if !is_hidden_file && name.starts_with('.') && e.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                return false;
            }
            true
        })
        .build();

    for entry in walker.flatten() {
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
            && entry.file_name() == filename
        {
            if let Ok(rel_path) = entry.path().strip_prefix(".") {
                locations.push(rel_path.to_path_buf());
            }
        }
    }

    locations
}
