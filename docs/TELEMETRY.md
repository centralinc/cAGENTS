# Telemetry in cAGENTS

cAGENTS collects **completely anonymous** telemetry data to help us understand how the tool is being used and improve it for everyone. This page explains exactly what we collect, how we protect your privacy, and how to opt out.

---

## 🔒 Privacy Guarantees

### What We Collect

- **Command names** (e.g., `build`, `preview`, `init`)
- **Command flags** (sanitized, e.g., `--dry-run`, `--force`)
- **Execution metrics** (duration, success/failure status)
- **Error types** (categorized, anonymized - e.g., "ConfigNotFound")
- **Platform information** (OS, architecture)
- **CLI version** (e.g., "0.5.1")
- **Anonymous machine ID** (salted SHA256 hash, unlinkable to you)
- **LLM session context** (when detected - see below)

### What We NEVER Collect

❌ **Code content** - We never see your code
❌ **File paths** - We don't know your file structure
❌ **Environment variables** - Including API keys, tokens, secrets
❌ **Personal information** - No usernames, emails, or names
❌ **IP addresses** - Mixpanel IP anonymization is enabled
❌ **Project names** - We don't know what you're working on

---

## 🤖 LLM Session Tracking

When cAGENTS detects it's being used by an LLM coding assistant (like Claude Code, Cursor, or GitHub Copilot), we track additional **completely anonymous** session metrics:

### What We Track

- **LLM type** (`claude_code`, `cursor`, `copilot`)
- **Session ID** (random UUID, links commands in one session)
- **Command count** (how many commands in this session)
- **Time since session start** (in milliseconds)
- **Error/retry patterns** (did the LLM retry after errors)

### What This Tells Us

This helps us understand:
- How LLMs use cAGENTS differently from humans
- Which commands LLMs struggle with (high retry rates)
- Typical LLM workflow patterns
- Where to focus improvements for AI assistance

### Privacy Notes

- LLM sessions are identified by a random UUID (not linkable to you or your project)
- We never collect the LLM's prompts, responses, or conversations
- Sessions expire after 30 minutes of inactivity
- Detection is based on environment variables only

---

## 🔍 Inspect Events (Debug Mode)

Want to see exactly what's being sent? Enable debug mode:

```bash
# Print events to stderr instead of sending them
export CAGENTS_TELEMETRY_DEBUG=1
cagents build

# Example output:
# 📊 Telemetry Event (DEBUG MODE - not sent):
# {
#   "event": "command_executed",
#   "distinct_id": "abc123...",
#   "properties": {
#     "command": "build",
#     "duration_ms": 245,
#     "success": true,
#     "cli_version": "0.5.1",
#     "os": "macos",
#     ...
#   }
# }
```

---

## ⛔ Opt-Out (Multiple Methods)

Telemetry is **opt-out** by default. We respect your choice and make it easy to disable:

### Method 1: CLI Command (Recommended)

```bash
# Disable telemetry globally
cagents telemetry disable

# Re-enable if you change your mind
cagents telemetry enable

# Check current status
cagents telemetry status
```

### Method 2: Environment Variable

```bash
# Disable for this session
export CAGENTS_TELEMETRY_DISABLED=1

# Or use the universal opt-out standard
export DO_NOT_TRACK=1
```

### Method 3: User Config File

Edit `~/.cagents/config.toml`:

```toml
[telemetry]
enabled = false
```

### Method 4: Project Config

Disable for all contributors in `.cAGENTS/config.toml`:

```toml
[telemetry]
enabled = false
```

### Method 5: Local Override

Override in `.cAGENTS/config.local.toml`:

```toml
[telemetry]
enabled = false  # or true to override project settings
```

---

## 🔄 Configuration Precedence

Settings are applied in this order (highest to lowest priority):

1. `CAGENTS_TELEMETRY_DISABLED=1` environment variable
2. `DO_NOT_TRACK=1` environment variable
3. `CAGENTS_TELEMETRY_DEBUG=1` (enables debug mode)
4. Local config (`.cAGENTS/config.local.toml`)
5. Project config (`.cAGENTS/config.toml`)
6. User config (`~/.cagents/config.toml`)
7. Default (enabled)

---

## 🏗️ CI/CD Behavior

**Telemetry is automatically disabled in CI environments** to avoid polluting analytics with build server data.

Detected CI environments:
- GitHub Actions
- GitLab CI
- CircleCI
- Travis CI
- Any environment with `CI=true`

To explicitly enable telemetry in CI:
```bash
export CAGENTS_TELEMETRY_IN_CI=1
```

---

## 📊 Example Events

### Command Execution Event

```json
{
  "event": "command_executed",
  "properties": {
    "command": "build",
    "duration_ms": 1234,
    "success": true,
    "cli_version": "0.5.1",
    "os": "macos",
    "arch": "arm64",
    "is_ci": false,
    "session_id": "uuid-here",
    "distinct_id": "abc123..." // anonymous machine ID
  }
}
```

### Command with LLM Context

```json
{
  "event": "command_executed",
  "properties": {
    "command": "preview",
    "llm_session_id": "different-uuid",
    "llm_type": "claude_code",
    "llm_command_index": 5,
    "llm_time_since_start_ms": 42000,
    ...
  }
}
```

### Error Event

```json
{
  "event": "error",
  "properties": {
    "error_type": "ConfigNotFound",
    "error_category": "config",
    "command_context": "build"
  }
}
```

---

## 🛡️ Technical Implementation

### Anonymous Machine ID

Your machine ID is generated using:

1. **Stable identifier**: MAC address (or hostname if MAC unavailable)
2. **Random salt**: UUID stored in `~/.cagents/telemetry/salt`
3. **Hashing**: SHA256(salt + identifier)

This ensures:
- The ID is stable for your machine
- The ID cannot be reverse-engineered to identify you
- Different machines with the same MAC have different IDs
- The salt is unique to you and never shared

### Data Storage

Telemetry data is stored locally in `~/.cagents/telemetry/`:

- `salt` - Random UUID for hashing
- `machine_id` - Cached anonymous machine ID (SHA256 hash)
- `session.json` - Current LLM session state (if active)

No telemetry data is stored outside your machine except the events sent to Mixpanel.

### Event Transport

- Events are queued asynchronously (never blocks the CLI)
- Batched every 10 events or 5 seconds (whichever comes first)
- 2-second timeout for sends
- Fails gracefully - network errors never affect CLI operation

---

## 🔐 What We Use the Data For

Telemetry helps us understand:

- **Most-used commands** → Prioritize improvements
- **Error patterns** → Fix bugs faster
- **Platform distribution** → Optimize cross-platform support
- **LLM usage** → Improve AI assistant experience
- **Performance metrics** → Identify slow operations

We do NOT use telemetry for:
- Tracking individuals
- Marketing or sales purposes
- Selling or sharing data with third parties

---

## ❓ FAQ

### Why is telemetry opt-out instead of opt-in?

Following industry standards (Next.js, Turborepo, Homebrew), opt-out telemetry provides more accurate usage data while still respecting user privacy. We make opting out very easy and clearly document what's collected.

### Can I see what data is sent?

Yes! Use debug mode:
```bash
CAGENTS_TELEMETRY_DEBUG=1 cagents <command>
```

This prints events to stderr instead of sending them.

### How do I know my IP address isn't collected?

1. Mixpanel's IP anonymization is enabled on our project
2. You can verify in debug mode - the event has no IP field
3. Our code never accesses or logs IP addresses

### What happens if I opt out?

The telemetry client is still initialized (to support the `telemetry status` command), but no events are sent. The overhead is negligible (<1ms).

### Will this slow down my builds?

No. Event sending is asynchronous and batched. Telemetry adds <5ms overhead per command in the worst case.

### What if telemetry has a network error?

The CLI continues normally. Network errors are silently ignored and never affect your workflow.

### Can I use a different Mixpanel project token?

Yes, for testing:

```toml
# .cAGENTS/config.local.toml
[telemetry]
mixpanel_token = "your_test_token"
```

### How do I verify telemetry is disabled?

```bash
cagents telemetry status
# Shows: Enabled: no
```

---

## 📞 Questions or Concerns?

If you have questions about telemetry or privacy concerns:

- Open an issue: https://github.com/centralinc/cagents/issues
- Read the source code: `crates/cagents-telemetry/`
- Check this doc: You're reading it!

We're committed to transparency and privacy. If something seems wrong, please let us know.

---

## 🙏 Thank You

By allowing anonymous telemetry, you help us make cAGENTS better for everyone. We genuinely appreciate it!

If you choose to opt out, that's completely fine too. cAGENTS will continue to work perfectly.
