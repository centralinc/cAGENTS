use crate::model::ProjectConfig;
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

/// Load a single config file
fn load_single_config(path: &Path) -> Result<ProjectConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
    let config: ProjectConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config: {}", path.display()))?;
    Ok(config)
}

/// Merge multiple configs (later configs override earlier ones)
/// For now, simple last-wins for each top-level field
fn merge_configs(configs: Vec<ProjectConfig>) -> Result<ProjectConfig> {
    if configs.is_empty() {
        anyhow::bail!("No configs to merge");
    }

    // Start with first config
    let mut merged = configs[0].clone();

    // Apply each subsequent config
    for cfg in configs.iter().skip(1) {
        // Merge project metadata
        if cfg.project.is_some() {
            merged.project = cfg.project.clone();
        }

        // Merge paths - required field, but allow partial overrides
        // For now, take the whole paths object if present
        // In a real implementation, we'd merge field-by-field
        merged.paths = cfg.paths.clone();

        // Merge defaults
        if let Some(ref new_defaults) = cfg.defaults {
            if let Some(ref mut existing_defaults) = merged.defaults {
                if new_defaults.engine.is_some() {
                    existing_defaults.engine = new_defaults.engine.clone();
                }
                if new_defaults.targets.is_some() {
                    existing_defaults.targets = new_defaults.targets.clone();
                }
                if new_defaults.order.is_some() {
                    existing_defaults.order = new_defaults.order;
                }
            } else {
                merged.defaults = Some(new_defaults.clone());
            }
        }

        // Merge variables - deep merge JSON values
        if let Some(ref new_vars) = cfg.variables {
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
                merged.variables = Some(new_vars.clone());
            }
        }

        // Merge execution settings
        if let Some(ref new_exec) = cfg.execution {
            if let Some(ref mut existing_exec) = merged.execution {
                if new_exec.shell.is_some() {
                    existing_exec.shell = new_exec.shell.clone();
                }
                if new_exec.timeout_ms.is_some() {
                    existing_exec.timeout_ms = new_exec.timeout_ms;
                }
                if new_exec.allow_commands.is_some() {
                    existing_exec.allow_commands = new_exec.allow_commands;
                }
            } else {
                merged.execution = Some(new_exec.clone());
            }
        }
    }

    Ok(merged)
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
