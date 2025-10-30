# cAGENTS

**Adaptable instruction for codegen models across humans and their machines.**

Generate `AGENTS.md`, `CLAUDE.md`, and tool-specific rules from composable templates. Context-aware, scope-sensitive, DRY.

Inspired by the `AGENTS.md` and `CLAUDE.md` conventions, cAGENTS makes instruction files adapt to your codebase structure—supporting Claude Code, Cursor, Cline, Windsurf, Codex, Aider, and any tool that reads these formats.

```
⚠️ Software in Alpha - Core features work, but expect rough edges and breaking changes.
```

---

## Quick Start

```bash
# Install
npm install --save-dev cagents

# Initialize
npx cagents init

# Create templates in .cAGENTS/templates/
# Build outputs
npx cagents build
```

---

## Example: Conditional Rules

Here's a simple example that inspired cAGENTS:

**Config** (`.cAGENTS/config.toml`):
```toml
[output]
targets = ["agents-md", "claude-md"]

[defaults]
engine = "builtin:simple"

[variables.command]
use_beads = "command -v bd >/dev/null 2>&1 && echo true || echo false"
```

**Template** (`.cAGENTS/templates/beads.md`):
```markdown
---
name: beads
when:
  use_beads: "true"
order: 20
---

## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd** for ALL issue tracking.

Quick commands:
- Check ready work: `bd ready --json`
- Create issue: `bd create "Title" -t bug|feature -p 0-4`
- Update status: `bd update bd-42 --status in_progress`
```

**How it works:**
- The `use_beads` variable runs a shell command checking if `bd` is installed
- If `bd` is available → template included in AGENTS.md
- If not → template skipped
- Same templates, different contexts

**Use cases:**
- Developers with `bd` installed get full tracking instructions
- Cloud agents without `bd` get clean AGENTS.md without the noise
- New contributors see only what's relevant to their environment

---

## Core Concepts

### Templates

Markdown files with YAML frontmatter in `.cAGENTS/templates/`:

```markdown
---
name: typescript-rules
order: 10
---
# TypeScript Guidelines

Use strict mode. Prefer functional components.
```

### Conditional Rendering

```yaml
---
when:
  language: ["typescript"]  # Language-specific
  role: ["backend"]         # Role-specific
  use_beads: "true"         # Tool availability
---
```

### Multiple Outputs

```toml
[output]
targets = ["agents-md", "claude-md", "cursorrules"]
```

One build, three formats.

### Nested Files

```yaml
---
globs: ["packages/api/**"]
outputIn: "matched"
---
```

Creates `packages/api/AGENTS.md` with package-specific rules.

---

## Migration

Already have rules?

```bash
npx cagents migrate
```

Converts AGENTS.md, CLAUDE.md, .cursorrules, or .cursor/rules/ into templates.

---

## Recommended Setup

> **🎯 Best Practice:** Don't commit generated files. Let cAGENTS rebuild them on every install.

This ensures everyone (and every environment) gets the right context automatically.

### Step 1: Gitignore Generated Files

```bash
npx cagents git ignore-outputs
```

This adds to `.gitignore`:
```gitignore
# cAGENTS outputs (generated, not committed)
AGENTS.md
CLAUDE.md
.cursorrules
**/AGENTS.md
**/CLAUDE.md
```

### Step 2: Auto-Build on Install

**package.json:**
```json
{
  "scripts": {
    "postinstall": "cagents build"
  }
}
```

**pnpm:**
```json
{
  "scripts": {
    "prepare": "cagents build"
  }
}
```

### Step 3: Delete Generated Files from Git

If you've already committed AGENTS.md:

```bash
git rm AGENTS.md CLAUDE.md .cursorrules
git rm -r --cached '**/AGENTS.md' '**/CLAUDE.md'  # Remove nested files
git commit -m "chore: remove generated files, now built via cagents"
```

### Result

- ✅ **Templates** are version controlled (`.cAGENTS/templates/`)
- ✅ **Generated files** are rebuilt on every `npm install`
- ✅ **Context** is always fresh and environment-specific
- ✅ **No merge conflicts** on generated files

---

## Advanced

For advanced features:
- Custom template engines (Handlebars, Jinja2, Liquid)
- Complex glob patterns
- Output strategies
- Monorepo configurations

See [ADVANCED.md](./ADVANCED.md)

---

## Commands

Quick reference:
- `cagents init` - Initialize .cAGENTS
- `cagents build` - Generate outputs
- `cagents migrate` - Import existing rules
- `cagents lint` - Validate config
- `cagents preview` - Preview build
- `cagents render <file>` - Render context for file
- `cagents context <file>` - Show metadata for file

See [COMMANDS.md](./COMMANDS.md) for full reference.

---

## Why?

### The Problem: Static Instruction Files Don't Scale

The `AGENTS.md` and `CLAUDE.md` conventions solved a real problem—giving AI coding assistants project-specific context. But static files have limits:

- **Global rules apply everywhere**, even when irrelevant
- **Manual context doesn't scale** across monorepos or complex codebases
- **Duplication is required** for similar-but-different rules across directories
- **No adaptation** based on file type, role, or available tools

Every developer and AI sees the same rules, regardless of:
- Tools available (linters, issue trackers, formatters)
- Project area (frontend vs backend vs infrastructure)
- File type (TypeScript vs Rust vs Python)

### The Solution: Adaptable Instruction

cAGENTS treats instruction as **composable infrastructure**:

- **Scope-sensitive** - Different rules for different files/directories
- **Template-driven** - DRY, composable, maintainable
- **Multi-format** - AGENTS.md, CLAUDE.md, .cursorrules from one source
- **Cross-tool** - Works with any tool reading these conventions (Claude Code, Cursor, Cline, Windsurf, Codex, Aider)
- **Monorepo-friendly** - Per-package rules without duplication

Like "Infrastructure as Code" but for AI instruction—**"Instruction as Code"**.

---

## Development & RFCs

### RFCs (Request for Comments)

Major features are designed through RFCs in `docs/rfcs/`. Current RFCs for v0.2.0:

- **[RFC 0001: Claude Skills Support](./docs/rfcs/0001-claude-skills-support.md)** - Generate dynamic skill files for Claude's on-demand loading
- **[RFC 0002: Progressive Disclosure](./docs/rfcs/0002-progressive-disclosure.md)** - Reduce context overload with structured outputs
- **[RFC 0003: Multi-Format Consistency](./docs/rfcs/0003-multi-format-consistency.md)** - Adopt AGENTS.md as primary standard
- **[RFC 0004: CLI Enhancements](./docs/rfcs/0004-cli-enhancements.md)** - New `context` command, improved `render`, caching

See [docs/rfcs/README.md](./docs/rfcs/README.md) for the RFC process and how to propose features.

### Contributing

Interested in contributing?

1. Review RFCs in `docs/rfcs/`
2. Follow TDD workflow (test first, then implement)
3. See [CLAUDE.md](./CLAUDE.md) for full development guidelines

---

## License

MIT

---
