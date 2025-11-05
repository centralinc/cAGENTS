# Telemetry Configuration Schema

## Overview

Telemetry configuration follows the same precedence hierarchy as project configuration:
1. User config (`~/.cagents/config.toml`) - lowest priority
2. Project config (`.cAGENTS/config.toml`) - medium priority
3. Local config (`.cAGENTS/config.local.toml`) - highest priority

## TOML Schema

```toml
[telemetry]
enabled = true           # Default: true (opt-out model)
debug = false            # Default: false (set true to inspect events without sending)
mixpanel_token = "..."   # Optional: override default token (for testing)
```

## Configuration Locations

### User-Level Config
**Path**: `~/.cagents/config.toml`

This allows users to disable telemetry globally across all projects:

```toml
[telemetry]
enabled = false
```

### Project-Level Config
**Path**: `.cAGENTS/config.toml`

Projects can disable telemetry for all contributors:

```toml
[telemetry]
enabled = false
```

### Local Override
**Path**: `.cAGENTS/config.local.toml`

Individual developers can override project settings:

```toml
[telemetry]
enabled = true
debug = true  # Inspect events without sending
```

## Environment Variables

The following environment variables take precedence over config files:

- `CAGENTS_TELEMETRY_DISABLED=1` - Disables telemetry completely
- `DO_NOT_TRACK=1` - Universal opt-out (respects https://consoledonottrack.com/)
- `CAGENTS_TELEMETRY_DEBUG=1` - Enables debug mode (prints events instead of sending)
- `CI=true` - Auto-disables telemetry in CI unless `CAGENTS_TELEMETRY_IN_CI=1` is set

## Precedence Order (Highest to Lowest)

1. `CAGENTS_TELEMETRY_DISABLED` env var
2. `DO_NOT_TRACK` env var
3. `CAGENTS_TELEMETRY_DEBUG` env var (sets debug mode)
4. Local config (`.cAGENTS/config.local.toml`)
5. Project config (`.cAGENTS/config.toml`)
6. User config (`~/.cagents/config.toml`)
7. Default (enabled=true, debug=false)

## CI Detection

In CI environments:
- Telemetry is **disabled by default**
- Can be explicitly enabled with `CAGENTS_TELEMETRY_IN_CI=1`
- Detected via `CI`, `CONTINUOUS_INTEGRATION`, or similar env vars

## Config Model in Rust

```rust
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Debug mode - print events instead of sending (default: false)
    #[serde(default)]
    pub debug: bool,

    /// Override Mixpanel token (for testing)
    pub mixpanel_token: Option<String>,
}

fn default_enabled() -> bool {
    true
}
```

## Storage Files

### Telemetry State Directory
**Path**: `~/.cagents/telemetry/`

Contains:
- `salt` - Random UUID for machine ID hashing
- `machine_id` - Cached anonymous machine ID
- `session.json` - Current LLM session state
- `notice_shown` - Flag to track first-run notice

### Session State Format
```json
{
  "llm_session_id": "uuid-here",
  "llm_type": "claude_code",
  "started_at": "2025-01-05T10:30:00Z",
  "command_count": 5,
  "last_command_at": "2025-01-05T10:35:42Z"
}
```

## CLI Commands

```bash
# Disable telemetry
cagents telemetry disable

# Enable telemetry
cagents telemetry enable

# Show status
cagents telemetry status
```

### Status Output Example
```
Telemetry: enabled
Machine ID: a3f8e9... (anonymized)
Debug Mode: off
Config Source: ~/.cagents/config.toml

Opt-out: cagents telemetry disable
Learn more: https://cagents.dev/telemetry
```

## Example Configurations

### Completely Disable Telemetry
```toml
# ~/.cagents/config.toml
[telemetry]
enabled = false
```

### Debug Mode (Inspect Events)
```toml
# .cAGENTS/config.local.toml
[telemetry]
debug = true
```

### Test with Custom Token
```toml
# .cAGENTS/config.local.toml
[telemetry]
mixpanel_token = "test_token_here"
```
