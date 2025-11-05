//! Async event transport with batching
//!
//! Sends telemetry events to Mixpanel in batches to minimize overhead

use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::config::TelemetryConfig;

/// Telemetry event for transport
#[derive(Debug, Clone)]
pub struct TransportEvent {
    pub event_name: String,
    pub distinct_id: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Async event transport with batching
pub struct EventTransport {
    sender: mpsc::UnboundedSender<TransportEvent>,
    config: TelemetryConfig,
}

impl EventTransport {
    /// Create a new event transport
    pub fn new(config: TelemetryConfig, mixpanel_token: String) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn background task for batching
        let config_clone = config.clone();
        tokio::spawn(async move {
            Self::batch_sender(rx, mixpanel_token, config_clone).await;
        });

        Self {
            sender: tx,
            config,
        }
    }

    /// Send an event (non-blocking)
    pub fn send(&self, event: TransportEvent) {
        if self.config.debug {
            // In debug mode, print instead of sending
            eprintln!("📊 Telemetry Event (DEBUG MODE - not sent):");
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "event": event.event_name,
                    "distinct_id": event.distinct_id,
                    "properties": event.properties,
                }))
                .unwrap_or_default()
            );
            return;
        }

        // Send to queue (fails silently if queue is full)
        let _ = self.sender.send(event);
    }

    /// Background task that batches and sends events
    async fn batch_sender(
        mut rx: mpsc::UnboundedReceiver<TransportEvent>,
        mixpanel_token: String,
        config: TelemetryConfig,
    ) {
        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap_or_default();

        let mut batch: Vec<TransportEvent> = Vec::new();
        let mut tick_interval = interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    batch.push(event);

                    // Send if batch is full (10 events)
                    if batch.len() >= 10 {
                        Self::send_batch(&client, &mixpanel_token, &batch, &config).await;
                        batch.clear();
                    }
                }
                _ = tick_interval.tick() => {
                    // Send batch every 5 seconds if we have events
                    if !batch.is_empty() {
                        Self::send_batch(&client, &mixpanel_token, &batch, &config).await;
                        batch.clear();
                    }
                }
            }
        }
    }

    /// Send a batch of events to Mixpanel
    async fn send_batch(
        client: &Client,
        token: &str,
        events: &[TransportEvent],
        config: &TelemetryConfig,
    ) {
        if events.is_empty() || !config.enabled {
            return;
        }

        // Convert to Mixpanel format
        let mixpanel_events: Vec<serde_json::Value> = events
            .iter()
            .map(|e| {
                let mut props = e.properties.clone();
                props.insert("token".to_string(), json!(token));
                props.insert("time".to_string(), json!(chrono::Utc::now().timestamp()));

                json!({
                    "event": e.event_name,
                    "properties": props
                })
            })
            .collect();

        // Send to Mixpanel (fail silently)
        let result = client
            .post("https://api.mixpanel.com/track")
            .json(&mixpanel_events)
            .send()
            .await;

        // Log errors in debug mode only
        if config.debug {
            if let Err(e) = result {
                eprintln!("⚠️  Telemetry send error: {}", e);
            }
        }

        // Always fail silently - never block the CLI
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_creation() {
        let config = TelemetryConfig {
            enabled: true,
            debug: false,
            mixpanel_token: None,
        };

        // This test just ensures the transport can be created
        // We can't test async sending in a unit test easily
        let _transport = EventTransport::new(config, "test_token".to_string());
    }
}
