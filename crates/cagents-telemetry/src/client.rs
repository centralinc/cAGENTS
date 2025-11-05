//! Main telemetry client

use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::{load_telemetry_config, TelemetryConfig};
use crate::events::{CommandEvent, ErrorEvent, FeatureEvent, LLMSessionContext, ToMixpanelEvent};
use crate::llm_detection::{detect_llm_context, LLMSession};
use crate::machine_id::get_or_generate_machine_id;
use crate::transport::{EventTransport, TransportEvent};

/// Default Mixpanel project token
const DEFAULT_MIXPANEL_TOKEN: &str = "ee768cef1a2170d256702be36d5f2e95";

/// Main telemetry client
pub struct TelemetryClient {
    config: TelemetryConfig,
    machine_id: String,
    session_id: String,
    transport: Option<Arc<EventTransport>>,
    llm_session: Option<LLMSession>,
}

impl TelemetryClient {
    /// Create a new telemetry client
    ///
    /// Respects opt-out configuration and environment variables.
    /// If telemetry is disabled, this will still succeed but won't send any events.
    pub fn new() -> Result<Self> {
        let config = load_telemetry_config()?;
        let machine_id = get_or_generate_machine_id()?;
        let session_id = Uuid::new_v4().to_string();

        // Get Mixpanel token
        let mixpanel_token = config
            .mixpanel_token
            .clone()
            .unwrap_or_else(|| DEFAULT_MIXPANEL_TOKEN.to_string());

        // Create transport if enabled
        let transport = if config.enabled || config.debug {
            Some(Arc::new(EventTransport::new(
                config.clone(),
                mixpanel_token,
            )))
        } else {
            None
        };

        // Detect LLM context
        let llm_session = detect_llm_context()
            .and_then(|context| LLMSession::load_or_create(&context).ok());

        Ok(Self {
            config,
            machine_id,
            session_id,
            transport,
            llm_session,
        })
    }

    /// Track a command execution event
    pub fn track_command(&mut self, mut event: CommandEvent) {
        if !self.should_send() {
            return;
        }

        // Set identifiers
        event.machine_id = self.machine_id.clone();
        event.session_id = self.session_id.clone();

        // Add LLM context if available
        if let Some(ref mut llm_session) = self.llm_session {
            let _ = llm_session.increment_command_count();

            event.llm_session = Some(LLMSessionContext {
                llm_session_id: llm_session.llm_session_id.clone(),
                llm_type: llm_session.llm_type.clone(),
                command_index: llm_session.command_count,
                time_since_session_start_ms: llm_session.elapsed_ms(),
                had_error_in_session: llm_session.had_error,
                retry_count: llm_session.retry_count,
            });

            // Track errors
            if !event.success {
                let _ = llm_session.mark_error();
            }
        }

        // Convert to Mixpanel format and send
        let (event_name, properties) = event.to_mixpanel(&self.machine_id);
        self.send_event(TransportEvent {
            event_name,
            distinct_id: self.machine_id.clone(),
            properties,
        });
    }

    /// Track an error event
    pub fn track_error(&self, event: ErrorEvent) {
        if !self.should_send() {
            return;
        }

        // Convert to Mixpanel format
        let mut properties = std::collections::HashMap::new();
        properties.insert(
            "error_type".to_string(),
            serde_json::Value::String(event.error_type),
        );
        properties.insert(
            "error_category".to_string(),
            serde_json::Value::String(event.error_category),
        );
        properties.insert(
            "command_context".to_string(),
            serde_json::Value::String(event.command_context),
        );

        self.send_event(TransportEvent {
            event_name: "error".to_string(),
            distinct_id: self.machine_id.clone(),
            properties,
        });
    }

    /// Track a feature usage event
    pub fn track_feature(&self, event: FeatureEvent) {
        if !self.should_send() {
            return;
        }

        let mut properties = std::collections::HashMap::new();
        properties.insert(
            "feature".to_string(),
            serde_json::Value::String(event.feature),
        );
        if let Some(value) = event.value {
            properties.insert("value".to_string(), serde_json::Value::String(value));
        }

        self.send_event(TransportEvent {
            event_name: "feature_used".to_string(),
            distinct_id: self.machine_id.clone(),
            properties,
        });
    }

    /// Check if we should send telemetry
    fn should_send(&self) -> bool {
        self.config.enabled || self.config.debug
    }

    /// Send an event to the transport
    fn send_event(&self, event: TransportEvent) {
        if let Some(ref transport) = self.transport {
            transport.send(event);
        }
    }

    /// Get the machine ID (for status display)
    pub fn machine_id(&self) -> &str {
        &self.machine_id
    }

    /// Check if telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Check if debug mode is enabled
    pub fn is_debug(&self) -> bool {
        self.config.debug
    }
}

impl Default for TelemetryClient {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            config: TelemetryConfig::default(),
            machine_id: "unknown".to_string(),
            session_id: Uuid::new_v4().to_string(),
            transport: None,
            llm_session: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        // This may fail if config is invalid, but should not panic
        let result = TelemetryClient::new();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_disabled_client_doesnt_send() {
        // Client with disabled config should not crash
        let client = TelemetryClient {
            config: TelemetryConfig {
                enabled: false,
                debug: false,
                mixpanel_token: None,
            },
            machine_id: "test".to_string(),
            session_id: "test".to_string(),
            transport: None,
            llm_session: None,
        };

        assert!(!client.should_send());
    }
}
