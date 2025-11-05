//! LLM session detection and tracking
//!
//! Detects when cAGENTS is being used by LLM coding assistants like:
//! - Claude Code
//! - Cursor
//! - GitHub Copilot

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use uuid::Uuid;

use crate::config::get_telemetry_dir;

/// LLM type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LLMType {
    ClaudeCode,
    Cursor,
    Copilot,
    Unknown,
}

impl LLMType {
    pub fn as_str(&self) -> &str {
        match self {
            LLMType::ClaudeCode => "claude_code",
            LLMType::Cursor => "cursor",
            LLMType::Copilot => "copilot",
            LLMType::Unknown => "unknown",
        }
    }
}

/// LLM context detected
#[derive(Debug, Clone)]
pub struct LLMContext {
    pub session_id: String,
    pub llm_type: LLMType,
    pub detected_at: DateTime<Utc>,
}

/// LLM session state (persisted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMSession {
    pub llm_session_id: String,
    pub llm_type: String,
    pub started_at: DateTime<Utc>,
    pub command_count: u32,
    pub last_command_at: DateTime<Utc>,
    pub had_error: bool,
    pub retry_count: u32,
}

impl LLMSession {
    /// Load or create a new LLM session
    pub fn load_or_create(context: &LLMContext) -> Result<Self> {
        let session_path = get_telemetry_dir()?.join("session.json");

        // Try to load existing session
        if session_path.exists() {
            if let Ok(content) = fs::read_to_string(&session_path) {
                if let Ok(session) = serde_json::from_str::<Self>(&content) {
                    // Check if session is still valid (< 30 min old)
                    if !session.is_expired() {
                        return Ok(session);
                    }
                }
            }
        }

        // Create new session
        let session = Self {
            llm_session_id: context.session_id.clone(),
            llm_type: context.llm_type.as_str().to_string(),
            started_at: context.detected_at,
            command_count: 0,
            last_command_at: context.detected_at,
            had_error: false,
            retry_count: 0,
        };

        session.save()?;
        Ok(session)
    }

    /// Check if session is expired (> 30 minutes old)
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.last_command_at);
        elapsed > Duration::minutes(30)
    }

    /// Increment command count
    pub fn increment_command_count(&mut self) -> Result<()> {
        self.command_count += 1;
        self.last_command_at = Utc::now();
        self.save()
    }

    /// Mark that an error occurred
    pub fn mark_error(&mut self) -> Result<()> {
        self.had_error = true;
        self.retry_count += 1;
        self.save()
    }

    /// Get elapsed time since session start in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.started_at);
        elapsed.num_milliseconds() as u64
    }

    /// Save session to disk
    pub fn save(&self) -> Result<()> {
        let session_path = get_telemetry_dir()?.join("session.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(session_path, content)?;
        Ok(())
    }
}

/// Detect if running in an LLM context
pub fn detect_llm_context() -> Option<LLMContext> {
    // Check environment variables
    if env::var("CLAUDE_CODE_SESSION_ID").is_ok() {
        return Some(LLMContext {
            session_id: get_or_create_session_id(),
            llm_type: LLMType::ClaudeCode,
            detected_at: Utc::now(),
        });
    }

    if env::var("CURSOR_SESSION_ID").is_ok() {
        return Some(LLMContext {
            session_id: get_or_create_session_id(),
            llm_type: LLMType::Cursor,
            detected_at: Utc::now(),
        });
    }

    if env::var("GITHUB_COPILOT_CHAT_SESSION_ID").is_ok() {
        return Some(LLMContext {
            session_id: get_or_create_session_id(),
            llm_type: LLMType::Copilot,
            detected_at: Utc::now(),
        });
    }

    // Could add parent process detection here
    // For now, just check env vars

    None
}

/// Get or create an LLM session ID
fn get_or_create_session_id() -> String {
    // Try to load existing session ID
    if let Ok(telemetry_dir) = get_telemetry_dir() {
        let session_path = telemetry_dir.join("session.json");
        if session_path.exists() {
            if let Ok(content) = fs::read_to_string(&session_path) {
                if let Ok(session) = serde_json::from_str::<LLMSession>(&content) {
                    if !session.is_expired() {
                        return session.llm_session_id;
                    }
                }
            }
        }
    }

    // Create new session ID
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_llm_type_string() {
        assert_eq!(LLMType::ClaudeCode.as_str(), "claude_code");
        assert_eq!(LLMType::Cursor.as_str(), "cursor");
        assert_eq!(LLMType::Copilot.as_str(), "copilot");
        assert_eq!(LLMType::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_session_expiration() {
        let mut session = LLMSession {
            llm_session_id: "test".to_string(),
            llm_type: "claude_code".to_string(),
            started_at: Utc::now() - Duration::hours(1),
            command_count: 5,
            last_command_at: Utc::now() - Duration::minutes(31),
            had_error: false,
            retry_count: 0,
        };

        assert!(session.is_expired());

        session.last_command_at = Utc::now();
        assert!(!session.is_expired());
    }

    #[test]
    #[serial]
    fn test_detect_llm_context_no_env() {
        // Clean env
        env::remove_var("CLAUDE_CODE_SESSION_ID");
        env::remove_var("CURSOR_SESSION_ID");
        env::remove_var("GITHUB_COPILOT_CHAT_SESSION_ID");

        let context = detect_llm_context();
        assert!(context.is_none());
    }

    #[test]
    #[serial]
    fn test_detect_claude_code() {
        let original = env::var("CLAUDE_CODE_SESSION_ID").ok();
        env::set_var("CLAUDE_CODE_SESSION_ID", "test-session");

        let context = detect_llm_context();
        assert!(context.is_some());

        let ctx = context.unwrap();
        assert_eq!(ctx.llm_type, LLMType::ClaudeCode);
        assert!(!ctx.session_id.is_empty());

        env::remove_var("CLAUDE_CODE_SESSION_ID");
        if let Some(val) = original {
            env::set_var("CLAUDE_CODE_SESSION_ID", val);
        }
    }

    #[test]
    #[serial]
    fn test_detect_cursor() {
        let original = env::var("CURSOR_SESSION_ID").ok();
        env::set_var("CURSOR_SESSION_ID", "test-session");

        let context = detect_llm_context();
        assert!(context.is_some());

        let ctx = context.unwrap();
        assert_eq!(ctx.llm_type, LLMType::Cursor);

        env::remove_var("CURSOR_SESSION_ID");
        if let Some(val) = original {
            env::set_var("CURSOR_SESSION_ID", val);
        }
    }

    #[test]
    #[serial]
    fn test_detect_copilot() {
        let original = env::var("GITHUB_COPILOT_CHAT_SESSION_ID").ok();
        env::set_var("GITHUB_COPILOT_CHAT_SESSION_ID", "test-session");

        let context = detect_llm_context();
        assert!(context.is_some());

        let ctx = context.unwrap();
        assert_eq!(ctx.llm_type, LLMType::Copilot);

        env::remove_var("GITHUB_COPILOT_CHAT_SESSION_ID");
        if let Some(val) = original {
            env::set_var("GITHUB_COPILOT_CHAT_SESSION_ID", val);
        }
    }

    #[test]
    fn test_session_increment_command_count() {
        let mut session = LLMSession {
            llm_session_id: "test".to_string(),
            llm_type: "claude_code".to_string(),
            started_at: Utc::now(),
            command_count: 0,
            last_command_at: Utc::now(),
            had_error: false,
            retry_count: 0,
        };

        assert_eq!(session.command_count, 0);

        let _ = session.increment_command_count();
        assert_eq!(session.command_count, 1);

        let _ = session.increment_command_count();
        assert_eq!(session.command_count, 2);
    }

    #[test]
    fn test_session_mark_error() {
        let mut session = LLMSession {
            llm_session_id: "test".to_string(),
            llm_type: "claude_code".to_string(),
            started_at: Utc::now(),
            command_count: 0,
            last_command_at: Utc::now(),
            had_error: false,
            retry_count: 0,
        };

        assert!(!session.had_error);
        assert_eq!(session.retry_count, 0);

        let _ = session.mark_error();
        assert!(session.had_error);
        assert_eq!(session.retry_count, 1);

        let _ = session.mark_error();
        assert_eq!(session.retry_count, 2);
    }

    #[test]
    fn test_session_elapsed_ms() {
        let session = LLMSession {
            llm_session_id: "test".to_string(),
            llm_type: "claude_code".to_string(),
            started_at: Utc::now() - Duration::milliseconds(5000),
            command_count: 0,
            last_command_at: Utc::now(),
            had_error: false,
            retry_count: 0,
        };

        let elapsed = session.elapsed_ms();
        assert!(elapsed >= 5000);
        assert!(elapsed < 6000); // Should be around 5 seconds
    }

    #[test]
    fn test_session_serialization() {
        let session = LLMSession {
            llm_session_id: "test-id".to_string(),
            llm_type: "cursor".to_string(),
            started_at: Utc::now(),
            command_count: 42,
            last_command_at: Utc::now(),
            had_error: true,
            retry_count: 3,
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: LLMSession = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.llm_session_id, "test-id");
        assert_eq!(deserialized.llm_type, "cursor");
        assert_eq!(deserialized.command_count, 42);
        assert_eq!(deserialized.retry_count, 3);
        assert!(deserialized.had_error);
    }
}
