# cAGENTS

**Compose AI coding rules from templates. Generate `AGENTS.md`, `CLAUDE.md`, and Cursor rules from a single source.**

Stop maintaining multiple static instruction files. Write small, focused templates that compose into context-aware rules for any AI coding tool.
Useful for monorepos, defining personal agent instructions, and defining cloud agent rules vs local agent rules.

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
  env: ["production"]      # Environment-specific
  language: ["typescript"]  # Language-specific
  use_beads: "true"        # Tool availability
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

✅ **Templates** are version controlled (`.cAGENTS/templates/`)
✅ **Generated files** are rebuilt on every `npm install`
✅ **Context** is always fresh and environment-specific
✅ **No merge conflicts** on generated files

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

**Problem:** Static `AGENTS.md` files can't adapt to context.

Every developer and AI sees the same rules, regardless of:
- Environment (local vs cloud vs production)
- Tools available (linters, issue trackers)
- Project area (frontend vs backend vs infra)

**Solution:** Compose from templates. Generate context-aware output.

**Benefits:**
- **DRY** - Write once, use everywhere
- **Context-aware** - Different rules for different contexts
- **Multi-format** - AGENTS.md, CLAUDE.md, .cursorrules from one source
- **Monorepo-friendly** - Per-package rules without duplication

---

## License

MIT

---

**Built for the age of AI-assisted coding.**

Give your agents the context they need, when they need it.
