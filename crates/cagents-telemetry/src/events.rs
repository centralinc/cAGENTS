//! Telemetry event data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Command execution event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEvent {
    // Event metadata
    pub event_type: String, // "command_executed"
    pub timestamp: DateTime<Utc>,

    // Anonymous identifiers
    pub machine_id: String,
    pub session_id: String, // UUID for this CLI session

    // Command info
    pub command: String, // "build" | "preview" | "init" | "render"
    pub subcommand: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub flags: Vec<String>, // Sanitized flag names only

    // Execution metadata
    pub duration_ms: u64,
    pub success: bool,
    pub error_type: Option<String>, // e.g. "ConfigNotFound", "RenderError"

    // Environment
    pub cli_version: String,
    pub os: String,   // "linux" | "macos" | "windows"
    pub arch: String, // "x64" | "arm64"
    pub is_ci: bool,

    // LLM context (if detected)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_session: Option<LLMSessionContext>,
}

impl CommandEvent {
    /// Create a new command event with defaults
    pub fn new(
        machine_id: String,
        session_id: String,
        command: String,
        cli_version: String,
    ) -> Self {
        Self {
            event_type: "command_executed".to_string(),
            timestamp: Utc::now(),
            machine_id,
            session_id,
            command,
            subcommand: None,
            flags: Vec::new(),
            duration_ms: 0,
            success: true,
            error_type: None,
            cli_version,
            os: get_os_string(),
            arch: get_arch_string(),
            is_ci: is_ci_env(),
            llm_session: None,
        }
    }
}

/// LLM session context for tracking LLM usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMSessionContext {
    // Session tracking
    pub llm_session_id: String, // Links multiple commands in one LLM session
    pub llm_type: String, // "claude_code" | "cursor" | "copilot" | "unknown"

    // Call pattern
    pub command_index: u32,              // nth command in this LLM session
    pub time_since_session_start_ms: u64, // milliseconds

    // Interaction pattern (anonymized)
    pub had_error_in_session: bool,
    pub retry_count: u32, // How many times LLM retried after errors
}

/// Error event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub event_type: String, // "error"
    pub timestamp: DateTime<Utc>,
    pub machine_id: String,
    pub session_id: String,

    // Error details (anonymized)
    pub error_type: String,     // "ConfigParseError", "TemplateNotFound"
    pub error_category: String, // "config" | "template" | "engine" | "io"
    pub command_context: String, // What command failed
}

impl ErrorEvent {
    /// Create a new error event
    pub fn new(
        machine_id: String,
        session_id: String,
        error_type: String,
        error_category: String,
        command_context: String,
    ) -> Self {
        Self {
            event_type: "error".to_string(),
            timestamp: Utc::now(),
            machine_id,
            session_id,
            error_type,
            error_category,
            command_context,
        }
    }
}

/// Feature usage event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureEvent {
    pub event_type: String, // "feature_used"
    pub timestamp: DateTime<Utc>,
    pub machine_id: String,
    pub session_id: String,

    pub feature: String,        // "custom_engine", "when_clause", "output_targets"
    pub value: Option<String>, // Anonymized metadata
}

impl FeatureEvent {
    /// Create a new feature usage event
    pub fn new(
        machine_id: String,
        session_id: String,
        feature: String,
        value: Option<String>,
    ) -> Self {
        Self {
            event_type: "feature_used".to_string(),
            timestamp: Utc::now(),
            machine_id,
            session_id,
            feature,
            value,
        }
    }
}

/// Convert event to Mixpanel format
pub trait ToMixpanelEvent {
    fn to_mixpanel(&self, distinct_id: &str) -> (String, HashMap<String, serde_json::Value>);
}

impl ToMixpanelEvent for CommandEvent {
    fn to_mixpanel(&self, distinct_id: &str) -> (String, HashMap<String, serde_json::Value>) {
        let mut properties = HashMap::new();

        properties.insert(
            "distinct_id".to_string(),
            serde_json::Value::String(distinct_id.to_string()),
        );
        properties.insert(
            "time".to_string(),
            serde_json::Value::Number(self.timestamp.timestamp().into()),
        );
        properties.insert(
            "session_id".to_string(),
            serde_json::Value::String(self.session_id.clone()),
        );
        properties.insert(
            "command".to_string(),
            serde_json::Value::String(self.command.clone()),
        );
        if let Some(subcommand) = &self.subcommand {
            properties.insert(
                "subcommand".to_string(),
                serde_json::Value::String(subcommand.clone()),
            );
        }
        properties.insert(
            "duration_ms".to_string(),
            serde_json::Value::Number(self.duration_ms.into()),
        );
        properties.insert(
            "success".to_string(),
            serde_json::Value::Bool(self.success),
        );
        if let Some(error) = &self.error_type {
            properties.insert(
                "error_type".to_string(),
                serde_json::Value::String(error.clone()),
            );
        }
        properties.insert(
            "cli_version".to_string(),
            serde_json::Value::String(self.cli_version.clone()),
        );
        properties.insert(
            "os".to_string(),
            serde_json::Value::String(self.os.clone()),
        );
        properties.insert(
            "arch".to_string(),
            serde_json::Value::String(self.arch.clone()),
        );
        properties.insert("is_ci".to_string(), serde_json::Value::Bool(self.is_ci));

        if let Some(llm) = &self.llm_session {
            properties.insert(
                "llm_session_id".to_string(),
                serde_json::Value::String(llm.llm_session_id.clone()),
            );
            properties.insert(
                "llm_type".to_string(),
                serde_json::Value::String(llm.llm_type.clone()),
            );
            properties.insert(
                "llm_command_index".to_string(),
                serde_json::Value::Number(llm.command_index.into()),
            );
            properties.insert(
                "llm_time_since_start_ms".to_string(),
                serde_json::Value::Number(llm.time_since_session_start_ms.into()),
            );
        }

        ("command_executed".to_string(), properties)
    }
}

/// Get OS string
fn get_os_string() -> String {
    if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "windows") {
        "windows".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Get architecture string
fn get_arch_string() -> String {
    if cfg!(target_arch = "x86_64") {
        "x64".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "arm64".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Check if running in CI
fn is_ci_env() -> bool {
    std::env::var("CI").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_event_creation() {
        let event = CommandEvent::new(
            "test_machine".to_string(),
            "test_session".to_string(),
            "build".to_string(),
            "0.5.1".to_string(),
        );

        assert_eq!(event.command, "build");
        assert_eq!(event.event_type, "command_executed");
        assert!(event.success);
        assert_eq!(event.cli_version, "0.5.1");
    }

    #[test]
    fn test_mixpanel_conversion() {
        let event = CommandEvent::new(
            "test_machine".to_string(),
            "test_session".to_string(),
            "preview".to_string(),
            "0.5.1".to_string(),
        );

        let (event_name, properties) = event.to_mixpanel("test_machine");
        assert_eq!(event_name, "command_executed");
        assert!(properties.contains_key("command"));
        assert!(properties.contains_key("cli_version"));
    }
}
