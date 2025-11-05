# cagents

## 0.6.0

### Minor Changes

- c65c0f8: Added privacy-first anonymous telemetry system with Mixpanel integration:

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

## 0.5.1

### Patch Changes

- 341d4a2: 1. Changed Linux build target from x86_64-unknown-linux-gnu (dynamic GLIBC) to x86_64-unknown-linux-musl (static musl) in both workflows 2. Added musl-tools installation step before building the Linux binary 3. Updated artifact paths to match the new target name

## 0.5.0

### Minor Changes

- d52df8d: clean up for launch

## 0.4.1

### Patch Changes

- 686ecb1: Fix nested file detection in multi-format migration. When using 'merge all formats' option, nested AGENTS.md and CLAUDE.md files are now properly discovered and imported, matching the behavior of single-format migration.
- 4c77de2: fix deploy

## 0.4.0

### Minor Changes

- e030236: Version reset and workflow improvements. This release includes:

  - Reset version to 0.4.0 following proper semver from 0.3.3
  - Fixed canary releases to include correct version in compiled binaries
  - Improved GitHub workflows to use centralbot with proper permissions
  - Enhanced version syncing between npm package and Rust binaries
  - All future canary and stable releases will show correct version in `--version` output

## 0.1.0

### Major Changes

- Version reset to 0.1.0 for alpha release cycle. Configured changesets for beta/RC releases.
