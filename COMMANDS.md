# cAGENTS Command Reference

Complete reference for all cAGENTS commands and options.

---

## Table of Contents

- [init](#init)
- [build](#build)
- [migrate](#migrate)
- [lint](#lint)
- [preview](#preview)
- [render](#render)
- [context](#context)
- [status](#status)
- [git](#git)
- [setup](#setup)

---

## `init`

Initialize cAGENTS in your project. Creates `.cAGENTS/` structure with config and example templates.

### Usage

```bash
cagents init [OPTIONS]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--preset <PRESET>` | Preset to use for initialization | `basic` |
| `--force` | Overwrite existing .cAGENTS directory | `false` |
| `--dry-run` | Show what would be created without writing | `false` |
| `--backup` | Backup existing .cAGENTS before overwriting | `false` |

### Examples

```bash
# Basic initialization
cagents init

# Use specific preset
cagents init --preset basic

# Force overwrite with backup
cagents init --force --backup

# Preview what will be created
cagents init --dry-run
```

### Behavior

1. Checks for existing AGENTS.md/CLAUDE.md/.cursorrules
2. If found, prompts to migrate (in interactive mode)
3. Creates `.cAGENTS/` directory structure
4. Writes `config.toml` with defaults
5. Creates example templates in `templates/`
6. Creates `.gitignore` for local config

### Files Created

```
.cAGENTS/
├── config.toml          # Project configuration
├── templates/           # Template directory
│   └── example.md       # Example template
└── .gitignore           # Ignore local configs
```

---

## `build`

Generate AGENTS.md files (and other configured outputs) from templates.

### Usage

```bash
cagents build [OPTIONS]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--out <DIR>` | Output directory (overrides config) | From config |
| `--dry-run` | Preview output without writing files | `false` |

### Examples

```bash
# Standard build
cagents build

# Build to custom directory
cagents build --out ./dist

# Preview without writing
cagents build --dry-run
```

### Build Process

1. Load config with precedence (user → project → local)
2. Execute command variables
3. Discover templates in `templatesDir`
4. Parse frontmatter and body
5. Filter by `when` clauses
6. Determine output directories from `globs` and `outputIn`
7. Render each template with engine
8. Merge rendered content
9. Write to output paths
10. Clean up old outputs no longer referenced

### Output Targets

Controlled by `[output]` section in config:

```toml
[output]
targets = ["agents-md", "claude-md", "cursorrules"]
```

Available targets:
- `agents-md` - AGENTS.md files
- `claude-md` - CLAUDE.md files
- `cursorrules` - .cursorrules files
- (More targets may be added)

### Environment Variables

**Read from environment:**
```toml
[variables.env]
app_env = "APP_ENV"
```

```bash
APP_ENV=production cagents build
```

**Disable interactivity in CI:**
```bash
CAGENTS_TEST=1 cagents build
```

---

## `migrate`

Migrate from other rule formats to cAGENTS templates.

### Usage

```bash
cagents migrate [OPTIONS]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--from <PATH>` | Specific file/directory to migrate | Auto-detect |
| `--backup` | Backup originals before deleting | `false` |

### Examples

```bash
# Auto-detect and migrate
cagents migrate

# Migrate specific format
cagents migrate --from AGENTS.md
cagents migrate --from .cursorrules
cagents migrate --from .cursor/rules

# Backup before migration
cagents migrate --backup
```

### Supported Formats

| Format | Path | Result |
|--------|------|--------|
| Cursor Legacy | `.cursorrules` | `.cAGENTS/templates/agents-cursor.md` |
| Cursor Modern | `.cursor/rules/` | One template per `.md` file |
| AGENTS.md | `AGENTS.md` | `.cAGENTS/templates/agents-root.md` |
| CLAUDE.md | `CLAUDE.md` | `.cAGENTS/templates/claude-root.md` |

### Nested File Support

If you have:
```
AGENTS.md
docs/AGENTS.md
packages/api/AGENTS.md
```

Migration creates:
```
.cAGENTS/templates/agents-root.md    # No globs
.cAGENTS/templates/agents-docs.md    # globs: ["docs/"]
.cAGENTS/templates/agents-api.md     # globs: ["packages/api/"]
```

Each template gets appropriate `globs` and `outputIn` to recreate the exact structure.

### Multi-Format Migration

If multiple formats exist (e.g., both AGENTS.md and CLAUDE.md):

**Interactive mode:**
- Prompts to select which format to migrate
- Option to "Merge all formats" creates separate templates for each

**Non-interactive:**
```bash
cagents migrate --from AGENTS.md    # Migrate just AGENTS.md
```

### Behavior

1. Detect available formats
2. For each detected file:
   - Read content
   - Create template with appropriate frontmatter
   - Add `globs` and `outputIn` for nested files
   - Write to `.cAGENTS/templates/`
3. Delete original files (after successful import)
4. Run initial `cagents build` to verify

---

## `lint`

Validate configuration and templates.

### Usage

```bash
cagents lint
```

### Checks

**Configuration:**
- Valid TOML syntax
- Required fields present
- Paths exist
- Engine format valid

**Templates:**
- Valid YAML frontmatter
- Required `name` field
- Valid target names
- Glob pattern syntax
- No conflicting globs

**Variables:**
- All referenced variables defined
- Command variables executable
- No circular dependencies

### Output

```
✓ Configuration valid
✓ Found 5 templates
✓ All templates valid
✓ No glob conflicts

Issues:
  ⚠ template 'example' references undefined variable 'foo'
  ⚠ glob pattern 'src/[invalid' has syntax error
```

---

## `preview`

Preview build output without writing files.

### Usage

```bash
cagents preview [PATH]
```

### Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `PATH` | File or directory to preview | `.` (root) |

### Examples

```bash
# Preview all outputs
cagents preview

# Preview specific file
cagents preview src/api/users.ts

# Preview directory
cagents preview packages/api/
```

### Output

Shows for each output file:
- Output path
- Contributing templates
- First 20 lines of rendered content
- Total line count

**Interactive mode:**
- Navigate between files with arrow keys
- Press Enter to see full content
- Press Q to quit

---

## `render`

Render AGENTS.md context for a specific file to stdout.

### Usage

```bash
cagents render <FILE> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `FILE` | Path to file to render context for | Yes |

### Options

| Option | Description | Example |
|--------|-------------|---------|
| `--var <KEY=VALUE>` | Override variables (repeatable) | `--var team=platform` |

### Examples

```bash
# Render to stdout
cagents render src/api/users.rs

# Save to file
cagents render src/api/users.rs > context.md

# Override variables
cagents render src/api/users.rs --var env=production --var team=api

# Chain with other tools
cagents render src/api/users.rs | pbcopy    # Copy to clipboard (macOS)
```

### Use Cases

- Test which rules apply to a file
- Generate file-specific context for AI
- Debug template matching
- Integrate with editors/IDEs

---

## `context`

Show metadata about rules, variables, and context for a file.

### Usage

```bash
cagents context <FILE> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `FILE` | Path to file to analyze | Yes |

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--json` | Output JSON instead of Markdown | `false` |
| `--var <KEY=VALUE>` | Override variables (repeatable) | None |

### Examples

```bash
# Show context as Markdown
cagents context src/api/users.ts

# JSON output for scripts
cagents context src/api/users.ts --json

# Override variables
cagents context src/api/users.ts --var env=production
```

### Output Format

**Markdown:**
```markdown
# Context for src/api/users.ts

## Matched Rules
- base-guidelines (order: 1)
- rust-rules (order: 10, matched: globs)
- api-layer (order: 20, matched: globs)

## Variables
- project: myapp
- branch: main
- env: development

## File Info
- Path: src/api/users.ts
- Extension: .ts
- Directory: src/api
```

**JSON:**
```json
{
  "file": "src/api/users.ts",
  "matched_rules": [
    {"name": "base-guidelines", "order": 1, "reason": "no filters"},
    {"name": "rust-rules", "order": 10, "reason": "glob match"}
  ],
  "variables": {
    "project": "myapp",
    "branch": "main"
  },
  "file_info": {
    "path": "src/api/users.ts",
    "extension": "ts",
    "directory": "src/api"
  }
}
```

---

## `status`

Show project status and statistics.

### Usage

```bash
cagents status
```

### Output

```
cAGENTS Status

Configuration:
  Path: .cAGENTS/config.toml
  Engine: builtin:simple
  Output: .

Templates:
  Found: 5 templates
  Directory: .cAGENTS/templates

Outputs:
  AGENTS.md: ✓ exists (last modified: 2h ago)
  CLAUDE.md: ✓ exists (last modified: 2h ago)

Templates:
  • base-guidelines.md (order: 1)
  • typescript-rules.md (order: 10, globs: **/*.ts)
  • api-layer.md (order: 20, globs: src/api/**)
  • testing.md (order: 30, globs: **/tests/**)
  • production.md (order: 40, when: env=production)
```

---

## `lint`

Validate configuration and templates.

### Usage

```bash
cagents lint
```

### Exit Codes

- `0` - No issues found
- `1` - Validation errors found

### Validations

**Config file:**
- Valid TOML syntax
- Required sections present
- Path directories exist
- Engine format valid
- Target names valid

**Templates:**
- Valid YAML frontmatter
- Required `name` field present
- No duplicate names
- Valid `order` values (numbers)
- Valid `when` clause syntax
- Valid `globs` pattern syntax
- Valid `target` names
- Valid `outputIn` values

**Variables:**
- Referenced variables are defined
- Command variables are executable
- No undefined variable references in templates

---

## `preview`

Preview build output before writing files.

### Usage

```bash
cagents preview [PATH]
```

### Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `PATH` | File or directory to preview | `.` |

### Output

**For each output file:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Output: AGENTS.md
Templates:
  • base-guidelines.md (order: 1)
  • typescript-rules.md (order: 10)

Preview (first 20 lines):
───────────────────────────────────
<!-- Auto-generated -->

# Base Guidelines
...

(120 total lines)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Interactive features:**
- Navigate between files
- View full content
- Exit with Q

---

## `git`

Git integration commands for managing .gitignore.

### Subcommands

#### `git ignore-outputs`

Add output files to .gitignore.

```bash
cagents git ignore-outputs
```

**Adds:**
```gitignore
# cAGENTS outputs
AGENTS.md
CLAUDE.md
.cursorrules
**/AGENTS.md
**/CLAUDE.md
```

**Behavior:**
- Creates `.gitignore` if it doesn't exist
- Appends to existing `.gitignore`
- Skips if patterns already present
- Adds comment for organization

#### `git unignore-outputs`

Remove output files from .gitignore.

```bash
cagents git unignore-outputs
```

**Removes:**
- All cAGENTS output patterns
- Comment line
- Maintains other .gitignore content

---

## `setup`

Setup package manager integration.

### Usage

```bash
cagents setup <MANAGER>
```

### Arguments

| Argument | Description | Options |
|----------|-------------|---------|
| `MANAGER` | Package manager to setup | `npm`, `pnpm`, `yarn` |

### Examples

```bash
cagents setup npm
cagents setup pnpm
```

### Behavior

Adds scripts to `package.json`:

```json
{
  "scripts": {
    "cagents:build": "cagents build",
    "cagents:lint": "cagents lint",
    "postinstall": "cagents build"
  }
}
```

**Note:** Preserves existing scripts, only adds missing ones.

---

## Global Options

These options work with any command:

| Option | Description |
|--------|-------------|
| `-h, --help` | Show help for command |
| `-V, --version` | Show cAGENTS version |

---

## Environment Variables

### `APP_ENV`, `AWS_REGION`, etc.

Custom variables defined in your config:

```toml
[variables.env]
app_env = "APP_ENV"
region = "AWS_REGION"
```

Usage:
```bash
APP_ENV=production AWS_REGION=us-west-2 cagents build
```

### `CAGENTS_TEST`

Disable interactive prompts (for CI/CD):

```bash
CAGENTS_TEST=1 cagents build
```

### `EDITOR`

Used by `setup` and other commands that open files:

```bash
EDITOR=code cagents init  # Opens in VS Code
```

---

## Exit Codes

All commands return standard exit codes:

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Error (validation failed, file not found, etc.) |
| `130` | Interrupted by user (Ctrl+C) |

---

## Command Aliases

While not built into cAGENTS, you can create shell aliases:

```bash
# .bashrc or .zshrc
alias cb="cagents build"
alias cl="cagents lint"
alias cm="cagents migrate"
```

Or npm scripts:
```json
{
  "scripts": {
    "b": "cagents build",
    "l": "cagents lint"
  }
}
```

---

## Common Workflows

### First Time Setup

```bash
# 1. Install
npm install --save-dev cagents

# 2. Initialize
npx cagents init

# 3. Edit templates in .cAGENTS/templates/

# 4. Build
npx cagents build

# 5. Ignore outputs
npx cagents git ignore-outputs

# 6. Setup auto-build
npx cagents setup npm
```

### Daily Development

```bash
# Make template changes
vim .cAGENTS/templates/my-rules.md

# Preview changes
cagents preview

# Build
cagents build

# Validate
cagents lint
```

### Debugging Template Matching

```bash
# See which rules apply to a file
cagents context src/api/users.ts

# Render just that file's context
cagents render src/api/users.ts

# Check if glob patterns match
cagents preview src/api/
```

### Migration

```bash
# 1. Migrate existing rules
cagents migrate --backup

# 2. Review generated templates
ls -la .cAGENTS/templates/

# 3. Edit as needed
vim .cAGENTS/templates/agents-root.md

# 4. Rebuild to verify
cagents build

# 5. Compare with original
diff AGENTS.md AGENTS.md.backup
```

### CI/CD Integration

```bash
# In your CI pipeline
CAGENTS_TEST=1 cagents lint
CAGENTS_TEST=1 cagents build --dry-run
```

Or with error checking:
```bash
if ! cagents lint; then
  echo "cAGENTS validation failed"
  exit 1
fi

cagents build
```

---

## Comparison with Other Tools

### vs Static AGENTS.md

| Feature | Static AGENTS.md | cAGENTS |
|---------|------------------|---------|
| Multiple formats | Manual duplication | One source |
| Nested rules | Multiple files to maintain | Auto-generated from templates |
| Conditional rules | Not possible | `when` clauses |
| Template reuse | Copy-paste | Compose and extend |
| Validation | None | `cagents lint` |

### vs agentstack

| Feature | agentstack | cAGENTS |
|---------|-----------|---------|
| Format | Single AGENTS.md | AGENTS.md + CLAUDE.md + .cursorrules |
| Templates | No | Yes |
| Conditional rules | No | Yes |
| Monorepo support | Manual | Automatic with globs |
| Migration | N/A | `cagents migrate` |

---

## Performance

### Build Speed

Typical build times:
- **Small project** (1-5 templates): <100ms
- **Medium project** (10-20 templates): <500ms
- **Large project** (50+ templates): <2s

### Optimization Tips

1. **Minimize command variables** - Execute at build time, can be slow
2. **Use static variables** when possible
3. **Reduce template count** - Combine related rules
4. **Cache expensive computations** - Store in static vars
5. **Increase timeouts** if needed:
   ```toml
   [execution]
   timeoutMs = 5000
   ```

---

## Troubleshooting

### Build fails with "Failed to parse frontmatter"

**Cause:** Invalid YAML syntax in template frontmatter

**Fix:**
1. Run `cagents lint` to find the problematic template
2. Check YAML syntax (indentation, quotes, colons)
3. Common issues:
   - Missing space after colon: `name:value` → `name: value`
   - Unquoted strings with special chars: `globs: [src/[id]/]` → `globs: ["src/\\[id\\]/"]`

### No output generated

**Cause:** All templates filtered out by `when` clauses

**Fix:**
1. Check variable values: `cagents status`
2. Check `when` conditions in templates
3. Try without conditions: remove `when` clause temporarily

### Nested AGENTS.md not generated

**Cause:** Missing or incorrect `globs`/`outputIn`

**Fix:**
1. Check template has `globs` field
2. Verify glob matches files: `cagents context <file>`
3. For directory output, use `outputIn: "matched"` with trailing slash:
   ```yaml
   globs: ["packages/api/"]
   outputIn: "matched"
   ```

### Command variables not working

**Cause:** Shell not found or command timeout

**Fix:**
1. Check shell exists:
   ```toml
   [execution]
   shell = "bash"  # or "sh", "zsh", etc.
   ```
2. Test command manually:
   ```bash
   bash -c "your command here"
   ```
3. Increase timeout:
   ```toml
   [execution]
   timeoutMs = 5000
   ```

---

## Tips & Tricks

### Quick Template Creation

```bash
cat > .cAGENTS/templates/new-rule.md << 'EOF'
---
name: new-rule
order: 10
---
# New Rule

Content here
EOF

cagents build
```

### Test Template Changes

```bash
# Preview before building
cagents preview

# Or dry-run
cagents build --dry-run
```

### Per-Branch Rules

```toml
[variables.command]
branch = "git rev-parse --abbrev-ref HEAD"
```

```yaml
---
when:
  branch: ["main", "production"]
---
# Production Branch Rules
```

### Tool Detection

```toml
[variables.command]
has_docker = "command -v docker >/dev/null && echo true || echo false"
has_poetry = "command -v poetry >/dev/null && echo true || echo false"
```

```yaml
---
when:
  has_docker: "true"
---
# Docker Development Rules
```

### Multiple Environments

```bash
# Development
APP_ENV=dev cagents build

# Staging
APP_ENV=staging cagents build

# Production
APP_ENV=production cagents build
```

Each generates different AGENTS.md based on `when` clauses.

---

## Advanced Examples

See [ADVANCED.md](./ADVANCED.md) for more complex configurations and use cases.

---

## Getting Help

- **Command help:** `cagents <command> --help`
- **GitHub Issues:** https://github.com/centralinc/cAGENTS/issues
- **Discussions:** https://github.com/centralinc/cAGENTS/discussions
