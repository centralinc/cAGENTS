---
"cagents": minor
---

Added privacy-first anonymous telemetry system with Mixpanel integration:

- **Completely anonymous** - SHA256-hashed machine IDs, no PII collected
- **LLM session tracking** - Detects Claude Code, Cursor, Copilot usage patterns
- **Multiple opt-out methods** - CLI command, env vars, config files, DO_NOT_TRACK support
- **Debug mode** - Inspect events before sending with `CAGENTS_TELEMETRY_DEBUG=1`
- **Auto-disabled in CI** - Respects CI environments unless explicitly enabled
- **Async batching** - Non-blocking event sending, <5ms overhead

New commands:
- `cagents telemetry enable` - Enable telemetry
- `cagents telemetry disable` - Disable telemetry
- `cagents telemetry status` - Show current telemetry settings

See docs/TELEMETRY.md for complete privacy details and opt-out instructions.
