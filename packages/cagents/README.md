# cagents

**Dynamic, context-aware AI coding rules from composable templates.**

⚠️ **Alpha** - Build from source only. npm package coming soon.

---

## What is cAGENTS?

A CLI that generates `AGENTS.md` from small, composable templates. Different context for different environments, languages, and code locations.

**The Problem:** AGENTS.md is static. Every AI sees the same context.

**The Solution:** `.cAGENTS` composes dynamic context from templates.

---

## Installation

### From Source (Current)

```bash
git clone https://github.com/centralinc/cAGENTS.git
cd cAGENTS
cargo build --release --workspace

# Binary at: target/release/cagents
```

### npm Package (Coming Soon)

```bash
# Not yet available
npm install -g cagents
```

---

## Quick Start

```bash
# Initialize
cd your-project
cagents init

# Creates .cAGENTS/ with config and templates

# Build
cagents build
```

---

## How It Works

Write templates with frontmatter:

```yaml
---
name: rust-guidelines
when:
  language: ["rust"]
globs: ["**/*.rs"]
---
## Rust Development

Use Result types for error handling.
```

Configure variables in `.cAGENTS/config.toml`:

```toml
[variables.static]
language = "rust"
```

Or use dynamic variables:

```toml
[variables.command]
app_env = "echo $APP_ENV"
```

---

## Commands

### ✅ Working

- `cagents init` - Initialize `.cAGENTS`
- `cagents build` - Generate AGENTS.md with filtering
- `cagents export --target cursor` - Export to Cursor
- `cagents import` - Import from other formats
- `cagents lint` - Validate configuration
- `cagents preview` - Preview build output
- `cagents render <file>` - Render context for file
- `cagents context <file>` - Show metadata for file
- `cagents config` - Manage config interactively
- `cagents status` - Show project stats
- `cagents git ignore-outputs` - Manage .gitignore
- `cagents setup pnpm` - Package manager integration

### 🔄 Planned

- `cagents watch` - Auto-rebuild (coming soon)

---

## Configuration

### Basic Config

```toml
# .cAGENTS/config.toml

[paths]
templatesDir = "templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"

[variables.static]
project = "myapp"
owner = "Jordan"

[variables.command]
branch = "git rev-parse --abbrev-ref HEAD"
```

### Template Frontmatter

```yaml
---
name: template-name
globs: ["src/api/**"]
when:
  env: ["dev", "staging"]
  language: ["Rust"]
order: 10
---
# Your content
```

---

## Features

### ✅ Working

- Context filtering (env/language via `when` clauses)
- Glob-based nested outputs
- Variables: static + command
- Built-in simple engine
- BYOC: external engines
- Config precedence
- Cursor export
- Import from multiple formats
- Old file cleanup

### 🔄 Planned

- Multiple init presets
- Environment variables
- Watch mode
- More export formats

---

## Use Cases

### 1. Environment-Specific

Define variables in config:

```toml
# .cAGENTS/config.toml
[variables.command]
app_env = "echo $APP_ENV"
```

Use in templates:

```yaml
---
when:
  app_env: ["dev"]
---
## Dev Environment
Local DB, debug shortcuts
```

### 2. Language-Specific

Configure language variable:

```toml
[variables.static]
language = "rust"
```

Use in templates:

```yaml
---
when:
  language: ["rust"]
globs: ["**/*.rs"]
---
## Rust Guidelines
Use Result types for error handling
```

### 3. Monorepo

Templates with globs generate nested AGENTS.md:

```yaml
---
globs: ["apps/web/**"]
---
```

Output:
```
AGENTS.md
apps/web/AGENTS.md
apps/api/AGENTS.md
```

---

## Template Engines

### Built-in Simple

```markdown
# {{project}}
Owner: {{owner}}
```

### External (BYOC)

```toml
[defaults]
engine = "command:python3 render.py"
```

Supports: Jinja2, Liquid, Tera, Handlebars, etc.

---

## Why cAGENTS?

AGENTS.md is becoming the standard for AI coding context, but it's static.

**.cAGENTS makes it dynamic** - different rules for:
- Different tools (Claude vs Cursor)
- Different environments (dev vs prod)
- Different languages (Rust vs TypeScript)
- Different locations (monorepo packages)

---

## Documentation

- [Main README](../../README.md) - Complete guide
- [CLAUDE.md](../../CLAUDE.md) - Development guide
- [docs/](../../docs/) - All documentation

---

## Links

- **GitHub**: [github.com/centralinc/cAGENTS](https://github.com/centralinc/cAGENTS)
- **Issues**: [github.com/centralinc/cAGENTS/issues](https://github.com/centralinc/cAGENTS/issues)

---

## License

MIT

---

**Built for AI-assisted coding.** 🤖
