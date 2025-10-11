# .cAGENTS

**Dynamic, context-aware AI coding rules from composable templates.**

⚠️ **Alpha Software** - Core features work, but expect rough edges and breaking changes.

---

## The Problem

AGENTS.md is becoming the standard for AI coding context. But it's static—every AI sees the same rules all the time.

What if you need:
- **Different context for different environments?** Production agents shouldn't see dev shortcuts
- **Different formats for different tools?** Team uses Claude AND Cursor
- **Location-based context?** Each package in monorepo needs specific rules

Traditional AGENTS.md: one massive file, everything for everyone.

**.cAGENTS: compose dynamic context from small templates.**

---

## Quick Start

### Installation

```bash
# Install as a dev dependency (Node.js project)
npm install --save-dev cagents

# Or using pnpm
pnpm add -D cagents
```

### Initialize

```bash
cd your-project
npm cagents init

# Creates:
# .cAGENTS/
#   ├── config.toml
#   ├── .gitignore
#   └── templates/
#       └── agents-root.md

## Core Concept

**Templates + Context → Dynamic Rules**

Instead of one giant AGENTS.md, you write small focused templates:

```
.cAGENTS/templates/
├── project-context.md      # Always applies
├── project-preferences.local.md       # Only for the local user, not checked into codebase
├── rust-guidelines.md      # Only for Rust files
└── typescript-patterns.md  # Only for TypeScript files
└── production.md           # Only when --env prod
```

Templates have **frontmatter** that controls when they apply:

```yaml
---
name: rust-guidelines
globs: ["**/*.rs"]
order: 10
---
## Rust Development

Use Result types for error handling.
Write tests in same file with #[cfg(test)].
```

Build with context:
```bash
npm cagents build

---

## Real Examples


### Example: Monorepo with Nested Context

**Problem:** Each package needs specific context.

**Solution:**

`templates/web-app.md`:
```yaml
---
name: web-app
globs: ["apps/web/**"]
---
## Web App (Next.js)

This is the main web application.
Pages in `app/`, API routes in `app/api/`
```

`templates/api-server.md`:
```yaml
---
name: api-server
globs: ["apps/api/**"]
---
## API Server (Node.js)

RESTful API server.
Routes in `src/routes/`, middleware in `src/middleware/`
```

**Output:**
```bash
cagents build

Generates:
├── AGENTS.md              (root context)
├── apps/web/AGENTS.md     (root + web-app rules)
└── apps/api/AGENTS.md     (root + api-server rules)
```

Each location gets relevant context automatically.

---

## Commands

### ✅ `cagents init`

Initialize `.cAGENTS` in your project.

```bash
cagents init

# Creates basic setup:
# - .cAGENTS/config.toml
# - .cAGENTS/templates/agents-root.md
# - .cAGENTS/.gitignore

# Options:
--force    # Overwrite existing setup
--backup   # Backup AGENTS.md before migration
--dry-run  # Show what would be created
```

**If AGENTS.md exists:** Automatically migrates to `.cAGENTS` format.

### ✅ `cagents build`

Generate AGENTS.md files.

```bash
cagents build

# With context filtering:
cagents build --env prod
cagents build --language rust
cagents build --env prod --language rust

# Options:
--env      # Filter by environment (dev, staging, prod, etc.)
--language # Filter by language (rust, typescript, python, etc.)
--out      # Custom output directory (coming soon)
--dry-run  # Show plan without writing files
```

**How it works:**
1. Loads config from `.cAGENTS/config.toml`
2. Discovers templates in `templates/`
3. Filters by context (`when` clauses)
4. Renders using builtin engine or external compiler
5. Merges rendered content
6. Writes AGENTS.md files (nested based on globs)

### ✅ `cagents export`

Export to other formats.

```bash
cagents export --target cursor

# Creates .cursor/rules/*.md files
```

**Supported targets:**
- ✅ `cursor` - Cursor IDE rules format
- 🔄 `claude-code` - Coming soon

### ✅ `cagents import`

Import from other formats to `.cAGENTS`.

```bash
cagents import              # Auto-detect format
cagents import --from .cursorrules

# Supported formats:
# - .cursorrules (Cursor legacy)
# - .cursor/rules/ (Cursor modern)
# - AGENTS.md
# - CLAUDE.md
```

### ✅ `cagents lint`

Validate configuration and templates.

```bash
cagents lint

🔍 Linting .cAGENTS configuration...

✅ Config valid
✅ All templates have required fields
✅ No glob conflicts
⚠️  Warning: Template 'old-rules.md' unreachable
```

### ✅ `cagents preview`

Preview build output before writing files.

```bash
cagents preview

# Shows:
# - Which files will be generated
# - Which templates contribute to each file
# - First 20 lines of rendered output
# - Interactive navigation (if multiple files)
```

### ✅ `cagents render <file>`

Render AGENTS.md context for a specific file.

```bash
cagents render src/api/users.rs

# Output to stdout (pipe to file or view directly)
cagents render src/api/users.rs > API_CONTEXT.md

# With custom variables:
cagents render src/api/users.rs --var team=platform --var env=prod
```

### ✅ `cagents context <file>`

Show metadata about which rules apply to a file.

```bash
cagents context src/api/users.rs

# Context for src/api/users.rs

## Matched Rules (3)
- **project-context** - alwaysApply: true (order: 1)
- **api-guidelines** - glob: src/api/** (order: 10)
- **rust-guidelines** - glob: **/*.rs (order: 20)

## Available Variables (5)
- `project` = "myapp" (config.static)
- `owner` = "Jordan" (config.static)
- `branch` = "main" (config.command)
...

# Options:
--json  # Output in JSON format
--var   # Override variables (KEY=VALUE)
```

### ✅ `cagents config`

Manage configuration interactively.

```bash
cagents config

Current Configuration:
  ▸ Static Variables:
     project = myapp
     owner = Jordan

  ▸ Paths:
     templatesDir = templates
     outputRoot = .

? What would you like to do?
  > View full config
    Edit in $EDITOR
    Exit
```

### ✅ `cagents status`

Show project stats and info.

```bash
cagents status

▸ cAGENTS Status

┌───────────────┬─────────────────────────┐
│ Configuration │ .cAGENTS/config.toml    │
│ Templates     │ 5 found                 │
│ Output        │ .                       │
│ AGENTS.md     │ Generated               │
└───────────────┴─────────────────────────┘

▸ Templates:
   • project-context
   • rust-guidelines
   • typescript-patterns
   • production-checklist
   • api-guidelines
```

### ✅ `cagents git`

Manage .gitignore for AGENTS.md files.

```bash
cagents git ignore-outputs     # Add AGENTS.md to .gitignore
cagents git unignore-outputs   # Remove from .gitignore
```

### ✅ `cagents setup <manager>`

Setup package manager integration.

```bash
cagents setup pnpm  # Add cagents commands to package.json
cagents setup npm   # Works with npm too
```

---

## Configuration

### Directory Structure

```
your-project/
├── .cAGENTS/
│   ├── config.toml              # Project config
│   ├── config.local.toml        # Local overrides (gitignored)
│   ├── .gitignore
│   └── templates/
│       ├── project-context.md
│       ├── rust-guidelines.md
│       └── typescript-patterns.md
├── AGENTS.md                    # Generated
└── src/
    └── api/
        └── AGENTS.md            # Generated (if globs match)
```

### Config File: `.cAGENTS/config.toml`

```toml
[paths]
templatesDir = "templates"
outputRoot = "."
cursorRulesDir = ".cursor/rules"  # For cursor export

[defaults]
engine = "builtin:simple"         # Template engine
# OR use external: engine = "command:python render.py"

[variables.static]
project = "myapp"
owner = "Jordan"
team = "Platform Engineering"

[variables.command]
branch = "git rev-parse --abbrev-ref HEAD"
commit = "git rev-parse --short HEAD"

[execution]
shell = "bash"
timeoutMs = 5000
```

**Config Precedence** (later overrides earlier):
1. `~/.cagents/config.toml` - User defaults
2. `.cAGENTS/config.toml` - Project config (committed)
3. `.cAGENTS/config.local.toml` - Local overrides (gitignored)

### Template Frontmatter

```yaml
---
name: template-name              # Required: unique identifier
description: What this does      # Optional
engine: builtin:simple           # Optional: override engine
globs:                           # Optional: file patterns for nested output
  - "src/api/**"
  - "tests/**"
alwaysApply: true                # Optional: always include in root
when:                            # Optional: context filters
  agentEnv: ["codex-cloud"]
order: 10                        # Optional: sort order (lower = earlier)
targets: ["agentsmd", "cursor"]  # Optional: output formats
---
# Your template content here

Use variables: {{project}}, {{owner}}, {{branch}}
```

**Key Fields:**
- `name` - Unique identifier (required)
- `globs` - Generate nested AGENTS.md in matching directories
- `alwaysApply` - Include in root output even with globs
- `when` - Only apply when context matches
- `order` - Sort order (default: 50, lower = earlier)
- `targets` - Which formats get this template (default: all)

### Variables

#### ✅ Static Variables

```toml
[variables.static]
project = "myapp"
owner = "Jordan"
api_url = "https://api.example.com"
```

Use in templates: `{{project}}`, `{{owner}}`, `{{api_url}}`

#### ✅ Command Variables (Dynamic)

```toml
[variables.command]
branch = "git rev-parse --abbrev-ref HEAD"
commit = "git rev-parse --short HEAD"
author = "git config user.name"
```

Executed at build time, output injected into templates.


## Template Engines

### Built-in Simple Engine

Default engine with `{{variable}}` syntax:

```markdown
# {{project}}

Owner: {{owner}}
Current branch: {{branch}}
```

**Supported:**
- ✅ Variable interpolation: `{{var}}`
- ✅ All variable types (static, command)
- ❌ Conditionals (use external engine)
- ❌ Loops (use external engine)

### BYOC: Bring Your Own Compiler

Use any template engine via external command:

```toml
[defaults]
engine = "command:python3 scripts/render-jinja.py"
```

**Protocol:**
- **Input (stdin)**: JSON with `templateSource`, `data`, `frontmatter`, `templatePath`
- **Output (stdout)**: JSON with `content` field

Example Python/Jinja2 renderer:

```python
#!/usr/bin/env python3
import sys
import json
from jinja2 import Template

input_data = json.load(sys.stdin)
template = Template(input_data['templateSource'])
rendered = template.render(input_data['data'])

output = {'content': rendered}
json.dump(output, sys.stdout)
```

**Supported external engines:**
- Jinja2 (Python)
- Liquid (Ruby)
- Tera (Rust)
- Handlebars (Node)
- Any language that reads JSON stdin and writes JSON stdout

---

## Feature Status

### ✅ Working Now

Core features tested and working:

- ✅ **Init** - Basic project initialization
- ✅ **Build** - Generate AGENTS.md with context filtering
- ✅ **Export** - Cursor format only
- ✅ **Import** - From .cursorrules, .cursor/rules/, AGENTS.md, CLAUDE.md
- ✅ **Lint** - Configuration validation
- ✅ **Preview** - See build output before writing
- ✅ **Render** - Generate context for specific file
- ✅ **Context** - Show rule metadata for file
- ✅ **Config** - Interactive configuration management
- ✅ **Status** - Project statistics
- ✅ **Variables** - Static + command variables
- ✅ **Context Filtering** - when clauses (env/role/language)
- ✅ **Glob-based Nested Outputs** - Multiple AGENTS.md files
- ✅ **Built-in Simple Engine** - {{variable}} syntax
- ✅ **BYOC** - External template engines
- ✅ **Config Precedence** - user < project < local
- ✅ **Git Integration** - Manage .gitignore
- ✅ **Old File Cleanup** - Remove outdated AGENTS.md files

---

## Why .cAGENTS?

### The Evolution of AI Context

1. **Early Days** - Each tool had its own format (`.cursorrules`, etc.)
2. **Cursor Innovation** - Auto-append context based on file location
3. **AGENTS.md Emerges** - Universal format across tools
4. **The Gap** - AGENTS.md is static, can't adapt to context

### What We Learned

- ✅ Universal formats are good (AGENTS.md standard)
- ✅ Context-aware rules are powerful (Cursor's approach)
- ❌ Tool-specific formats fragment ecosystem
- ❌ Static files don't scale for complex projects

### The .cAGENTS Approach

**Combine the best of both:**
- Generate universal AGENTS.md format
- Compose dynamically based on context
- Support multiple output formats
- Stay flexible with BYOC

**Design Principles:**
1. **Simplicity First** - Easy things should be easy
2. **Power When Needed** - Complex cases should be possible
3. **Standards-Based** - Generate standard formats, don't invent new ones
4. **Developer Experience** - Beautiful CLI, clear errors, fast builds

---

## Architecture

**Pipeline:** config → loader → planner → render → merge → writers

```
Config Loading          Discover all templates in templatesDir
      ↓                             ↓
Parse frontmatter      Filter by context (when clauses)
      ↓                             ↓
Execute engine         Plan output locations (globs)
      ↓                             ↓
Merge sections         Write AGENTS.md files
```

**Modules:**
- `config` - TOML parsing, precedence merging
- `loader` - Template discovery, frontmatter parsing
- `planner` - Context filtering, output planning
- `render` - Template engine execution (builtin + BYOC)
- `merge` - Section combining, deduplication
- `writers` - Format-specific output (AGENTS.md, Cursor .mdc)
- `init` - Project scaffolding
- `import` - Format conversion
- `lint` - Validation
- `interactive` - CLI prompts and menus

See [CLAUDE.md](./CLAUDE.md) for development details.

---

## License

MIT - See [LICENSE](./LICENSE)

---

**Built for the age of AI-assisted coding.**

Give your agents the context they need, when they need it.
