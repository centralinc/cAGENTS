//! Telemetry configuration loading and validation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Telemetry configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled (default: true, opt-out model)
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Debug mode - print events instead of sending (default: false)
    #[serde(default)]
    pub debug: bool,

    /// Override Mixpanel token (for testing, optional)
    pub mixpanel_token: Option<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debug: false,
            mixpanel_token: None,
        }
    }
}

fn default_enabled() -> bool {
    true
}

/// Load telemetry configuration with precedence:
/// 1. Environment variables (highest priority)
/// 2. Local config (.cAGENTS/config.local.toml)
/// 3. Project config (.cAGENTS/config.toml)
/// 4. User config (~/.cagents/config.toml)
/// 5. Default (enabled=true)
pub fn load_telemetry_config() -> Result<TelemetryConfig> {
    let mut config = TelemetryConfig::default();

    // Load from user config
    if let Some(home_dir) = dirs::home_dir() {
        let user_config = home_dir.join(".cagents/config.toml");
        if user_config.exists() {
            if let Ok(cfg) = load_config_from_file(&user_config) {
                config = cfg;
            }
        }
    }

    // Load from project config
    let project_config = PathBuf::from(".cAGENTS/config.toml");
    if project_config.exists() {
        if let Ok(cfg) = load_config_from_file(&project_config) {
            merge_config(&mut config, cfg);
        }
    }

    // Load from local config
    let local_config = PathBuf::from(".cAGENTS/config.local.toml");
    if local_config.exists() {
        if let Ok(cfg) = load_config_from_file(&local_config) {
            merge_config(&mut config, cfg);
        }
    }

    // Apply environment variable overrides
    apply_env_overrides(&mut config)?;

    Ok(config)
}

/// Load telemetry config from a TOML file
fn load_config_from_file(path: &PathBuf) -> Result<TelemetryConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;

    // Parse full config to extract telemetry section
    #[derive(Deserialize)]
    struct FullConfig {
        #[serde(default)]
        telemetry: Option<TelemetryConfig>,
    }

    let full_config: FullConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config: {}", path.display()))?;

    Ok(full_config.telemetry.unwrap_or_default())
}

/// Merge new config into existing config
fn merge_config(base: &mut TelemetryConfig, new: TelemetryConfig) {
    // enabled and debug are overwritten if present in new config
    // We can't tell if they were explicitly set or defaulted,
    // so we only override if the new value is false (opt-out)
    if !new.enabled {
        base.enabled = false;
    }
    if new.debug {
        base.debug = true;
    }
    if new.mixpanel_token.is_some() {
        base.mixpanel_token = new.mixpanel_token;
    }
}

/// Apply environment variable overrides
fn apply_env_overrides(config: &mut TelemetryConfig) -> Result<()> {
    // CAGENTS_TELEMETRY_DISABLED=1 disables telemetry
    if env::var("CAGENTS_TELEMETRY_DISABLED").is_ok() {
        config.enabled = false;
        return Ok(());
    }

    // DO_NOT_TRACK=1 (universal opt-out)
    if env::var("DO_NOT_TRACK").is_ok() {
        config.enabled = false;
        return Ok(());
    }

    // CAGENTS_TELEMETRY_DEBUG=1 enables debug mode
    if env::var("CAGENTS_TELEMETRY_DEBUG").is_ok() {
        config.debug = true;
    }

    // Auto-disable in CI unless explicitly enabled
    if is_ci() && env::var("CAGENTS_TELEMETRY_IN_CI").is_err() {
        config.enabled = false;
    }

    Ok(())
}

/// Check if running in CI environment
fn is_ci() -> bool {
    env::var("CI").is_ok()
        || env::var("CONTINUOUS_INTEGRATION").is_ok()
        || env::var("GITHUB_ACTIONS").is_ok()
        || env::var("GITLAB_CI").is_ok()
        || env::var("CIRCLECI").is_ok()
        || env::var("TRAVIS").is_ok()
}

/// Get the telemetry state directory
pub fn get_telemetry_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("Could not determine home directory")?;
    let telemetry_dir = home_dir.join(".cagents").join("telemetry");
    fs::create_dir_all(&telemetry_dir)?;
    Ok(telemetry_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TelemetryConfig::default();
        assert!(config.enabled);
        assert!(!config.debug);
        assert!(config.mixpanel_token.is_none());
    }

    #[test]
    fn test_is_ci_detection() {
        // Save original env
        let original = env::var("CI").ok();

        env::set_var("CI", "true");
        assert!(is_ci());

        env::remove_var("CI");
        assert!(!is_ci());

        // Restore
        if let Some(val) = original {
            env::set_var("CI", val);
        }
    }
}
