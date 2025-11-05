use crate::model::{ProjectConfig, PartialProjectConfig, Paths};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_project_config(path: &str) -> Result<ProjectConfig> {
    let raw = fs::read_to_string(path).with_context(|| format!("read config {}", path))?;
    let cfg: ProjectConfig = toml::from_str(&raw).with_context(|| "parse TOML")?;
    Ok(cfg)
}

/// Load config with precedence:
/// 1. User config (~/.cagents/config.toml) - lowest priority
/// 2. Project config (.cAGENTS/config.toml) - medium priority
/// 3. Local config (.cAGENTS/config.local.toml) - highest priority
///
/// Later configs override earlier ones (deep merge for nested objects)
pub fn load_config_with_precedence() -> Result<ProjectConfig> {
    let mut configs = Vec::new();

    // 1. Try user config
    if let Some(home_dir) = dirs::home_dir() {
        let user_config = home_dir.join(".cagents/config.toml");
        if user_config.exists() {
            match load_single_config(&user_config) {
                Ok(cfg) => configs.push(cfg),
                Err(e) => eprintln!("Warning: Failed to load user config: {}", e),
            }
        }
    }

    // 2. Load project config (required)
    let project_config = PathBuf::from(".cAGENTS/config.toml");
    if !project_config.exists() {
        anyhow::bail!("Project config not found at .cAGENTS/config.toml");
    }
    let cfg = load_single_config(&project_config)?;
    configs.push(cfg);

    // 3. Try local config (optional override)
    let local_config = PathBuf::from(".cAGENTS/config.local.toml");
    if local_config.exists() {
        match load_single_config(&local_config) {
            Ok(cfg) => configs.push(cfg),
            Err(e) => eprintln!("Warning: Failed to load local config: {}", e),
        }
    }

    // Merge all configs (later overrides earlier)
    merge_configs(configs)
}

/// Load a single config file as a partial config
/// This allows configs to be incomplete (e.g., local configs that only override some fields)
fn load_single_config(path: &Path) -> Result<PartialProjectConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
    let config: PartialProjectConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config: {}", path.display()))?;
    Ok(config)
}

/// Merge multiple partial configs and validate the result
/// Later configs override earlier ones. Final config must have all required fields.
fn merge_configs(configs: Vec<PartialProjectConfig>) -> Result<ProjectConfig> {
    if configs.is_empty() {
        anyhow::bail!("No configs to merge");
    }

    // Start with empty partial config
    let mut merged = PartialProjectConfig::default();

    // Apply each config in order (later overrides earlier)
    for cfg in configs {
        // Merge project metadata
        if cfg.project.is_some() {
            merged.project = cfg.project;
        }

        // Merge paths field-by-field
        if let Some(new_paths) = cfg.paths {
            let mut paths = merged.paths.take().unwrap_or_default();

            if new_paths.templates_dir.is_some() {
                paths.templates_dir = new_paths.templates_dir;
            }
            if new_paths.output_root.is_some() {
                paths.output_root = new_paths.output_root;
            }
            if new_paths.cursor_rules_dir.is_some() {
                paths.cursor_rules_dir = new_paths.cursor_rules_dir;
            }

            merged.paths = Some(paths);
        }

        // Merge defaults
        if let Some(new_defaults) = cfg.defaults {
            if let Some(ref mut existing_defaults) = merged.defaults {
                if new_defaults.engine.is_some() {
                    existing_defaults.engine = new_defaults.engine;
                }
                if new_defaults.targets.is_some() {
                    existing_defaults.targets = new_defaults.targets;
                }
                if new_defaults.order.is_some() {
                    existing_defaults.order = new_defaults.order;
                }
            } else {
                merged.defaults = Some(new_defaults);
            }
        }

        // Merge variables - deep merge JSON values
        if let Some(new_vars) = cfg.variables {
            if let Some(ref mut existing_vars) = merged.variables {
                if new_vars.static_.is_some() {
                    existing_vars.static_ = merge_json_values(
                        existing_vars.static_.as_ref(),
                        new_vars.static_.as_ref(),
                    );
                }
                if new_vars.env.is_some() {
                    existing_vars.env = merge_json_values(
                        existing_vars.env.as_ref(),
                        new_vars.env.as_ref(),
                    );
                }
                if new_vars.command.is_some() {
                    existing_vars.command = merge_json_values(
                        existing_vars.command.as_ref(),
                        new_vars.command.as_ref(),
                    );
                }
            } else {
                merged.variables = Some(new_vars);
            }
        }

        // Merge execution settings
        if let Some(new_exec) = cfg.execution {
            if let Some(ref mut existing_exec) = merged.execution {
                if new_exec.shell.is_some() {
                    existing_exec.shell = new_exec.shell;
                }
                if new_exec.timeout_ms.is_some() {
                    existing_exec.timeout_ms = new_exec.timeout_ms;
                }
                if new_exec.allow_commands.is_some() {
                    existing_exec.allow_commands = new_exec.allow_commands;
                }
            } else {
                merged.execution = Some(new_exec);
            }
        }

        // Merge output settings
        if cfg.output.is_some() {
            merged.output = cfg.output;
        }
    }

    // Validate and convert to full ProjectConfig
    validate_and_convert(merged)
}

/// Validate that a merged partial config has all required fields and convert to ProjectConfig
fn validate_and_convert(partial: PartialProjectConfig) -> Result<ProjectConfig> {
    // Validate paths (required)
    let partial_paths = partial.paths.ok_or_else(|| {
        anyhow::anyhow!("Missing required [paths] section in merged config")
    })?;

    let templates_dir = partial_paths.templates_dir.ok_or_else(|| {
        anyhow::anyhow!("Missing required field: paths.templatesDir")
    })?;

    let output_root = partial_paths.output_root.ok_or_else(|| {
        anyhow::anyhow!("Missing required field: paths.outputRoot")
    })?;

    let paths = Paths {
        templates_dir,
        output_root,
        cursor_rules_dir: partial_paths.cursor_rules_dir,
    };

    Ok(ProjectConfig {
        project: partial.project,
        paths,
        defaults: partial.defaults,
        variables: partial.variables,
        execution: partial.execution,
        output: partial.output,
    })
}

/// Merge two JSON values (for variables section)
/// If both are objects, merge keys; otherwise new overrides old
fn merge_json_values(
    old: Option<&serde_json::Value>,
    new: Option<&serde_json::Value>,
) -> Option<serde_json::Value> {
    match (old, new) {
        (None, None) => None,
        (None, Some(n)) => Some(n.clone()),
        (Some(o), None) => Some(o.clone()),
        (Some(o), Some(n)) => {
            if let (Some(old_obj), Some(new_obj)) = (o.as_object(), n.as_object()) {
                let mut merged = old_obj.clone();
                for (k, v) in new_obj {
                    merged.insert(k.clone(), v.clone());
                }
                Some(serde_json::Value::Object(merged))
            } else {
                // Non-object values: new wins
                Some(n.clone())
            }
        }
    }
}
