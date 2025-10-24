# Advanced cAGENTS Usage

This guide covers advanced features and configurations for cAGENTS.

---

## Table of Contents

- [Custom Template Engines](#custom-template-engines)
- [Complex Glob Patterns](#complex-glob-patterns)
- [Output Strategies](#output-strategies)
- [Monorepo Configurations](#monorepo-configurations)
- [Template Variables](#template-variables)
- [Conditional Logic](#conditional-logic)
- [Config Precedence](#config-precedence)
- [Performance Optimization](#performance-optimization)

---

## Custom Template Engines

While the built-in `builtin:simple` engine handles basic variable substitution, you can use any template engine via external commands.

### BYOC (Bring Your Own Compiler) Protocol

External engines communicate via JSON over stdin/stdout:

**Input (stdin):**
```json
{
  "templateSource": "template content with {{vars}}...",
  "templatePath": ".cAGENTS/templates/example.md",
  "data": {
    "project": "myapp",
    "branch": "main"
  },
  "frontmatter": {
    "name": "example",
    "order": 10
  },
  "cwd": "/path/to/project"
}
```

**Output (stdout):**
```json
{
  "content": "rendered content...",
  "diagnostics": []  // Optional warnings/errors
}
```

### Example: Python Jinja2

**scripts/render.py:**
```python
#!/usr/bin/env python3
import sys
import json
from jinja2 import Template

input_data = json.load(sys.stdin)
template = Template(input_data['templateSource'])
rendered = template.render(**input_data['data'])

json.dump({"content": rendered}, sys.stdout)
```

**Config:**
```toml
[defaults]
engine = "command:python3 scripts/render.py"
```

### Example: Node Handlebars

**scripts/render.js:**
```javascript
#!/usr/bin/env node
const Handlebars = require('handlebars');

const input = JSON.parse(require('fs').readFileSync(0, 'utf-8'));
const template = Handlebars.compile(input.templateSource);
const rendered = template(input.data);

console.log(JSON.stringify({ content: rendered }));
```

**Config:**
```toml
[defaults]
engine = "command:node scripts/render.js"
```

---

## Complex Glob Patterns

### Glob Syntax

Globs use standard glob syntax:
- `**` - Match any number of directories
- `*` - Match any characters except `/`
- `?` - Match single character
- `[abc]` - Match character class
- `{a,b}` - Match alternatives

### Special Characters

If your paths contain glob special characters (like Next.js `[id]` routes), they're automatically escaped during migration.

### Multiple Globs

```yaml
---
globs:
  - "packages/*/src/**"
  - "apps/*/src/**"
---
```

### Directory vs File Globs

**File glob** (`packages/**/*.ts`):
- Matches files
- With `outputIn: "parent"` → outputs in file's parent directory

**Directory glob** (`packages/api/`):
- Matches directories (trailing slash)
- With `outputIn: "matched"` → outputs in matched directory

---

## Output Strategies

Control where AGENTS.md files are generated with `outputIn`:

### `parent` (default)

```yaml
---
globs: ["src/**/*.ts"]
outputIn: "parent"
---
```

For each matching file, output to its parent directory.

Example:
- Matches `src/api/users.ts`, `src/api/posts.ts`
- Creates `src/api/AGENTS.md`

### `matched`

```yaml
---
globs: ["packages/*/"]
outputIn: "matched"
---
```

Output directly in matched directories.

Example:
- Matches `packages/api/`, `packages/web/`
- Creates `packages/api/AGENTS.md`, `packages/web/AGENTS.md`

### `common-parent`

```yaml
---
globs: ["src/**/*.ts", "lib/**/*.ts"]
outputIn: "common-parent"
---
```

Find common parent of all matches, output there.

Example:
- Matches files in `src/` and `lib/`
- Common parent is `.` (root)
- Creates `./AGENTS.md`

---

## Monorepo Configurations

### Per-Package Rules

**Structure:**
```
.cAGENTS/templates/
├── base-rules.md          # Applies everywhere
├── api-package.md         # globs: ["packages/api/**"]
└── web-package.md         # globs: ["packages/web/**"]
```

**Result:**
```
packages/api/AGENTS.md     # base + api rules
packages/web/AGENTS.md     # base + web rules
AGENTS.md                  # base rules only
```

### Conditional Package Rules

```yaml
---
name: api-production-rules
globs: ["packages/api/**"]
when:
  env: ["production", "staging"]
---
```

Only applies to API package AND when env matches.

### Shared + Specific

```yaml
---
name: shared-testing
globs:
  - "packages/*/tests/**"
  - "apps/*/tests/**"
---
```

Applies to all test directories across packages and apps.

---

## Template Variables

### Static Variables

Defined in config, never change:

```toml
[variables.static]
project = "myapp"
team = "platform"
api_url = "https://api.example.com"
```

### Command Variables

Execute at build time:

```toml
[variables.command]
branch = "git rev-parse --abbrev-ref HEAD"
commit = "git rev-parse --short HEAD"
has_docker = "command -v docker >/dev/null && echo true || echo false"
```

### Environment Variables

```toml
[variables.env]
app_env = "APP_ENV"        # Read from $APP_ENV
region = "AWS_REGION"
```

### Variable Substitution

In templates:
```markdown
Project: {{project}}
Branch: {{branch}}
API: {{api_url}}
```

**Note:** The `builtin:simple` engine only does basic substitution. For conditionals, loops, or filters, use an external engine.

---

## Conditional Logic

### `when` Clause Matching

All conditions must match (AND logic):

```yaml
---
when:
  env: ["production", "staging"]  # env must be prod OR staging
  language: ["rust"]              # AND language must be rust
  has_docker: "true"              # AND has_docker must be "true"
---
```

### Target Filtering

```yaml
---
when:
  target: ["claude-md"]  # Only in CLAUDE.md output
---
```

### No `when` Clause

Templates without `when` apply everywhere:

```yaml
---
name: base-guidelines
---
```

This is the recommended approach for universal rules.

---

## Config Precedence

Configs merge with later values overriding earlier:

1. **User config** (`~/.cagents/config.toml`) - Personal defaults
2. **Project config** (`.cAGENTS/config.toml`) - Team settings
3. **Local overrides** (`.cAGENTS/config.local.toml`) - Developer-specific

Example:

**~/.cagents/config.toml:**
```toml
[variables.static]
author = "Your Name"
editor = "vim"
```

**.cAGENTS/config.toml:**
```toml
[variables.static]
project = "myapp"
author = "Team Name"  # Overrides user config
```

**.cAGENTS/config.local.toml:**
```toml
[variables.static]
author = "Local Dev"  # Overrides both
```

Result: `author = "Local Dev"`

---

## Performance Optimization

### Reduce Command Execution

Command variables execute on every build. Cache expensive operations:

**Slow:**
```toml
[variables.command]
git_stats = "git log --all --oneline | wc -l"  # Runs every build
```

**Fast:**
```toml
[variables.static]
git_stats = "1000"  # Update manually when needed
```

### Timeout Configuration

```toml
[execution]
shell = "bash"
timeoutMs = 3000      # Kill commands after 3 seconds
allowCommands = true  # Disable to prevent command execution
```

### Template Organization

Fewer templates = faster builds. Combine related rules:

**Instead of:**
```
typescript-imports.md
typescript-types.md
typescript-testing.md
```

**Use:**
```
typescript-all.md  # All TypeScript rules in one file
```

---

## Advanced Glob Patterns

### Exclude Patterns

Use `.gitignore` to exclude directories from glob matching:

```gitignore
node_modules/
target/
dist/
```

Or use `.cagentsignore`:

```
# .cagentsignore
*.test.ts
__mocks__/
```

### Glob Simplification

cAGENTS automatically simplifies overlapping globs:

**Before:**
```yaml
globs:
  - "src/**/*.ts"
  - "src/api/**/*.ts"  # Redundant
```

**After simplification:**
```yaml
globs:
  - "src/**/*.ts"
```

### Cross-Package Patterns

```yaml
---
globs:
  - "packages/*/src/api/**"
  - "apps/*/api/**"
---
```

Matches API code across all packages and apps.

---

## Template Frontmatter Reference

Complete frontmatter options:

```yaml
---
# Required
name: template-name

# Optional
description: Human-readable description
order: 10                    # Sort order (default: 50, lower = earlier)
engine: builtin:simple       # Override config engine
extends: base-template       # Inherit from another template

# Filtering
globs:
  - "src/**/*.ts"
  - "tests/**"
when:
  env: ["production"]
  language: ["rust"]
  custom_var: "value"
  target: ["agents-md"]

# Output control
outputIn: matched           # Where to output: matched, parent, common-parent
targets: ["agents-md"]      # Which formats get this (legacy, use when.target instead)

# Merging
merge:
  sections: append          # How to merge: append, replace, prepend
---
```

---

## Multiple Environments

### Environment-Specific Rules

```yaml
# .cAGENTS/templates/production.md
---
name: production-rules
when:
  env: ["production"]
---
# Production Guidelines

- All changes require review
- Feature flags required for new features
- Monitor error rates closely
```

```yaml
# .cAGENTS/templates/development.md
---
name: development-rules
when:
  env: ["development"]
---
# Development Guidelines

- Experiment freely
- Use feature branches
- Run tests locally before pushing
```

Build with environment:

```bash
APP_ENV=production cagents build   # Includes production rules
APP_ENV=development cagents build  # Includes development rules
```

**Config:**
```toml
[variables.env]
env = "APP_ENV"
```

---

## Debugging

### Preview Before Building

```bash
cagents preview
```

Shows what will be generated without writing files.

### Render Specific File

```bash
cagents render src/api/users.ts
```

Shows which rules apply to a specific file.

### Context Metadata

```bash
cagents context src/api/users.ts --json
```

JSON output showing:
- Matched rules
- Variable values
- File metadata

### Lint Configuration

```bash
cagents lint
```

Validates:
- Config TOML syntax
- Template frontmatter
- Glob patterns
- Target names
- Variable references

---

## Migration Details

### Nested File Migration

cAGENTS automatically discovers nested AGENTS.md/CLAUDE.md files:

```
AGENTS.md
docs/AGENTS.md
packages/api/AGENTS.md
```

After migration:
```
.cAGENTS/templates/agents-root.md     # From root AGENTS.md
.cAGENTS/templates/agents-docs.md     # globs: ["docs/"], outputIn: "matched"
.cAGENTS/templates/agents-api.md      # globs: ["packages/api/"], outputIn: "matched"
```

Running `cagents build` recreates the exact same structure.

### Multi-Format Migration

```bash
cagents migrate
```

If you have both AGENTS.md and CLAUDE.md, migration creates:
- Separate templates for each format
- Target filtering (`when.target: ["agents-md"]` or `["claude-md"]`)
- Preserves nested file structure

### Cursor Rules Migration

**.cursor/rules/typescript.md** → `.cAGENTS/templates/cursor-typescript.md`

Preserves directory structure and file names.

---

## Troubleshooting

### Build Not Updating

1. Check config exists: `ls .cAGENTS/config.toml`
2. Validate config: `cagents lint`
3. Preview output: `cagents preview`
4. Check template frontmatter syntax

### Templates Not Matching

Use `cagents render <file>` to see which rules apply to a specific file.

### Command Variables Not Working

Check execution config:

```toml
[execution]
shell = "bash"              # Must be available on PATH
timeoutMs = 3000           # Increase if commands are slow
```

Verify command works in shell:
```bash
bash -c "your command here"
```

### Windows Path Issues

Globs always use forward slashes, even on Windows:

```yaml
# Correct
globs: ["src/api/**"]

# Incorrect (don't do this)
globs: ["src\\api\\**"]
```

---

## Best Practices

### Template Organization

```
.cAGENTS/templates/
├── 00-base.md              # Order: 1-10, no globs, universal rules
├── 10-typescript.md        # Order: 10-20, language-specific
├── 20-api-layer.md         # Order: 20-30, globs: ["src/api/**"]
└── 90-environment.md       # Order: 90+, when: {env: ["production"]}
```

Use numbering prefix for easy sorting.

### Keep Templates Focused

**Bad:**
```markdown
---
name: everything
---
# TypeScript Rules
...
# API Rules
...
# Testing Rules
...
# Database Rules
...
```

**Good:**
```markdown
# Three separate files
typescript.md
api-layer.md
testing.md
```

Easier to maintain and reuse across projects.

### Use `when` Sparingly

Only add `when` clauses when you truly need conditional inclusion. Most rules should apply everywhere:

```yaml
---
name: base-guidelines
# No when clause - applies everywhere
---
```

### Commit Templates, Ignore Outputs

```bash
git add .cAGENTS/
git commit -m "Add cAGENTS templates"

# Add to .gitignore:
AGENTS.md
CLAUDE.md
.cursorrules
**/AGENTS.md
**/CLAUDE.md
```

Or use: `cagents git ignore-outputs`

---

## Example Configurations

### Monorepo with Per-Package Rules

```toml
[paths]
templatesDir = "templates"
outputRoot = "."

[output]
targets = ["agents-md", "claude-md"]

[defaults]
engine = "builtin:simple"
```

**Templates:**
```
.cAGENTS/templates/
├── base.md                    # No globs, applies to root only
├── package-api.md            # globs: ["packages/api/"]
├── package-web.md            # globs: ["packages/web/"]
└── testing.md                # globs: ["**/tests/**"]
```

### Environment-Specific Rules

```toml
[variables.env]
app_env = "APP_ENV"
feature_flags = "FEATURE_FLAGS"

[variables.command]
has_docker = "command -v docker >/dev/null && echo true || echo false"
```

**Template:**
```yaml
---
when:
  app_env: ["production"]
  has_docker: "true"
---
# Production Docker Rules

Use multi-stage builds.
Always specify resource limits.
```

### Language-Specific Rules

```yaml
---
name: rust-rules
when:
  language: ["rust"]
---
# Rust Guidelines

Run `cargo clippy` before committing.
Use `#[cfg(test)]` for test modules.
```

Set language via CLI:
```bash
cagents build --language rust
```

Or via config:
```toml
[variables.static]
language = "rust"
```

---

## Architecture

### Build Pipeline

1. **Config Loading**: Merge user → project → local configs
2. **Template Discovery**: Find all `.md` files in `templatesDir`
3. **Frontmatter Parsing**: Extract YAML metadata
4. **Context Building**: Collect variables (static, command, env)
5. **Rule Filtering**: Apply `when` clauses and glob patterns
6. **Rendering**: Execute template engine for each rule
7. **Merging**: Combine rendered rules by section
8. **Writing**: Output to target directories
9. **Cleanup**: Remove old AGENTS.md files no longer needed

### Template Resolution

For file `src/api/users.ts`:

1. Load all templates
2. Filter by `when` clause (env, language, etc.)
3. Filter by `globs` (must match file path)
4. Sort by `order`
5. Render with template engine
6. Merge into final output

### Output Directory Calculation

With `globs` and `outputIn`:

1. Find all files matching glob patterns
2. Apply `outputIn` strategy:
   - `parent`: Use each file's parent directory
   - `matched`: Use matched directory (for directory globs)
   - `common-parent`: Find common ancestor
3. Generate one AGENTS.md per unique directory

---

## Extending cAGENTS

### Custom Init Presets

Create preset in `~/.cagents/presets/mypreset/`:

```
~/.cagents/presets/mypreset/
├── config.toml
└── templates/
    └── example.md
```

Use with:
```bash
cagents init --preset mypreset
```

### Programmatic API

```javascript
const { build, migrate } = require('cagents');

// Build programmatically
await build({ dryRun: false, outDir: null });

// Migrate programmatically
await migrate({ from: null, backup: true });
```

---

## Platform Support

### Cross-Platform Paths

cAGENTS normalizes paths across platforms:
- Globs always use forward slashes
- Windows backslashes converted automatically
- Path separators handled correctly

### Platform-Specific Commands

```toml
[variables.command]
# Works on Unix
has_docker = "command -v docker >/dev/null && echo true || echo false"

# Works on Windows
has_docker = "where docker >nul 2>&1 && echo true || echo false"
```

Or use portable scripts:
```toml
[variables.command]
has_docker = "node scripts/check-docker.js"
```

---

## Migration from Other Tools

### From agentstack

agentstack uses a single AGENTS.md. To migrate:

```bash
cagents migrate
```

Then split into focused templates manually.

### From Cursor

Cursor uses `.cursor/rules/`. Migration preserves structure:

```bash
cagents migrate
```

**.cursor/rules/typescript.md** → `.cAGENTS/templates/cursor-typescript.md`

### Manual Migration

1. Create `.cAGENTS/` structure
2. Copy rules into templates
3. Add frontmatter
4. Test with `cagents preview`
5. Build with `cagents build`

---

## Roadmap

- [ ] Web UI for managing templates
- [ ] Template marketplace/registry
- [ ] Git hooks for auto-build
- [ ] IDE extensions
- [ ] Template validation with schemas

See [GitHub Issues](https://github.com/centralinc/cAGENTS/issues) for current roadmap.

---

## FAQ

**Q: Can I use cAGENTS without npm?**

A: Yes! Install the binary directly from GitHub releases or build from source with Cargo.

**Q: Does this work with [insert AI tool]?**

A: cAGENTS generates standard AGENTS.md files that work with any tool supporting the format. It also generates CLAUDE.md and .cursorrules for tool-specific formats.

**Q: Can I keep my existing AGENTS.md?**

A: Yes! Use `cagents migrate` to convert it into templates, or ignore the .cAGENTS folder and keep using static files.

**Q: How do I share templates across projects?**

A: Create templates in `~/.cagents/presets/` and use `cagents init --preset <name>`.

**Q: Does this work in CI/CD?**

A: Yes! Set `CAGENTS_TEST=1` environment variable to disable interactive prompts.

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development setup and guidelines.

---

## Support

- GitHub Issues: https://github.com/centralinc/cAGENTS/issues
- Discussions: https://github.com/centralinc/cAGENTS/discussions
