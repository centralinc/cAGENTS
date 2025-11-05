//! # cAGENTS Telemetry
//!
//! Privacy-first telemetry system for cAGENTS CLI.
//!
//! ## Privacy Guarantees
//!
//! - **Completely Anonymous**: No PII, no IP addresses, no identifiable data
//! - **Transparent**: Clear documentation of what's collected
//! - **Opt-out First**: Easy to disable with multiple methods
//! - **Fail Gracefully**: Never blocks or slows down the CLI
//! - **Debug Mode**: Inspect events before they're sent
//!
//! ## What We Collect
//!
//! - Command names and flags (sanitized)
//! - Success/failure status and duration
//! - Error types (anonymized)
//! - Platform/OS/architecture
//! - CLI version
//! - Anonymized machine ID (salted hash)
//! - LLM session context (when detected)
//!
//! ## What We Never Collect
//!
//! - Code content or file paths
//! - Environment variables
//! - API keys or secrets
//! - User names or personal info
//! - IP addresses (Mixpanel anonymization enabled)
//!
//! ## Opt-Out
//!
//! ```bash
//! # Via CLI
//! cagents telemetry disable
//!
//! # Via environment variable
//! export CAGENTS_TELEMETRY_DISABLED=1
//!
//! # Via config file (~/.cagents/config.toml)
//! [telemetry]
//! enabled = false
//! ```

pub mod client;
pub mod config;
pub mod events;
pub mod llm_detection;
pub mod machine_id;
pub mod transport;

pub use client::TelemetryClient;
pub use config::TelemetryConfig;
pub use events::{CommandEvent, ErrorEvent, FeatureEvent, LLMSessionContext};
pub use llm_detection::{detect_llm_context, LLMContext, LLMType};
pub use machine_id::get_or_generate_machine_id;

/// Re-export common types
pub type Result<T> = anyhow::Result<T>;
