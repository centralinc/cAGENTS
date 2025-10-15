# cAGENTS

**Dynamic, context-aware AI coding rules from composable templates.**

```
⚠️ Alpha Software - Core features work, but expect rough edges and breaking changes.
```

## Why cAGENTS?

AGENTS.md is becoming the standard for AI coding context. But static files have limitations:

- **No context adaptation** - Every AI sees the same rules, whether you're in a local dev env or cloud coding agent.
- **No format flexibility** - Can't easily generate AGENTS.md, CLAUDE.md, and Cursor rules from the same source
- **No location awareness** - Monorepos need different rules per package, but maintaining multiple files is tedious

**cAGENTS solves this** by letting you write small, focused templates that compose into dynamic AGENTS.md files. Templates can target specific file patterns, environments, or output formats.

---

## Getting Started

### Installation

```bash
# Install as a dev dependency
npm install --save-dev cagents

# Or using pnpm
pnpm add -D cagents
```

### Initialize

```bash
pnpm cagents init
```

This creates:
- `.cAGENTS/config.toml` - Configuration file
- `.cAGENTS/templates/` - Directory for your templates
- `.cAGENTS/.gitignore` - Local config overrides

### Create Templates

Templates are Markdown files with YAML frontmatter:

```markdown
---
name: project-guidelines
alwaysApply: true
order: 1
---
# Project Guidelines

Use TypeScript for all new code.
Write tests alongside implementation.
```

Save templates in `.cAGENTS/templates/`.

### Build

```bash
cagents build
```

This generates `AGENTS.md` (and any other configured outputs) from your templates.

### Migrate Existing Rules

If you already have AGENTS.md, CLAUDE.md, .cursorrules, or Cursor rules:

```bash
cagents migrate
```

This converts your existing rules into .cAGENTS templates.

---

## Simple Implementation

The simplest way to use cAGENTS:

1. **Add AGENTS.md to .gitignore** so generated files aren't committed:
   ```bash
   cagents git ignore-outputs
   ```

2. **Hook up build to postinstall** so AGENTS.md is regenerated on every install:
   ```json
   {
     "scripts": {
       "postinstall": "cagents build"
     }
   }
   ```

Now your AGENTS.md is always fresh and generated from your templates. Team members get the right context automatically when they install dependencies, and cloud coding agents (like Codex Cloud) can generate environment-specific rules on the fly.

---

## Configuration

### Config File: `.cAGENTS/config.toml`

```toml
[paths]
templatesDir = "templates"    # Where templates live
outputRoot = "."              # Where to write AGENTS.md

[defaults]
engine = "builtin:simple"     # Template engine (builtin:simple or command:<cmd>)

[variables.static]
project = "myapp"             # Static variables for templates
owner = "Your Name"

[variables.command]
branch = "git rev-parse --abbrev-ref HEAD"   # Dynamic variables from shell commands

[execution]
shell = "bash"
timeoutMs = 3000

[output]
targets = ["agents-md"]       # Output formats: agents-md, claude-md, cursorrules
```

**Config Precedence** (later overrides earlier):
1. `~/.cagents/config.toml` - User defaults (optional)
2. `.cAGENTS/config.toml` - Project config (committed)
3. `.cAGENTS/config.local.toml` - Local overrides (gitignored)

### Template Frontmatter

```yaml
---
name: template-name              # Required: unique identifier
description: What this does      # Optional: human-readable description
engine: builtin:simple           # Optional: override config engine
globs:                           # Optional: generate nested AGENTS.md in matching dirs
  - "src/api/**"
  - "tests/**"
alwaysApply: true                # Optional: include in root AGENTS.md
when:                            # Optional: conditional inclusion
  env: ["codex-cloud"]           # Only include when --env codex-cloud
  language: ["rust"]             # Only include when --language rust
  target: ["agents-md"]          # Only include for specific output formats
order: 10                        # Optional: sort order (default: 50, lower = earlier)
targets: ["agents-md", "cursor"] # Optional: which outputs get this template
---
# Your template content here

Use variables: {{project}}, {{owner}}, {{branch}}
```

### Variables

**Static variables** (defined in config):
```toml
[variables.static]
project = "myapp"
api_url = "https://api.example.com"
```

**Command variables** (executed at build time):
```toml
[variables.command]
branch = "git rev-parse --abbrev-ref HEAD"
commit = "git rev-parse --short HEAD"
```

Use in templates: `{{project}}`, `{{branch}}`, etc.

### Template Engines

**Built-in engine** (`builtin:simple`):
- Simple `{{variable}}` substitution
- No conditionals or loops
- Fast and reliable

**External engines** (`command:<cmd>`):
```toml
[defaults]
engine = "command:python3 scripts/render.py"
```

Your command receives JSON on stdin:
```json
{
  "templateSource": "template content...",
  "data": {"project": "myapp", ...},
  "frontmatter": {...},
  "templatePath": ".cAGENTS/templates/example.md"
}
```

And returns JSON on stdout:
```json
{
  "content": "rendered content..."
}
```

This lets you use Jinja2, Handlebars, Liquid, or any template engine you prefer.

---

## Commands

### `cagents init`

Initialize .cAGENTS in your project.

```bash
cagents init              # Create basic setup
cagents init --force      # Overwrite existing setup
cagents init --dry-run    # Show what would be created
```

### `cagents build`

Generate AGENTS.md files from templates.

```bash
cagents build                                        # Build with all templates
cagents build --env codex-cloud                      # Filter by environment
cagents build --language rust                        # Filter by language
cagents build --env codex-cloud --language rust      # Combine filters
cagents build --dry-run                              # Preview without writing
```

**How it works:**
1. Loads config with precedence (user → project → local)
2. Discovers templates in `templatesDir`
3. Filters by `when` clauses and context flags
4. Renders using configured engine
5. Merges rendered content
6. Writes AGENTS.md files (nested based on globs)
7. Cleans up old AGENTS.md files that no longer match

### `cagents migrate`

Migrate from other formats to .cAGENTS.

```bash
cagents migrate              # Auto-detect and convert
cagents migrate --from .cursorrules
cagents migrate --backup     # Backup originals before converting
```

**Supported formats:**
- `.cursorrules` (Cursor legacy)
- `.cursor/rules/` (Cursor modern)
- `AGENTS.md`
- `CLAUDE.md`

### `cagents lint`

Validate configuration and templates.

```bash
cagents lint
```

Checks:
- Config file is valid TOML
- All templates have required frontmatter
- No glob conflicts
- No unreachable templates

### `cagents preview`

Preview build output before writing files.

```bash
cagents preview
```

Shows:
- Which files will be generated
- Which templates contribute to each file
- First 20 lines of rendered output
- Interactive navigation (if multiple files)

### `cagents render <file>`

Render AGENTS.md context for a specific file.

```bash
cagents render src/api/users.rs              # Render to stdout
cagents render src/api/users.rs > context.md # Save to file
cagents render src/api/users.rs --var team=platform --var feature=auth
```

Useful for:
- Testing which rules apply to a file
- Generating context for specific files
- Debugging template matching

### `cagents context <file>`

Show metadata about which rules apply to a file.

```bash
cagents context src/api/users.rs
cagents context src/api/users.rs --json         # JSON output
cagents context src/api/users.rs --var team=platform
```

Shows:
- Matched rules and why they matched
- Available variables and their values
- File information (path, extension, directory)

### `cagents status`

Show project stats and info.

```bash
cagents status
```

Displays:
- Configuration path
- Number of templates found
- Output directory
- Whether AGENTS.md exists
- List of all templates

### `cagents config`

View current configuration (deprecated - just edit the file directly).

```bash
cagents config              # View config
```

**Note:** It's easier to just edit `.cAGENTS/config.toml` directly or use `$EDITOR .cAGENTS/config.toml`.

### `cagents export`

Export to other formats (deprecated - use `build` with `[output]` config instead).

```bash
cagents export --target cursor
```

**Note:** Use the `[output]` section in your config instead:
```toml
[output]
targets = ["agents-md", "claude-md", "cursorrules"]
```

Then `cagents build` will generate all formats.

### `cagents git`

Manage .gitignore for AGENTS.md files.

```bash
cagents git ignore-outputs     # Add AGENTS.md to .gitignore
cagents git unignore-outputs   # Remove from .gitignore
```

### `cagents setup`

Setup package manager integration.

```bash
cagents setup npm        # Add cagents commands to package.json
cagents setup pnpm       # Works with pnpm too
```

---

## Tips

**Monorepos:** Use globs to generate AGENTS.md in each package:

```yaml
---
name: web-app-rules
globs: ["apps/web/**"]
---
# Web App Rules
...
```

This creates `apps/web/AGENTS.md` with just the rules that match `apps/web/**`.

**Cloud coding agent rules:**

Add extra context when running in Codex Cloud or other cloud coding environments:

```yaml
---
name: cloud-agent-context
when:
  env: ["codex-cloud"]
---
# Cloud Agent Context

This project uses a microservices architecture deployed on AWS.
API documentation: https://api.example.com/docs
Architecture diagrams: https://wiki.example.com/architecture
...
```

Then in your cloud agent's startup script:
```bash
export CODE_CLOUD=1
cagents build --env codex-cloud
```

Now cloud agents get additional context that might not be relevant for local development.

**Multiple output formats:**

```toml
[output]
targets = ["agents-md", "claude-md", "cursorrules"]
```

One `cagents build` generates all formats.

**Template organization:**

```
.cAGENTS/templates/
├── base-guidelines.md       # alwaysApply: true
├── rust-rules.md            # globs: ["**/*.rs"]
├── typescript-rules.md      # globs: ["**/*.ts"]
└── cloud-agent-context.md   # when: { env: ["codex-cloud"] }
```

---

## License

MIT - See [LICENSE](./LICENSE)

---

**Built for the age of AI-assisted coding.**

Give your agents the context they need, when they need it.
