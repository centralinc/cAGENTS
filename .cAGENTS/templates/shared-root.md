---
name: shared-root
description: shared sections
alwaysApply: true
order: 20
---

## Project Overview

**cAGENTS** is a cross-platform generator for `AGENTS.md` and compatible rule formats (e.g., Cursor `.mdc`). It composes rule files (Markdown + YAML front-matter), renders them with a BYOC (Bring-Your-Own-Compiler) engine, and writes context-scoped outputs at the repo root and nested directories.

This is a hybrid monorepo with:
- **Rust workspace** (`crates/`) - The fast, deterministic core
- **pnpm workspace** (`packages/`) - Thin TypeScript wrapper for npm distribution

## Build Commands

### Rust workspace
```bash
# Build all crates
cargo build --workspace

# Build with locked dependencies
cargo build --workspace --locked

# Run CLI directly
./target/debug/cagents --help
```

### Node wrapper (local development)
```bash
# Install all pnpm dependencies
pnpm -w i

# Build Node wrapper
pnpm --filter cagents run build

# Or build everything recursively
pnpm -w -r build

# Run via pnpm wrapper
pnpm cagents --help
```

### Testing
```bash
# Rust tests (runs in parallel by default)
cargo test --workspace

# Run tests sequentially (useful for debugging)
cargo test --workspace -- --test-threads=1

# Format and lint Rust
cargo fmt
cargo clippy -- -D warnings

# Format TypeScript
pnpm -w run fmt
```

**Note:** Tests automatically disable interactive prompts via multiple detection methods:
- Compile-time `cfg!(test)` flag (for unit tests)
- Thread name pattern matching (detects `test_` in thread name)
- Manual override via `CAGENTS_TEST=1` environment variable

If you still see interactive prompts during tests, run with:
```bash
CAGENTS_TEST=1 cargo test --workspace
```

## Architecture Overview

### Repository Structure

```
crates/
  cagents-core/     - Library (planning, renderers, writers)
  cagents-cli/      - CLI binary entry point
packages/
  cagents/          - pnpm wrapper (downloads binary in releases)
docs/               - PRD, architecture, config, CLI, compiler protocol, roadmap
examples/           - Runnable samples
```


### Core (Rust) - `crates/cagents-core/`

The pipeline flows through these modules (see `crates/cagents-core/src/lib.rs`):

1. **config** - TOML config loader (project-local `.agentscribe/config.toml` + user-level `~/.cagents/config.toml`)
2. **loader** - Rule discovery (front-matter + body parsing)
3. **model** - Data structures for `ProjectConfig`, `RuleFrontmatter`, scoping (`When`), merge strategies
4. **planner** - Determines which rules apply based on globs, `alwaysApply`, `when` conditions (env/role/language), order, and extends
5. **render** - Template rendering via BYOB command adapters
6. **adapters/** - External command protocol via stdin/stdout JSON (built-in engines removed in M11)
7. **merge** - Section-aware rule merging
8. **writers/** - Output to `AGENTS.md` and Cursor `.mdc` formats

### Wrapper (TypeScript) - `packages/cagents/`

Thin pnpm package that downloads/executes prebuilt binaries in real releases. For local development, the postinstall script warns to build from source. Exposes a programmatic API.

### Configuration

Config files use TOML with these key sections:
- `[paths]` - `templatesDir`, `outputRoot`, `cursorRulesDir`
- `[defaults]` - `engine` (e.g. `command:python3 tools/render.py`), `targets`, `order`
- `[variables]` - `static`, `env`, `command` (for template data)
- `[execution]` - `shell`, `timeoutMs`, `allowCommands`

Rule templates have YAML front-matter with fields like `engine`, `globs`, `alwaysApply`, `when`, `order`, `extends`, `targets`, `merge`.

### BYOC Compiler Protocol

External compilers (e.g., Python Jinja2, Node MDX) communicate via JSON:
- **IN (stdin)**: `{ templateSource, templatePath, data, frontmatter, cwd }`
- **OUT (stdout)**: `{ content, diagnostics? }`

See `docs/COMPILERS.md` and `crates/cagents-core/src/adapters/command.rs`.

## Development Workflow

### STRICT TDD REQUIREMENT

**You MUST follow Test-Driven Development for ALL changes:**

1. **Write failing test FIRST**
   ```bash
   # Create test in appropriate location
   # - crates/cagents-core/tests/ for integration tests
   # - Module #[cfg(test)] for unit tests

   cargo test <test_name>
   # Should FAIL initially
   ```

2. **Implement minimal code to pass**
   ```rust
   // Add just enough code to make test pass
   ```

3. **Verify test passes**
   ```bash
   cargo test <test_name>
   # Should PASS now
   ```

4. **Run full test suite**
   ```bash
   cargo test --workspace -- --test-threads=1
   # ALL tests should pass
   ```

5. **Refactor if needed** (keeping tests green)

**Examples:**
- Adding new command? Write integration test first
- New validation rule? Write lint test first
- New preset? Write init test first

### Task Tracking Requirement

Track work with Beads instead of Markdown. Run \`bd quickstart\` to see how.


### Test Locations

**Unit Tests:**
- Place in same file as code: `#[cfg(test)] mod tests { ... }`
- Test individual functions in isolation

**Integration Tests:**
- `crates/cagents-core/tests/*.rs` - Core functionality
- `crates/cagents-cli/tests/*.rs` - CLI commands

**Test Naming:**
```rust
#[test]
fn test_<feature>_<scenario>() {
    // Arrange
    let input = setup_test_data();

    // Act
    let result = function_under_test(input);

    // Assert
    assert_eq!(result, expected);
}
```

### Example TDD Session

```bash
# 1. Create test
cat > crates/cagents-core/tests/new_feature.rs << 'EOF'
#[test]
fn test_new_feature_works() {
    let result = new_function();
    assert_eq!(result, expected_value);
}
EOF

# 2. Run test (should fail)
cargo test test_new_feature_works
# ❌ FAILS: function not found

# 3. Implement
# Add function to appropriate module

# 4. Run test (should pass)
cargo test test_new_feature_works
# ✅ PASSES

# 5. Run all tests
cargo test --workspace -- --test-threads=1
# ✅ ALL PASS

# 6. Commit
git add -A
git commit -m "Add new_feature with tests"
```

### Non-Negotiable Rules

1. **Work must include tests**
2. **run full test suite before considering a task compete**
3. **Include a CHANGELOG update for all work**
4. **Work must be tracked with Beads**
5. **TDD always: test first, code second**
6. **Update relevant docs and README.md for consumers**

Violating these rules will result in technical debt and bugs.