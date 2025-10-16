pub mod config;
pub mod model;
pub mod loader;
pub mod planner;
pub mod render;
pub mod merge;
pub mod writers;
pub mod adapters;
pub mod init;
pub mod interactive;
pub mod import;
pub mod lint;
pub mod helpers;

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Initialize cAGENTS in the current project
pub fn cmd_init(preset: &str, force: bool, dry_run: bool, backup: bool) -> Result<()> {
    use owo_colors::OwoColorize;

    // M4 Slice 5: Beautiful output
    if dry_run {
        println!("{} {}", "▸".bright_blue(), "Dry run mode - no files will be created".bright_blue());
        println!();
    }

    println!();
    println!("{}", "━".repeat(60).bright_black());
    println!("{} {}", "▸".bright_cyan(), "Initializing cAGENTS".bright_cyan().bold());
    println!("{}", "━".repeat(60).bright_black());
    println!();

    // Detect project info
    let info = init::ProjectInfo::detect()?;

    // Check for existing setup
    if info.has_cagents_dir {
        if force {
            println!("▸  Overwriting existing .cAGENTS/ (--force enabled)");
            println!();
        } else {
            anyhow::bail!(
                ".cAGENTS/ already exists.\n\n\
                Use --force to overwrite, or run 'cagents build' to use existing setup."
            );
        }
    }

    // M2 Slice 6: Dry run mode
    if dry_run {
        if info.has_agents_md {
            println!("▸ Would migrate existing AGENTS.md");
            println!("   Would create .cAGENTS/ with migrated template");
        } else {
            println!("▸ Would create .cAGENTS/ with preset: {}", preset);
        }
        println!();
        println!("→ Run without --dry-run to create files");
        return Ok(());
    }

    // Check for existing rule formats and suggest migration
    if info.has_agents_md || info.has_claude_md || info.has_cursorrules || info.has_cursor_rules {
        println!("▸ Found existing rule files:");
        if info.has_agents_md {
            println!("   • AGENTS.md");
        }
        if info.has_claude_md {
            println!("   • CLAUDE.md");
        }
        if info.has_cursorrules {
            println!("   • .cursorrules");
        }
        if info.has_cursor_rules {
            println!("   • .cursor/rules/");
        }
        println!();
        println!("  You can migrate these using: {} {}", "cagents migrate".bright_white(), "--backup".bright_black());
        println!();
    }

    // Only basic preset supported now (preset parameter ignored)
    init::init_basic(&info, force, backup)?;

    Ok(())
}

/// Execute a shell command and return its stdout as a trimmed string
fn execute_command(shell: &str, command: &str) -> Result<String> {
    let output = Command::new(shell)
        .arg("-c")
        .arg(command)
        .output()
        .with_context(|| format!("Failed to execute command: {}", command))?;

    if !output.status.success() {
        anyhow::bail!(
            "Command failed with exit code {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().to_string())
}

fn build_template_data_map(config: &crate::model::ProjectConfig) -> serde_json::Map<String, serde_json::Value> {
    let mut data = serde_json::Map::new();

    if let Some(vars) = &config.variables {
        if let Some(static_vars) = &vars.static_ {
            if let Some(obj) = static_vars.as_object() {
                for (key, value) in obj {
                    data.insert(key.clone(), value.clone());
                }
            }
        }

        if let Some(command_vars) = &vars.command {
            if let Some(obj) = command_vars.as_object() {
                let shell = config
                    .execution
                    .as_ref()
                    .and_then(|e| e.shell.as_deref())
                    .unwrap_or("bash");

                for (key, value) in obj {
                    if let Some(command) = value.as_str() {
                        match execute_command(shell, command) {
                            Ok(output) => {
                                data.insert(key.clone(), serde_json::Value::String(output));
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to execute command '{}': {}",
                                    command, e
                                );
                                data.insert(
                                    key.clone(),
                                    serde_json::Value::String(String::new()),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    data
}

fn merge_rule_data(
    base_data: &serde_json::Map<String, serde_json::Value>,
    rule: &loader::Rule,
) -> serde_json::Value {
    let mut merged = base_data.clone();

    if let Some(vars) = &rule.frontmatter.vars {
        if let Some(obj) = vars.as_object() {
            for (key, value) in obj {
                merged.insert(key.clone(), value.clone());
            }
        }
    }

    serde_json::Value::Object(merged)
}

fn resolve_engine_spec<'a>(
    rule: &'a loader::Rule,
    defaults: Option<&'a crate::model::Defaults>,
) -> Result<&'a str> {
    let engine_spec = rule
        .frontmatter
        .engine
        .as_deref()
        .or_else(|| defaults.and_then(|d| d.engine.as_deref()))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Template '{}' is missing an engine. Provide engine: \"command:<cmd>\" or \"builtin:simple\" in frontmatter, or set defaults.engine in config.",
                rule.path.display()
            )
        })?;

    Ok(engine_spec)
}

fn render_rule_with_command(
    rule: &loader::Rule,
    base_data: &serde_json::Map<String, serde_json::Value>,
    defaults: Option<&crate::model::Defaults>,
) -> Result<String> {
    let engine_spec = resolve_engine_spec(rule, defaults)?;
    let data_value = merge_rule_data(base_data, rule);

    // Check if using builtin engine
    if engine_spec.starts_with("builtin:") {
        let engine_type = engine_spec.strip_prefix("builtin:").unwrap().trim();

        match engine_type {
            "simple" => {
                adapters::builtin::render_simple(&rule.body, &data_value)
                    .with_context(|| format!("Builtin engine failed for template: {:?}", rule.path))
            }
            _ => {
                anyhow::bail!(
                    "Unknown builtin engine '{}'. Available: builtin:simple",
                    engine_type
                );
            }
        }
    } else if engine_spec.starts_with("command:") {
        // External command engine
        let command = engine_spec
            .strip_prefix("command:")
            .map(str::trim)
            .filter(|cmd| !cmd.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Engine '{}' has invalid command format (template {}).",
                    engine_spec,
                    rule.path.display()
                )
            })?;

        let frontmatter_json = serde_json::to_value(&rule.frontmatter)?;
        let path_str = rule.path.to_string_lossy();

        adapters::command::render_external(
            command,
            &rule.body,
            &data_value,
            &frontmatter_json,
            &path_str,
        )
        .with_context(|| format!("External compiler failed: {:?}", rule.path))
    } else {
        anyhow::bail!(
            "Invalid engine spec '{}'. Must start with 'builtin:' or 'command:' (template {}).",
            engine_spec,
            rule.path.display()
        );
    }
}

/// Build AGENTS.md and optional exports
///
/// M1 Slice 1 implementation:
/// - Loads config from .cAGENTS/config.toml (no precedence yet)
/// - Discovers templates in templatesDir
/// - Filters rules: no when clause = always applies, otherwise filtered by globs
/// - Renders with Handlebars using static variables only
/// - Merges by simple concatenation (no per-section strategies)
/// - Writes single root AGENTS.md
///
/// Not yet implemented:
/// - env/role/language filtering (params ignored)
/// - Command variable execution
/// - Nested directory outputs
/// - Config precedence (local/user configs)
/// - Custom output path (--out param ignored)
/// - Dry run mode
pub fn cmd_build(
    _out: Option<String>,
    _dry_run: bool,
) -> Result<()> {
    use owo_colors::OwoColorize;

    // 1. Load config with precedence (user < project < local)
    let config = config::load_config_with_precedence()?;

    // 2. Discover all rule templates
    let base_dir = PathBuf::from(".cAGENTS");
    let all_rules = loader::discover_rules(&config, &base_dir)?;

    // 3. Build template data from config variables
    let base_data = build_template_data_map(&config);

    // 4. Build context from config variables (for use in when clauses)
    let mut context_variables = std::collections::HashMap::new();

    // Add config variables to context (for use in when clauses)
    for (key, value) in &base_data {
        if let Some(s) = value.as_str() {
            context_variables.insert(key.clone(), s.to_string());
        }
    }

    // Create context from variables
    let context = planner::BuildContext::from_variables(context_variables);

    // 5. Plan outputs (group rules by target directories)
    let project_root = PathBuf::from(&config.paths.output_root);
    let outputs = planner::plan_outputs(&all_rules, &context, &project_root)?;

    let defaults = config.defaults.as_ref();

    // Get output targets from config (default to ["agents-md"])
    let output_targets = config
        .output
        .as_ref()
        .and_then(|o| o.targets.as_ref())
        .cloned()
        .unwrap_or_else(|| vec!["agents-md".to_string()]);

    // 6. Cleanup old files before writing new ones
    let current_output_paths: Vec<PathBuf> = outputs.keys().cloned().collect();

    // Cleanup old AGENTS.md files from directories no longer in plan
    let dir_cleaned_count = writers::agents_md::cleanup_old_outputs(&current_output_paths)?;

    // Cleanup output files for targets that were removed from config
    let target_cleaned_count = writers::agents_md::cleanup_old_target_files(&output_targets, &project_root)?;

    let total_cleaned = dir_cleaned_count + target_cleaned_count;
    if total_cleaned > 0 {
        println!("  {} Removed {} old output file(s)", "✓".bright_green(), total_cleaned);
        println!();
    }

    // 7. For each target directory, render and write target files
    // M8: Enhanced output with progress
    let mut files_written = 0;
    let mut target_files_created: std::collections::HashSet<String> = std::collections::HashSet::new();
    let total_outputs = outputs.len();

    if total_outputs > 0 {
        println!("{} {}", "▸".bright_cyan(), "Generating files...".bright_cyan());
        println!();
    }

    for (idx, (target_dir, rules)) in outputs.iter().enumerate() {
        // Show progress
        if total_outputs > 1 {
            println!("   {} {} {}/{}",
                "⠿".bright_black(),
                target_dir.display().to_string().bright_white(),
                (idx + 1).to_string().bright_black(),
                total_outputs.to_string().bright_black()
            );
        }

        // Determine output path and whether this is root
        let output_root = PathBuf::from(&config.paths.output_root);
        let output_dir = output_root.join(target_dir);
        let is_root = target_dir == &PathBuf::from(".");

        // Write to all configured targets
        for target in &output_targets {
            // Create context with current target for filtering
            // Clone all variables from the base context and add the target
            let mut target_variables = context.variables.clone();
            target_variables.insert("target".to_string(), target.clone());
            let target_context = planner::BuildContext::from_variables(target_variables);

            // Filter rules for this specific target
            let target_rules: Vec<&loader::Rule> = rules
                .iter()
                .filter(|rule| target_context.matches_when(&rule.frontmatter.when))
                .collect();

            if target_rules.is_empty() {
                continue; // Skip this target if no rules apply
            }

            // Render rules for this target
            let mut target_rendered_bodies = Vec::new();
            for rule in &target_rules {
                let rendered = render_rule_with_command(rule, &base_data, defaults)?;
                target_rendered_bodies.push(rendered);
            }

            // Merge for this target
            let target_merged = merge::merge_rule_bodies(&target_rendered_bodies)?;

            // Write to appropriate file
            match target.as_str() {
                "agents-md" => {
                    writers::agents_md::write_agents_md(&output_dir, &target_merged, is_root)?;
                    target_files_created.insert(target.to_string());
                }
                "claude-md" => {
                    writers::claude_md::write_claude_md(&output_dir, &target_merged, is_root)?;
                    target_files_created.insert(target.to_string());
                }
                "cursorrules" => {
                    writers::cursorrules::write_cursorrules(&output_dir, &target_merged)?;
                    target_files_created.insert(target.to_string());
                }
                _ => {
                    eprintln!("  Warning: Unknown output target '{}' - skipping", target);
                }
            }
        }

        files_written += 1;
    }

    // 8. Save output tracking for future cleanup (directories + targets)
    if let Err(e) = writers::agents_md::save_full_tracking(&current_output_paths, &output_targets) {
        eprintln!("  Warning: Could not save output tracking: {}", e);
    }

    // M4 Slice 5: Beautiful output
    println!();
    if files_written == 0 {
        println!("{} {}", "▸ ".yellow(), "No rules matched - no files generated".yellow());
    } else {
        println!("{} {}", "✓".bright_green(), "Generated Successfully!".green().bold());
        println!();

        // Show which target files were created (sorted for consistent output)
        let mut target_names: Vec<String> = target_files_created.iter()
            .map(|t| match t.as_str() {
                "agents-md" => "AGENTS.md",
                "claude-md" => "CLAUDE.md",
                "cursorrules" => ".cursorrules",
                _ => t.as_str(),
            })
            .map(|s| s.to_string())
            .collect();

        target_names.sort(); // Deterministic order: AGENTS.md, CLAUDE.md, ...

        if !target_names.is_empty() {
            for name in target_names {
                println!("   {} {}", "▸".bright_white(), name.bright_white());
            }
        }
    }
    println!();

    Ok(())
}
/// Export to different formats
/// DEPRECATED: Use `cagents build` with `[output]` config instead
pub fn cmd_export(target: &str) -> Result<()> {
    use owo_colors::OwoColorize;

    println!();
    println!("{} {}", "⚠".yellow(), "The export command is deprecated".yellow().bold());
    println!();
    println!("  Use {} instead with output targets in your config:", "cagents build".bright_white());
    println!();
    println!("  {}", "[output]".bright_black());
    println!("  {} {}", "targets =".bright_black(), r#"["agents-md", "claude-md", "cursorrules"]"#.bright_white());
    println!();
    println!("  This will generate all formats during build.");
    println!();
    println!("{}", "─".repeat(60).bright_black());
    println!();

    println!("{} {}", "▸".bright_cyan(), format!("Exporting to {} format (legacy)", target).bright_cyan().bold());
    println!();

    // M5 Slice 1: Cursor export
    if target == "cursor" {
        // Load config
        let config = config::load_config_with_precedence()?;

        // Discover templates
        let base_dir = PathBuf::from(".cAGENTS");
        let all_rules = loader::discover_rules(&config, &base_dir)?;

        // Build context (no filtering for export)
        let context = planner::BuildContext::new(None, None, None);

        // Filter rules
        let root_rules = planner::filter_rules_for_root(&all_rules, &context)?;

        // Build template data once for export
        let base_data = build_template_data_map(&config);
        let defaults = config.defaults.as_ref();

        // Render rules using BYOB commands
        let mut rendered_bodies = Vec::new();
        for rule in &root_rules {
            let rendered = render_rule_with_command(rule, &base_data, defaults)?;
            rendered_bodies.push(rendered);
        }

        // Write to cursor format
        let cursor_base = config.paths.cursor_rules_dir
            .as_ref()
            .map(|s| {
                // cursor_rules_dir might be ".cursor/rules", extract just ".cursor"
                let path = PathBuf::from(s);
                if path.ends_with("rules") {
                    path.parent().unwrap_or(&path).to_path_buf()
                } else {
                    path
                }
            })
            .unwrap_or_else(|| PathBuf::from(".cursor"));

        writers::cursor_mdc::write_cursor_rules(&cursor_base, &root_rules, &rendered_bodies)?;

        let count = root_rules.iter()
            .filter(|r| r.frontmatter.targets.as_ref()
                .map(|t| t.contains(&"cursor".to_string()))
                .unwrap_or(false))
            .count();

        println!("{} {}", "✓".bright_green(), "Exported successfully!".green().bold());
        println!();
        println!("   {} {} Cursor rules", "▸".bright_white(), count.to_string().bright_white().bold());
        println!("   {} {}", "▸".bright_blue(), cursor_base.join("rules").display().to_string().bright_white());
        println!();

        return Ok(());
    }

    // Other formats not yet implemented
    println!("{} Format '{}' not yet implemented", "▸ ".yellow(), target.yellow());
    println!("   Available: cursor");
    Ok(())
}

/// M6: Lint and validate configuration
pub fn cmd_lint() -> Result<()> {
    use owo_colors::OwoColorize;

    println!("{} {}", "▸".bright_cyan(), "Linting cAGENTS configuration...".bright_cyan().bold());
    println!();

    // Run all validations
    let result = lint::lint_all()?;

    // Print results
    result.print();

    // Exit with error code if errors found
    if result.has_errors() {
        anyhow::bail!("Lint failed with {} errors", result.error_count());
    }

    Ok(())
}

/// M7: Preview command - show build plan with rendered output preview
pub fn cmd_preview(_path: &str) -> Result<()> {
    use owo_colors::OwoColorize;

    println!();
    println!("{} {}", "▸".bright_cyan(), "Build Preview".bright_cyan().bold());
    println!();

    // Load config and rules
    let config = config::load_config_with_precedence()?;
    let base_dir = PathBuf::from(".cAGENTS");
    let all_rules = loader::discover_rules(&config, &base_dir)?;

    if all_rules.is_empty() {
        println!("{} {}", "ℹ️".bright_blue(), "No rules found".bright_blue());
        return Ok(());
    }

    // Get build context
    let context = planner::BuildContext::new(None, None, None);

    // Plan outputs
    let outputs = planner::plan_outputs(&all_rules, &context, &PathBuf::from("."))?;

    if outputs.is_empty() {
        println!("{} {}", "▸".yellow(), "No outputs planned - no rules match".yellow());
        return Ok(());
    }

    println!("{} {}", "✓".bright_green(), format!("{} file(s) will be generated:", outputs.len()).green().bold());
    println!();

    // Load base data for rendering
    let base_data = build_template_data_map(&config);
    let defaults = config.defaults.as_ref();

    // Show each output file
    for (idx, (target_dir, rules)) in outputs.iter().enumerate() {
        let output_path = if target_dir == &PathBuf::from(".") {
            PathBuf::from("AGENTS.md")
        } else {
            target_dir.join("AGENTS.md")
        };

        println!("{} {}", format!("{}.", idx + 1).bright_black(), output_path.display().to_string().bright_white().bold());
        println!();

        // Show which rules contribute
        println!("  {} {}", "Rules:".bright_black(), format!("{} template(s)", rules.len()).bright_white());
        for rule in rules {
            let name = rule.frontmatter.name.as_deref().unwrap_or("(unnamed)");
            println!("    {} {}", "•".bright_black(), name.bright_white());

            if let Some(globs) = &rule.frontmatter.globs {
                if !globs.is_empty() {
                    println!("      {} {}", "Globs:".bright_black(), globs.join(", ").yellow());
                }
            }
            if rule.frontmatter.when.is_none() {
                println!("      {} {}", "Apply:".bright_black(), "Always (no when clause)".green());
            }
        }
        println!();

        // Render preview
        println!("  {} {}", "Preview:".bright_cyan(), "(first 20 lines)".bright_black());
        println!("  {}", "─".repeat(70).bright_black());

        // Render each rule and merge
        let mut rendered_bodies = Vec::new();
        for rule in rules {
            match render_rule_with_command(rule, &base_data, defaults) {
                Ok(rendered) => rendered_bodies.push(rendered),
                Err(e) => {
                    println!("  {} {}", "Error:".bright_red(), e.to_string().red());
                    continue;
                }
            }
        }

        let merged = merge::merge_rule_bodies(&rendered_bodies)?;

        // Show first 20 lines of preview
        let lines: Vec<&str> = merged.lines().collect();
        let preview_lines = lines.iter().take(20);

        for line in preview_lines {
            println!("  {}", line.bright_white());
        }

        if lines.len() > 20 {
            println!("  {}", format!("... ({} more lines)", lines.len() - 20).bright_black());
        }

        println!("  {}", "─".repeat(70).bright_black());
        println!();
    }

    // Interactive navigation for multiple files
    if interactive::is_interactive() && outputs.len() > 1 {
        use inquire::Select;

        println!("{} {}", "▸".bright_blue(), "Navigate outputs:".bright_blue());
        println!();

        loop {
            // Build menu options
            let mut options: Vec<String> = outputs.keys().map(|dir| {
                if dir == &PathBuf::from(".") {
                    "AGENTS.md (root)".to_string()
                } else {
                    format!("{}/AGENTS.md", dir.display())
                }
            }).collect();
            options.push("Exit".to_string());

            let selection = Select::new("View full preview:", options)
                .prompt()
                .ok();

            if let Some(selected) = selection {
                if selected == "Exit" {
                    break;
                }

                // Find the matching output
                let selected_idx = outputs.iter().position(|(dir, _)| {
                    let path_str = if dir == &PathBuf::from(".") {
                        "AGENTS.md (root)".to_string()
                    } else {
                        format!("{}/AGENTS.md", dir.display())
                    };
                    path_str == selected
                });

                if let Some(idx) = selected_idx {
                    let (target_dir, rules) = outputs.iter().nth(idx).unwrap();
                    let output_path = if target_dir == &PathBuf::from(".") {
                        PathBuf::from("AGENTS.md")
                    } else {
                        target_dir.join("AGENTS.md")
                    };

                    println!();
                    println!("{}", "═".repeat(70).bright_black());
                    println!("{} {}", "▸".bright_cyan(), output_path.display().to_string().bright_cyan().bold());
                    println!("{}", "═".repeat(70).bright_black());
                    println!();

                    // Render full content
                    let mut rendered_bodies = Vec::new();
                    for rule in rules {
                        match render_rule_with_command(rule, &base_data, defaults) {
                            Ok(rendered) => rendered_bodies.push(rendered),
                            Err(e) => {
                                println!("{} {}", "Error:".bright_red(), e.to_string().red());
                                continue;
                            }
                        }
                    }

                    let merged = merge::merge_rule_bodies(&rendered_bodies)?;
                    println!("{}", merged);
                    println!();
                    println!("{}", "═".repeat(70).bright_black());
                    println!();
                }
            } else {
                break;
            }
        }
    } else if outputs.len() == 1 {
        println!("{} {}", "▸".bright_blue(), "Tip: Run `cagents build` to generate this file".bright_blue());
    }

    Ok(())
}

/// M8: Status command - show project stats
pub fn cmd_status() -> Result<()> {
    use owo_colors::OwoColorize;
    use comfy_table::{Table, Row, Cell};
    use comfy_table::presets::UTF8_FULL;

    println!();
    println!("{} {}", "▸".bright_cyan(), "cAGENTS Status".bright_cyan().bold());
    println!();

    // Check if cAGENTS exists
    if !PathBuf::from(".cAGENTS").exists() {
        println!("{} {}", "✗".bright_red(), "Not initialized".red());
        println!();
        println!("   Run {} to get started", "cagents init".bright_white());
        return Ok(());
    }

    // Load config
    let config = config::load_config_with_precedence()?;
    let base_dir = PathBuf::from(".cAGENTS");

    // Count templates
    let templates = loader::discover_rules(&config, &base_dir)?;

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    table.add_row(Row::from(vec![
        Cell::new("Configuration").fg(comfy_table::Color::Cyan),
        Cell::new(".cAGENTS/config.toml").fg(comfy_table::Color::Green),
    ]));

    table.add_row(Row::from(vec![
        Cell::new("Templates").fg(comfy_table::Color::Cyan),
        Cell::new(format!("{} found", templates.len())).fg(comfy_table::Color::White),
    ]));

    table.add_row(Row::from(vec![
        Cell::new("Output").fg(comfy_table::Color::Cyan),
        Cell::new(&config.paths.output_root).fg(comfy_table::Color::White),
    ]));

    // Check if AGENTS.md exists
    if PathBuf::from("AGENTS.md").exists() {
        table.add_row(Row::from(vec![
            Cell::new("AGENTS.md").fg(comfy_table::Color::Cyan),
            Cell::new("Generated").fg(comfy_table::Color::Green),
        ]));
    }

    println!("{}", table);
    println!();

    // Show templates
    if !templates.is_empty() {
        println!("{} {}:", "▸".bright_blue(), "Templates".bright_blue().bold());
        for template in &templates {
            let name = template.frontmatter.name.as_deref().unwrap_or("unnamed");
            println!("   {} {}", "•".bright_black(), name.bright_white());
        }
        println!();
    }

    Ok(())
}

/// M7 Slice 2: Setup package manager integration
pub fn cmd_setup(manager: &str) -> Result<()> {
    use owo_colors::OwoColorize;

    println!("{} {}", "▸".bright_cyan(), format!("Setting up {} integration", manager).bright_cyan().bold());
    println!();

    match manager {
        "pnpm" | "npm" => helpers::setup::setup_pnpm()?,
        _ => {
            anyhow::bail!("Unknown package manager '{}'. Supported: pnpm, npm", manager);
        }
    }

    Ok(())
}

/// M5 Slice 2: Migrate from other formats
pub fn cmd_migrate(from: Option<&str>, backup: bool) -> Result<()> {
    use owo_colors::OwoColorize;

    println!("{} {}", "▸".bright_cyan(), "Migrating rules".bright_cyan().bold());
    println!();

    // Detect all available formats
    let all_formats = import::detect_all_formats();

    if all_formats.is_empty() {
        println!("✗ No formats found to migrate");
        println!();
        println!("   Looking for:");
        println!("     • .cursorrules (Cursor legacy)");
        println!("     • .cursor/rules/ (Cursor modern)");
        println!("     • AGENTS.md");
        println!("     • CLAUDE.md");
        return Ok(());
    }

    // If specific format requested via --from
    if let Some(path) = from {
        let p = PathBuf::from(path);
        if p.ends_with(".cursorrules") {
            import::import_cursorrules(backup)?;
        } else if p.ends_with("rules") || p.ends_with(".cursor") {
            import::import_cursor_rules(backup)?;
        } else if p.ends_with("AGENTS.md") {
            import::import_agents_md(backup)?;
        } else if p.ends_with("CLAUDE.md") {
            import::import_claude_md(backup)?;
        } else {
            anyhow::bail!("Unknown import format: {}", path);
        }
    } else if all_formats.len() == 1 {
        // Single format found, import it directly
        let format = &all_formats[0];
        println!("▸ Found {}", format.display_name());
        println!();

        match format {
            import::ImportFormat::CursorLegacy => import::import_cursorrules(backup)?,
            import::ImportFormat::CursorModern => import::import_cursor_rules(backup)?,
            import::ImportFormat::AgentsMd => import::import_agents_md(backup)?,
            import::ImportFormat::ClaudeMd => import::import_claude_md(backup)?,
        }
    } else {
        // Multiple formats found
        println!("▸ Found {} formats to migrate:", all_formats.len());
        for format in &all_formats {
            println!("   • {}", format.display_name());
        }
        println!();

        if interactive::is_interactive() {
            use inquire::Select;

            // Build menu options
            let mut options: Vec<String> = all_formats.iter()
                .map(|f| f.display_name().to_string())
                .collect();
            options.push("Merge all formats into separate templates".to_string());

            let choice = Select::new("Multiple formats found. What would you like to do?", options).prompt()?;

            if choice.contains("Merge all") {
                // Import and merge all
                import::import_multiple_formats(&all_formats, backup)?;
            } else {
                // Import the selected format
                let selected_format = all_formats.iter()
                    .find(|f| f.display_name() == choice)
                    .unwrap();

                match selected_format {
                    import::ImportFormat::CursorLegacy => import::import_cursorrules(backup)?,
                    import::ImportFormat::CursorModern => import::import_cursor_rules(backup)?,
                    import::ImportFormat::AgentsMd => import::import_agents_md(backup)?,
                    import::ImportFormat::ClaudeMd => import::import_claude_md(backup)?,
                }
            }
        } else {
            println!("▸  Multiple formats found. Use --from to specify which to migrate:");
            for format in &all_formats {
                println!("     cagents migrate --from {}", format.file_path().display());
            }
            println!();
            println!("   Or run interactively to merge all formats.");
            return Ok(());
        }
    }

    println!();
    println!("{}", "Next steps:".bright_white());
    println!("  1. Run: {}", "cagents build".bright_white());
    println!("  2. Check: {}", "AGENTS.md".bright_white());

    Ok(())
}

/// M4 Slice 4: Config command for interactive configuration
/// DEPRECATED: Users can directly edit .cAGENTS/config.toml
pub fn cmd_config() -> Result<()> {
    use owo_colors::OwoColorize;

    println!();
    println!("{} {}", "⚠".yellow(), "The config command is deprecated".yellow().bold());
    println!();
    println!("  Edit your configuration directly:");
    println!("  {}", ".cAGENTS/config.toml".bright_white());
    println!();
    println!("  Or use your editor:");
    println!("  {} {}", "$".bright_black(), "$EDITOR .cAGENTS/config.toml".bright_white());
    println!();
    println!("{}", "─".repeat(60).bright_black());
    println!();

    // Check if config exists
    let config_path = PathBuf::from(".cAGENTS/config.toml");
    if !config_path.exists() {
        println!("{} {}", "✗".bright_red(), "No cAGENTS configuration found.".red());
        println!();
        println!("   Run {} to initialize.", "cagents init".bright_white());
        return Ok(());
    }

    // Load current config
    let config = config::load_config_with_precedence()?;

    println!("{}", "Current Configuration:".bright_white());
    println!();

    // Display variables
    if let Some(ref vars) = config.variables {
        if let Some(ref static_vars) = vars.static_ {
            if let Some(obj) = static_vars.as_object() {
                println!("  {} {}", "▸".bright_blue(), "Static Variables:".bright_blue());
                for (key, value) in obj {
                    println!("     {} = {}", key.bright_white(), value.to_string().trim_matches('"').bright_black());
                }
                println!();
            }
        }
    }

    // Show paths
    println!("  {} {}", "▸".bright_blue(), "Paths:".bright_blue());
    println!("     templatesDir = {}", config.paths.templates_dir.bright_white());
    println!("     outputRoot = {}", config.paths.output_root.bright_white());
    println!();

    // Interactive menu
    if interactive::is_interactive() {
        use inquire::Select;

        let options = vec![
            "View full config",
            "Edit in $EDITOR",
            "Exit",
        ];

        let choice = Select::new("What would you like to do?", options).prompt()?;

        match choice {
            "View full config" => {
                let content = std::fs::read_to_string(&config_path)?;
                println!();
                println!("{}", "━".repeat(60).bright_black());
                println!("{}", content);
                println!("{}", "━".repeat(60).bright_black());
            }
            "Edit in $EDITOR" => {
                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                std::process::Command::new(&editor)
                    .arg(&config_path)
                    .status()?;
                println!("{} {}", "✓".green(), "Config updated".green());
            }
            _ => {}
        }
    }

    Ok(())
}

/// Render AGENTS.md for a specific file
pub fn cmd_render(file_path: &str, var_args: Vec<String>) -> Result<()> {
    // 1. Load config with precedence
    let config = config::load_config_with_precedence()?;

    // 2. Discover all rule templates
    let base_dir = PathBuf::from(".cAGENTS");
    let all_rules = loader::discover_rules(&config, &base_dir)?;

    // 3. Parse variables from CLI args
    let mut variables = serde_json::Map::new();
    for var_arg in var_args {
        if let Some((key, value)) = var_arg.split_once('=') {
            variables.insert(key.to_string(), serde_json::Value::String(value.to_string()));
        } else {
            anyhow::bail!("Invalid variable format '{}'. Expected KEY=VALUE", var_arg);
        }
    }

    // 4. Build context with generic variables (no hardcoded env/role/language)
    // For now, BuildContext still expects the old fields, but we inject variables into template data
    let context = planner::BuildContext::new(None, None, None);

    // 5. Resolve file path (make absolute or relative to cwd)
    let file_path = PathBuf::from(file_path);
    let file_path = if file_path.is_absolute() {
        file_path
    } else {
        std::env::current_dir()?.join(&file_path)
    };

    // Convert to relative path for glob matching
    let project_root = PathBuf::from(&config.paths.output_root);
    let rel_file_path = file_path
        .strip_prefix(&project_root)
        .unwrap_or(&file_path);

    // 6. Filter rules for this specific file
    let matching_rules = planner::filter_rules_for_file(&all_rules, rel_file_path, &context)?;

    if matching_rules.is_empty() {
        // No matching rules - output empty or warning to stderr
        eprintln!("No rules match file: {}", rel_file_path.display());
        return Ok(());
    }

    // 7. Build template data from config variables + CLI variables
    let mut base_data = build_template_data_map(&config);

    // Override/add CLI variables
    for (key, value) in variables {
        base_data.insert(key, value);
    }

    let defaults = config.defaults.as_ref();

    // 8. Render each matching rule
    let mut rendered_bodies = Vec::new();
    for rule in &matching_rules {
        let rendered = render_rule_with_command(rule, &base_data, defaults)?;
        rendered_bodies.push(rendered);
    }

    // 9. Merge rendered bodies
    let merged = merge::merge_rule_bodies(&rendered_bodies)?;

    // 10. Output to stdout (no extra formatting, just the content)
    print!("{}", merged);

    Ok(())
}

/// Show comprehensive context and metadata for a file
pub fn cmd_context(file_path: &str, var_args: Vec<String>, json_output: bool) -> Result<()> {
    // 1. Load config with precedence
    let config = config::load_config_with_precedence()?;

    // 2. Discover all rule templates
    let base_dir = PathBuf::from(".cAGENTS");
    let all_rules = loader::discover_rules(&config, &base_dir)?;

    // 3. Parse variables from CLI args
    let mut variables = serde_json::Map::new();
    for var_arg in var_args {
        if let Some((key, value)) = var_arg.split_once('=') {
            variables.insert(key.to_string(), serde_json::Value::String(value.to_string()));
        } else {
            anyhow::bail!("Invalid variable format '{}'. Expected KEY=VALUE", var_arg);
        }
    }

    // 4. Build context
    let context = planner::BuildContext::new(None, None, None);

    // 5. Resolve file path
    let file_path = PathBuf::from(file_path);
    let file_path = if file_path.is_absolute() {
        file_path
    } else {
        std::env::current_dir()?.join(&file_path)
    };

    let project_root = PathBuf::from(&config.paths.output_root);
    let rel_file_path = file_path
        .strip_prefix(&project_root)
        .unwrap_or(&file_path);

    // 6. Filter rules for this specific file
    let matching_rules = planner::filter_rules_for_file(&all_rules, rel_file_path, &context)?;

    if matching_rules.is_empty() {
        eprintln!("No rules match file: {}", rel_file_path.display());
        return Ok(());
    }

    // 7. Build template data from config variables + CLI variables
    let mut base_data = build_template_data_map(&config);

    // Track variable sources
    let mut var_sources = serde_json::Map::new();
    for (key, value) in &base_data {
        var_sources.insert(
            key.clone(),
            serde_json::json!({
                "value": value,
                "source": "config.static"
            })
        );
    }

    // Override/add CLI variables
    for (key, value) in variables {
        var_sources.insert(
            key.clone(),
            serde_json::json!({
                "value": value.clone(),
                "source": "cli"
            })
        );
        base_data.insert(key, value);
    }

    let defaults = config.defaults.as_ref();

    // 8. Render each matching rule
    let mut rendered_bodies = Vec::new();
    for rule in &matching_rules {
        let rendered = render_rule_with_command(rule, &base_data, defaults)?;
        rendered_bodies.push(rendered);
    }

    // 9. Merge rendered bodies
    let merged = merge::merge_rule_bodies(&rendered_bodies)?;

    // 10. Collect metadata about matched rules
    let mut rules_metadata = Vec::new();
    for rule in &matching_rules {
        let reason = if rule.frontmatter.when.is_none() {
            "always (no when clause)".to_string()
        } else if let Some(globs) = &rule.frontmatter.globs {
            format!("glob: {}", globs.join(", "))
        } else {
            "no glob".to_string()
        };

        rules_metadata.push(serde_json::json!({
            "name": rule.frontmatter.name.as_ref().unwrap_or(&"unnamed".to_string()),
            "path": rule.path.to_string_lossy(),
            "reason": reason,
            "order": rule.frontmatter.order.unwrap_or(50)
        }));
    }

    // 11. Extract file info
    let file_info = serde_json::json!({
        "extension": rel_file_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or(""),
        "directory": rel_file_path.parent()
            .and_then(|p| p.to_str())
            .unwrap_or("."),
        "relative_path": rel_file_path.to_string_lossy()
    });

    // 12. Output in requested format
    if json_output {
        // JSON format
        let output = serde_json::json!({
            "file": rel_file_path.to_string_lossy(),
            "matched_rules": rules_metadata,
            "variables": var_sources,
            "file_info": file_info,
            "rendered_content": merged
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Markdown format
        println!("# Context for {}\n", rel_file_path.display());

        println!("## Matched Rules ({})", matching_rules.len());
        for rule in &matching_rules {
            let name = rule.frontmatter.name.as_deref().unwrap_or("unnamed");
            let reason = if rule.frontmatter.when.is_none() {
                "always (no when clause)".to_string()
            } else if let Some(globs) = &rule.frontmatter.globs {
                format!("glob: {}", globs.join(", "))
            } else {
                "no glob".to_string()
            };
            let order = rule.frontmatter.order.unwrap_or(50);
            println!("- **{}** - {} (order: {})", name, reason, order);
        }
        println!();

        println!("## Available Variables ({})", var_sources.len());
        for (key, value) in &var_sources {
            if let Some(obj) = value.as_object() {
                let val = obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let src = obj.get("source").and_then(|v| v.as_str()).unwrap_or("");
                println!("- `{}` = \"{}\" ({})", key, val, src);
            }
        }
        println!();

        println!("## File Info");
        println!("- Extension: {}", file_info["extension"].as_str().unwrap_or(""));
        println!("- Directory: {}", file_info["directory"].as_str().unwrap_or(""));
        println!("- Path: {}", file_info["relative_path"].as_str().unwrap_or(""));
        println!();

        println!("---\n");
        print!("{}", merged);
    }

    Ok(())
}
