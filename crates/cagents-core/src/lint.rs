// Linting and validation for config and templates

use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LintIssue {
    pub severity: Severity,
    pub file: String,
    pub line: Option<usize>,
    pub code: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

pub struct LintResult {
    pub issues: Vec<LintIssue>,
}

impl LintResult {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    pub fn add_error(&mut self, file: &str, message: &str) {
        self.issues.push(LintIssue {
            severity: Severity::Error,
            file: file.to_string(),
            line: None,
            code: "error".to_string(),
            message: message.to_string(),
            suggestion: None,
        });
    }

    pub fn add_warning(&mut self, file: &str, message: &str) {
        self.issues.push(LintIssue {
            severity: Severity::Warning,
            file: file.to_string(),
            line: None,
            code: "warning".to_string(),
            message: message.to_string(),
            suggestion: None,
        });
    }

    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Error).count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Warning).count()
    }

    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }

    pub fn print(&self) {
        if self.issues.is_empty() {
            println!("{} {}", "✅".bright_green(), "All checks passed!".green().bold());
            return;
        }

        // Print errors
        let errors: Vec<_> = self.issues.iter().filter(|i| i.severity == Severity::Error).collect();
        if !errors.is_empty() {
            println!("{} {} errors", "✗".bright_red(), errors.len().to_string().red().bold());
            println!();
            for issue in errors {
                println!("  {} {}", "•".bright_red(), issue.file.bright_white());
                println!("    {}", issue.message.red());
                if let Some(ref suggestion) = issue.suggestion {
                    println!("    {} {}", "→".bright_blue(), suggestion.bright_blue());
                }
                println!();
            }
        }

        // Print warnings
        let warnings: Vec<_> = self.issues.iter().filter(|i| i.severity == Severity::Warning).collect();
        if !warnings.is_empty() {
            println!("{} {} warnings", "▸ ".bright_yellow(), warnings.len().to_string().yellow().bold());
            println!();
            for issue in warnings {
                println!("  {} {}", "•".bright_yellow(), issue.file.bright_white());
                println!("    {}", issue.message.yellow());
                if let Some(ref suggestion) = issue.suggestion {
                    println!("    {} {}", "→".bright_blue(), suggestion.bright_blue());
                }
                println!();
            }
        }

        // Summary
        println!("{}", "─".repeat(60).bright_black());
        let summary = format!("Found {} errors, {} warnings", self.error_count(), self.warning_count());
        if self.has_errors() {
            println!("{}", summary.red());
        } else {
            println!("{}", summary.yellow());
        }
    }
}

/// Known valid output targets
const VALID_TARGETS: &[&str] = &["agents-md", "claude-md", "cursorrules"];

/// Validate config file
pub fn validate_config() -> Result<LintResult> {
    let mut result = LintResult::new();
    let config_path = PathBuf::from(".cAGENTS/config.toml");

    // Check config exists
    if !config_path.exists() {
        result.add_error(".cAGENTS/config.toml", "Config file not found");
        return Ok(result);
    }

    // Try to parse
    match crate::config::load_config_with_precedence() {
        Ok(config) => {
            // Check templatesDir exists
            let templates_dir = PathBuf::from(".cAGENTS").join(&config.paths.templates_dir);
            if !templates_dir.exists() {
                result.add_error(
                    ".cAGENTS/config.toml",
                    &format!("templatesDir '{}' does not exist", config.paths.templates_dir)
                );
            }

            // Check outputRoot is writable
            let output_root = PathBuf::from(&config.paths.output_root);
            if !output_root.exists() {
                result.add_warning(
                    ".cAGENTS/config.toml",
                    &format!("outputRoot '{}' does not exist", config.paths.output_root)
                );
            }

            // Validate output.targets values
            if let Some(output) = &config.output {
                if let Some(targets) = &output.targets {
                    for target in targets {
                        if !VALID_TARGETS.contains(&target.as_str()) {
                            result.add_error(
                                ".cAGENTS/config.toml",
                                &format!(
                                    "Unknown output target '{}'. Valid targets: {}",
                                    target,
                                    VALID_TARGETS.join(", ")
                                )
                            );
                        }
                    }
                }
            }
        }
        Err(e) => {
            result.add_error(".cAGENTS/config.toml", &format!("{:#}", e));
        }
    }

    Ok(result)
}

/// Validate templates
pub fn validate_templates() -> Result<LintResult> {
    let mut result = LintResult::new();
    let config_path = PathBuf::from(".cAGENTS/config.toml");

    if !config_path.exists() {
        return Ok(result); // Config validation will catch this
    }

    let config = match crate::config::load_config_with_precedence() {
        Ok(c) => c,
        Err(_) => return Ok(result), // Config validation will catch parse errors
    };

    let base_dir = PathBuf::from(".cAGENTS");
    let templates_dir = base_dir.join(&config.paths.templates_dir);

    if !templates_dir.exists() {
        return Ok(result); // Config validation will catch this
    }

    // Discover and validate templates
    match crate::loader::discover_rules(&config, &base_dir) {
        Ok(rules) => {
            if rules.is_empty() {
                result.add_warning(
                    &config.paths.templates_dir,
                    "No templates found. Add .md files to templates directory."
                );
            }

            // Validate each template has required fields
            for rule in &rules {
                let filename = rule.path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                if rule.frontmatter.name.is_none() {
                    result.add_warning(
                        filename,
                        "Template missing 'name' field in frontmatter"
                    );
                }

                // Validate when.target values
                if let Some(when) = &rule.frontmatter.when {
                    if let Some(targets) = &when.target {
                        for target in targets {
                            if !VALID_TARGETS.contains(&target.as_str()) {
                                result.add_error(
                                    filename,
                                    &format!(
                                        "Invalid when.target value '{}'. Valid targets: {}",
                                        target,
                                        VALID_TARGETS.join(", ")
                                    )
                                );
                            }
                        }
                    }
                }

                // Check for undefined variables when using builtin:simple engine
                let engine_spec = rule.frontmatter.engine.as_deref()
                    .or_else(|| config.defaults.as_ref().and_then(|d| d.engine.as_deref()));

                if let Some(engine) = engine_spec {
                    if engine == "builtin:simple" {
                        // Extract variables from template body using {{var}} pattern
                        if let Err(e) = validate_template_variables(&rule.body, &config, filename) {
                            result.add_error(filename, &e.to_string());
                        }
                    }
                }
            }
        }
        Err(e) => {
            result.add_error("templates/", &format!("Failed to load templates: {}", e));
        }
    }

    Ok(result)
}

/// Validate template variables are defined
fn validate_template_variables(
    template_body: &str,
    config: &crate::model::ProjectConfig,
    _filename: &str,
) -> Result<()> {
    use regex::Regex;

    let var_pattern = Regex::new(r"\{\{(\w+)\}\}").unwrap();
    let mut undefined_vars = Vec::new();

    // Collect available variables from config
    let mut available_vars = std::collections::HashSet::new();
    if let Some(vars) = &config.variables {
        if let Some(static_vars) = &vars.static_ {
            if let Some(obj) = static_vars.as_object() {
                for key in obj.keys() {
                    available_vars.insert(key.clone());
                }
            }
        }
        if let Some(cmd_vars) = &vars.command {
            if let Some(obj) = cmd_vars.as_object() {
                for key in obj.keys() {
                    available_vars.insert(key.clone());
                }
            }
        }
    }

    // Find all {{variable}} patterns in template
    for cap in var_pattern.captures_iter(template_body) {
        let var_name = cap[1].to_string();
        if !available_vars.contains(&var_name) {
            undefined_vars.push(var_name);
        }
    }

    // Remove duplicates
    undefined_vars.sort();
    undefined_vars.dedup();

    if !undefined_vars.is_empty() {
        anyhow::bail!("Undefined variables: {}", undefined_vars.join(", "));
    }

    Ok(())
}

/// Run all lint checks
pub fn lint_all() -> Result<LintResult> {
    let mut result = LintResult::new();

    // Validate config
    let config_result = validate_config()?;
    let has_config_errors = config_result.has_errors();
    result.issues.extend(config_result.issues);

    // Validate templates (only if config is valid)
    if !has_config_errors {
        let template_result = validate_templates()?;
        result.issues.extend(template_result.issues);
    }

    Ok(result)
}
