# cAGENTS Basic Example

This example demonstrates the minimal end-to-end functionality of cAGENTS (M1 Slice 1).

## What This Example Shows

- ✅ Loading config from `cAGENTS/config.toml`
- ✅ Discovering rule templates in `cAGENTS/templates/`
- ✅ Parsing YAML front-matter from templates
- ✅ Rendering templates via the default `command:` engine
- ✅ Merging rendered rules into a single document
- ✅ Writing output to `AGENTS.md`

## Structure

```
cAGENTS/
├── config.toml              # Project configuration
├── tools/
│   └── render.py            # Example Python renderer invoked via command engine
└── templates/
    └── typescript.hbs.md    # Template with front-matter rendered via render.py

AGENTS.md                    # Generated output (not checked in)
```

## Configuration

The `config.toml` defines:

- **Paths:** Where to find templates and write output
- **Defaults:** Default template engine command (`command:python3 cAGENTS/tools/render.py`), order, targets
- **Static Variables:** `owner="Jordan"`, `tone="concise"` (injected into templates)
- **Command Variables:** `branch` command (NOT executed in M1 Slice 1)

## Template Example

`typescript.hbs.md` shows:

```yaml
---
name: ts-basics
globs: ["**/*.ts", "packages/frontend/**"]
alwaysApply: true
order: 10
---
## TypeScript

- Prefer explicit types; avoid `any`.
- Run typecheck: `pnpm typecheck`.
```

The `alwaysApply: true` flag means this rule appears in the root `AGENTS.md` regardless of globs.

## How to Run

From this directory:

```bash
# Build AGENTS.md
cargo run -p cagents -- build

# Verify output
cat AGENTS.md
```

Expected output in `AGENTS.md`:

```markdown
## TypeScript

- Prefer explicit types; avoid `any`.
- Run typecheck: `pnpm typecheck`.
- See [System Models](docs/system-design/models.md) for domain entities.
```

## What Doesn't Work Yet (By Design)

This example only tests M1 Slice 1 features. The following are NOT demonstrated:

- ❌ Multiple templates being merged (only one template exists)
- ❌ Command variable execution (branch command not run)
- ❌ Advanced renderer features (example engine only supports simple `{{var}}` replacement)
- ❌ Nested directory outputs
- ❌ env/role/language filtering
- ❌ Cursor .mdc export

## Testing

This example is used by the integration test:

```bash
cargo test -p cagents build_smoke
```

The test:
1. Copies `cAGENTS/` to a temp directory
2. Runs `cagents build`
3. Asserts `AGENTS.md` exists and contains expected content
4. Snapshot tests the exact output format

## Next Steps

To see more advanced features, wait for:
- **M1 Slice 2:** Multiple templates, variable substitution, command execution
- **M2:** Cursor export, linting, richer BYOB renderer examples
- **M3:** Additional tooling integrations
