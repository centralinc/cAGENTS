//! Integration tests for telemetry system

use cagents_telemetry::{CommandEvent, TelemetryClient, TelemetryConfig};
use serial_test::serial;
use std::env;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
#[serial]
async fn test_client_tracks_command_with_llm_context() {
    // Set up LLM environment
    let original = env::var("CLAUDE_CODE_SESSION_ID").ok();
    env::set_var("CLAUDE_CODE_SESSION_ID", "test-session-123");

    // Create client
    let mut client = TelemetryClient::new().unwrap_or_default();

    // Track a command
    let mut event = CommandEvent::new(
        client.machine_id().to_string(),
        "test-session".to_string(),
        "build".to_string(),
        "0.5.1".to_string(),
    );
    event.duration_ms = 100;
    event.success = true;

    client.track_command(event);

    // Cleanup
    env::remove_var("CLAUDE_CODE_SESSION_ID");
    if let Some(val) = original {
        env::set_var("CLAUDE_CODE_SESSION_ID", val);
    }

    // Test passes if no panic
}

#[tokio::test]
#[serial]
async fn test_client_respects_disabled_config() {
    let original = env::var("CAGENTS_TELEMETRY_DISABLED").ok();
    env::set_var("CAGENTS_TELEMETRY_DISABLED", "1");

    let client = TelemetryClient::new().unwrap();

    assert!(!client.is_enabled());
    assert!(!client.is_debug());

    env::remove_var("CAGENTS_TELEMETRY_DISABLED");
    if let Some(val) = original {
        env::set_var("CAGENTS_TELEMETRY_DISABLED", val);
    }
}

#[tokio::test]
#[serial]
async fn test_client_enables_debug_mode() {
    let original = env::var("CAGENTS_TELEMETRY_DEBUG").ok();
    env::set_var("CAGENTS_TELEMETRY_DEBUG", "1");

    let client = TelemetryClient::new().unwrap();

    assert!(client.is_debug());

    env::remove_var("CAGENTS_TELEMETRY_DEBUG");
    if let Some(val) = original {
        env::set_var("CAGENTS_TELEMETRY_DEBUG", val);
    }
}

#[tokio::test]
#[serial]
async fn test_config_precedence_env_over_file() {
    // Create temp config that enables telemetry
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&temp_dir).unwrap();

    let config_dir = temp_dir.path().join(".cAGENTS");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"
[telemetry]
enabled = true
"#,
    )
    .unwrap();

    // But env var should disable it
    let original = env::var("CAGENTS_TELEMETRY_DISABLED").ok();
    env::set_var("CAGENTS_TELEMETRY_DISABLED", "1");

    let client = TelemetryClient::new().unwrap();
    assert!(!client.is_enabled());

    // Cleanup
    env::remove_var("CAGENTS_TELEMETRY_DISABLED");
    if let Some(val) = original {
        env::set_var("CAGENTS_TELEMETRY_DISABLED", val);
    }
    env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
#[serial]
async fn test_do_not_track_env_var() {
    let original = env::var("DO_NOT_TRACK").ok();
    env::set_var("DO_NOT_TRACK", "1");

    let client = TelemetryClient::new().unwrap();
    assert!(!client.is_enabled());

    env::remove_var("DO_NOT_TRACK");
    if let Some(val) = original {
        env::set_var("DO_NOT_TRACK", val);
    }
}

#[tokio::test]
#[serial]
async fn test_multiple_commands_in_llm_session() {
    let original = env::var("CLAUDE_CODE_SESSION_ID").ok();
    env::set_var("CLAUDE_CODE_SESSION_ID", "test-123");

    let mut client = TelemetryClient::new().unwrap();

    // Track first command
    let event1 = CommandEvent::new(
        client.machine_id().to_string(),
        "session1".to_string(),
        "build".to_string(),
        "0.5.1".to_string(),
    );
    client.track_command(event1);

    // Track second command
    let event2 = CommandEvent::new(
        client.machine_id().to_string(),
        "session2".to_string(),
        "preview".to_string(),
        "0.5.1".to_string(),
    );
    client.track_command(event2);

    // Cleanup
    env::remove_var("CLAUDE_CODE_SESSION_ID");
    if let Some(val) = original {
        env::set_var("CLAUDE_CODE_SESSION_ID", val);
    }

    // Test passes if no panic
}

#[tokio::test]
async fn test_client_with_custom_token() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&temp_dir).unwrap();

    let config_dir = temp_dir.path().join(".cAGENTS");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"
[telemetry]
mixpanel_token = "custom_test_token"
"#,
    )
    .unwrap();

    let client = TelemetryClient::new().unwrap();
    // Client should initialize successfully with custom token
    assert!(!client.machine_id().is_empty());

    env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_machine_id_is_consistent() {
    let client1 = TelemetryClient::new().unwrap_or_default();
    let client2 = TelemetryClient::new().unwrap_or_default();

    // Machine IDs should be the same
    assert_eq!(client1.machine_id(), client2.machine_id());
}

#[tokio::test]
async fn test_machine_id_format() {
    let client = TelemetryClient::new().unwrap_or_default();
    let machine_id = client.machine_id();

    // Should be a 64-character hex string (SHA256)
    assert_eq!(machine_id.len(), 64);
    assert!(machine_id.chars().all(|c| c.is_ascii_hexdigit()));
}
