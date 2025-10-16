// Import from various rule formats (Cursor, Claude)

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Import from .cursorrules file (legacy Cursor format)
pub fn import_cursorrules(backup: bool) -> Result<()> {
    let cursorrules = PathBuf::from(".cursorrules");

    if !cursorrules.exists() {
        anyhow::bail!(".cursorrules file not found");
    }

    let content = fs::read_to_string(&cursorrules)?;

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

    // Convert .cursorrules to template (new naming)
    let template = format!(r#"---
name: agents-cursor
description: Imported from .cursorrules
targets: ["agentsmd", "cursor"]
order: 1
---
{}
"#, content);

    fs::write(templates_dir.join("agents-cursor.md"), template)?;
    fs::write(cagents_dir.join(".gitignore"), "config.local.toml\n**.local.*\n.output-cache\n")?;

    // Backup original (if requested)
    if backup {
        fs::copy(&cursorrules, ".cursorrules.backup")?;
        println!("Backup:");
        println!("  .cursorrules → .cursorrules.backup");
        println!();
    }

    // Remove original file after successful import
    fs::remove_file(&cursorrules).ok();
    println!("  ✓ Removed original .cursorrules");
    println!();

    println!("✓ Imported .cursorrules to cAGENTS!");
    println!();
    println!("Created:");
    println!("  .cAGENTS/config.toml");
    println!("  .cAGENTS/templates/agents-cursor.md");
    println!();

    // Auto-run build
    println!("▸ Running initial build...");
    println!();
    if let Err(e) = crate::cmd_build(None, None, None, None, false) {
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

    // Convert each .md file to a template
    let mut count = 0;
    let mut simplified_rules = Vec::new();

    for entry in fs::read_dir(&rules_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let content = fs::read_to_string(&path)?;
            let name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("rule");

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
    if let Err(e) = crate::cmd_build(None, None, None, None, false) {
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

/// Import from AGENTS.md file
pub fn import_agents_md(backup: bool) -> Result<()> {
    let agents_md = PathBuf::from("AGENTS.md");

    if !agents_md.exists() {
        anyhow::bail!("AGENTS.md file not found");
    }

    let content = fs::read_to_string(&agents_md)?;

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

    // Convert AGENTS.md to template
    let template = format!(r#"---
name: agents-root
description: Imported from AGENTS.md
order: 1
---
{}
"#, content);

    fs::write(templates_dir.join("agents-root.md"), template)?;
    fs::write(cagents_dir.join(".gitignore"), "config.local.toml\n**.local.*\n.output-cache\n")?;

    // Backup original (if requested)
    if backup {
        fs::copy(&agents_md, "AGENTS.md.backup")?;
        println!("Backup:");
        println!("  AGENTS.md → AGENTS.md.backup");
        println!();
    }

    // Remove original file after successful import
    fs::remove_file(&agents_md).ok();
    println!("  ✓ Removed original AGENTS.md");
    println!();

    println!("✓ Imported AGENTS.md to cAGENTS!");
    println!();
    println!("Created:");
    println!("  .cAGENTS/config.toml");
    println!("  .cAGENTS/templates/agents-root.md");
    println!();

    // Auto-run build
    println!("▸ Running initial build...");
    println!();
    if let Err(e) = crate::cmd_build(None, None, None, None, false) {
        println!("⚠ Build failed: {}", e);
    } else {
        println!("✓ AGENTS.md regenerated from template!");
    }

    Ok(())
}

/// Import from CLAUDE.md file
pub fn import_claude_md(backup: bool) -> Result<()> {
    let claude_md = PathBuf::from("CLAUDE.md");

    if !claude_md.exists() {
        anyhow::bail!("CLAUDE.md file not found");
    }

    let content = fs::read_to_string(&claude_md)?;

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

    // Convert CLAUDE.md to template
    let template = format!(r#"---
name: agents-root
description: Imported from CLAUDE.md
order: 1
---
{}
"#, content);

    fs::write(templates_dir.join("agents-root.md"), template)?;
    fs::write(cagents_dir.join(".gitignore"), "config.local.toml\n**.local.*\n.output-cache\n")?;

    // Backup original (if requested)
    if backup {
        fs::copy(&claude_md, "CLAUDE.md.backup")?;
        println!("Backup:");
        println!("  CLAUDE.md → CLAUDE.md.backup");
        println!();
    }

    // Remove original file after successful import
    fs::remove_file(&claude_md).ok();
    println!("  ✓ Removed original CLAUDE.md");
    println!();

    println!("✓ Imported CLAUDE.md to cAGENTS!");
    println!();
    println!("Created:");
    println!("  .cAGENTS/config.toml");
    println!("  .cAGENTS/templates/agents-root.md");
    println!();

    // Auto-run build
    println!("▸ Running initial build...");
    println!();
    if let Err(e) = crate::cmd_build(None, None, None, None, false) {
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
                // For modern cursor, concatenate all files
                let mut combined = String::new();
                for entry in fs::read_dir(&path)? {
                    let entry = entry?;
                    if entry.path().extension().and_then(|e| e.to_str()) == Some("md") {
                        let file_content = fs::read_to_string(entry.path())?;
                        combined.push_str(&file_content);
                        combined.push_str("\n\n");
                    }
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
    if let Err(e) = crate::cmd_build(None, None, None, None, false) {
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
