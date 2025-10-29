# Proposal 004: CLI Enhancements

## Metadata

- **Status:** Draft
- **Target Version:** 0.2.0
- **Author(s):** AI Implementation Team
- **Created:** 2025-10-29
- **Last Updated:** 2025-10-29
- **Related Issues:** N/A
- **Related PRs:** N/A
- **Related Proposals:** [001-claude-skills-support.md](./001-claude-skills-support.md)

## Overview

### Problem Statement

The current CLI provides basic functionality (`init`, `build`, `migrate`, `lint`, `preview`, `render`), but lacks features for:

1. **On-demand context queries**: No way to ask "what rules apply to this file?" without rendering full output
2. **Targeted rendering**: Can't render just one template/section without full build
3. **Machine-readable output**: No JSON format for programmatic consumption
4. **Skill inspection**: No way to see which skills would be generated or validate them
5. **Performance**: Repeated CLI invocations re-parse config and templates (no caching)

These limitations hinder:
- IDE integrations (need fast, programmatic queries)
- Debugging (hard to understand why a template was/wasn't included)
- Skill development (can't preview skills before building)

### Goals

1. **Add `cagents context <file>` command**: Show metadata about which rules apply to a file
2. **Enhance `cagents render`**: Add `--section` and `--json` flags for targeted/programmatic use
3. **Add skill commands**: `--validate-skills` and `list-skills` for skill development workflow
4. **Improve performance**: Implement caching for repeated invocations
5. **Better debugging**: Add `--verbose` flags for detailed condition evaluation

### Non-Goals

- **Runtime server**: Not implementing a persistent cAGENTS server (could be future work)
- **Watch mode**: Not auto-rebuilding on file changes (use external tools)
- **Interactive mode**: All commands remain scriptable (no TUI)

## Background

### Current State

**Existing Commands:**
```bash
cagents init              # Initialize .cAGENTS/
cagents build             # Generate all outputs
cagents migrate           # Import existing rules
cagents lint              # Validate config
cagents preview           # Preview build (dry run)
cagents render <file>     # Render context for specific file
cagents context <file>    # (mentioned in HN discussion but not implemented)
cagents git ignore-outputs # Add outputs to .gitignore
```

**Current `render` Limitations:**
- Always outputs full merged content (can't target one template)
- Only text output (no JSON)
- Re-parses everything on each invocation

### Motivation

**Use Cases:**

1. **IDE Integration:**
   > An AI coding assistant plugin needs to query "what rules apply to `src/api/users.ts`?" without rendering the full 500-line output.

2. **Debugging:**
   > Developer wants to know why the "database" template isn't appearing in their AGENTS.md (which condition failed?).

3. **Skill Development:**
   > Developer writing skill descriptions wants to see which skills would be generated and preview their metadata before building.

4. **Performance:**
   > CI pipeline runs `cagents render` hundreds of times; caching could speed it up significantly.

### References

- [HN Discussion on `cagents context`](https://news.ycombinator.com/item?id=42465839)
- [IDE Integration Patterns](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide)

## Detailed Design

### 1. New Command: `cagents context <file>`

**Purpose:** Show metadata about which templates apply to a file without rendering full content.

**Usage:**
```bash
cagents context src/api/users.ts
```

**Output:**
```
Applicable templates for src/api/users.ts:

  ✓ typescript-rules (order: 10, core: true)
    Description: TypeScript coding conventions
    Conditions: language=typescript
    Globs: **/*.ts, **/*.tsx

  ✓ api-guidelines (order: 20, core: true)
    Description: API design patterns
    Globs: src/api/**

  ✗ database-migrations (order: 30, core: false)
    Reason: Does not match globs (migrations/**)

  ✗ python-style (order: 40, core: true)
    Reason: Condition failed (language=python, but file is typescript)
```

**Flags:**
- `--verbose` / `-v`: Show detailed condition evaluation
- `--json`: Output as JSON for programmatic use

**JSON Output:**
```json
{
  "file": "src/api/users.ts",
  "language": "typescript",
  "environment": {
    "env": "development",
    "use_beads": "true"
  },
  "applicable_templates": [
    {
      "name": "typescript-rules",
      "order": 10,
      "core": true,
      "description": "TypeScript coding conventions",
      "conditions": {"language": ["typescript"]},
      "globs": ["**/*.ts", "**/*.tsx"],
      "matched": true
    },
    {
      "name": "api-guidelines",
      "order": 20,
      "core": true,
      "globs": ["src/api/**"],
      "matched": true
    }
  ],
  "excluded_templates": [
    {
      "name": "database-migrations",
      "reason": "Glob mismatch",
      "globs": ["migrations/**"]
    },
    {
      "name": "python-style",
      "reason": "Condition failed: language=python"
    }
  ]
}
```

**Implementation:**

```rust
pub fn run_context_command(
    file_path: &Path,
    config: &ProjectConfig,
    verbose: bool,
    json: bool,
) -> Result<()> {
    // Load and evaluate templates
    let templates = load_templates(&config)?;
    let environment = evaluate_environment(config)?;
    let file_context = infer_file_context(file_path)?;  // language, globs, etc.

    let mut applicable = Vec::new();
    let mut excluded = Vec::new();

    for template in templates {
        let match_result = evaluate_template_match(&template, &file_context, &environment);

        if match_result.matched {
            applicable.push((template, match_result));
        } else {
            excluded.push((template, match_result));
        }
    }

    // Output
    if json {
        print_context_json(&applicable, &excluded, file_path, &file_context)?;
    } else {
        print_context_human(&applicable, &excluded, file_path, verbose)?;
    }

    Ok(())
}

fn evaluate_template_match(
    template: &Template,
    file_context: &FileContext,
    environment: &Environment,
) -> MatchResult {
    let mut result = MatchResult { matched: true, reasons: Vec::new() };

    // Check globs
    if let Some(globs) = &template.frontmatter.globs {
        if !any_glob_matches(globs, file_context.path) {
            result.matched = false;
            result.reasons.push(format!("Glob mismatch: {:?}", globs));
            return result;
        }
    }

    // Check when conditions
    if let Some(when) = &template.frontmatter.when {
        if let Some(languages) = &when.language {
            if !languages.contains(&file_context.language) {
                result.matched = false;
                result.reasons.push(format!(
                    "Condition failed: language={:?}, but file is {}",
                    languages, file_context.language
                ));
                return result;
            }
        }

        // Check other conditions (env, tools, etc.)
        // ...
    }

    result
}
```

### 2. Enhanced `cagents render`

**Current:**
```bash
cagents render src/api/users.ts
```
Outputs full merged content for that file.

**Enhanced:**

**Flag: `--section <name>`**

Render only a specific template by name:

```bash
cagents render --section typescript-rules
```

Output:
```markdown
# TypeScript Rules

Use strict mode. Prefer functional components...
```

**Flag: `--json`**

Output structured JSON:

```bash
cagents render --json src/api/users.ts
```

Output:
```json
{
  "file": "src/api/users.ts",
  "applicable_templates": [
    {
      "name": "typescript-rules",
      "order": 10,
      "content": "# TypeScript Rules\n\nUse strict mode..."
    },
    {
      "name": "api-guidelines",
      "order": 20,
      "content": "# API Guidelines\n\nRESTful design..."
    }
  ],
  "merged_output": "# Project Rules\n\n## TypeScript Rules\n...",
  "conditions_evaluated": {
    "env": "development",
    "language": "typescript",
    "use_beads": "true"
  }
}
```

**Flag: `--no-cache`**

Force fresh parse (bypass cache):

```bash
cagents render --no-cache src/file.ts
```

**Implementation:**

```rust
pub fn run_render_command(
    file_path: Option<&Path>,
    section: Option<&str>,
    json: bool,
    no_cache: bool,
) -> Result<()> {
    let config = load_config()?;
    let cache = if no_cache { None } else { load_cache()? };

    let templates = if let Some(cache) = cache {
        cache.templates
    } else {
        load_and_cache_templates(&config)?
    };

    // Filter templates
    let applicable_templates = if let Some(section_name) = section {
        // Render only specified section
        templates
            .into_iter()
            .filter(|t| t.frontmatter.name == section_name)
            .collect()
    } else if let Some(file_path) = file_path {
        // Render for specific file (existing behavior)
        filter_templates_for_file(&templates, file_path)?
    } else {
        return Err(Error::InvalidUsage("Must specify --section or <file>"));
    };

    // Render output
    let output = if json {
        render_json(&applicable_templates, file_path)?
    } else {
        render_text(&applicable_templates)?
    };

    println!("{}", output);
    Ok(())
}
```

### 3. Skill Commands

**Flag: `--validate-skills`**

Validate skills without writing files:

```bash
cagents build --validate-skills
```

Output:
```
Validating skills...

✓ typescript-rules
  Description: TypeScript coding conventions

✓ testing-guidelines
  Description: Testing best practices

✗ database-migrations
  Error: Missing description field

Summary: 2 valid, 1 error
```

**New Command: `cagents list-skills`**

List all skills that would be generated:

```bash
cagents list-skills
```

Output:
```
Skills that would be generated:

  typescript-rules
    Description: TypeScript coding conventions
    Allowed tools: Bash, Read, Write
    Source: .cAGENTS/templates/typescript.md

  testing-guidelines
    Description: Testing best practices
    Allowed tools: Bash, Read
    Source: .cAGENTS/templates/testing.md

Total: 2 skills
```

**With `--json`:**
```json
{
  "skills": [
    {
      "name": "typescript-rules",
      "slug": "typescript-rules",
      "description": "TypeScript coding conventions",
      "allowed_tools": ["Bash", "Read", "Write"],
      "source_template": ".cAGENTS/templates/typescript.md",
      "output_path": ".claude/skills/typescript-rules/SKILL.md"
    }
  ],
  "total": 2
}
```

**Implementation:**

```rust
pub fn run_list_skills_command(config: &ProjectConfig, json: bool) -> Result<()> {
    let templates = load_templates(config)?;
    let environment = evaluate_environment(config)?;

    let mut skills = Vec::new();

    for template in templates {
        // Check if template would generate a skill
        if !evaluate_conditions(&template.frontmatter.when, &environment) {
            continue;  // Skip templates that don't pass conditions
        }

        if template.frontmatter.description.is_none() {
            eprintln!("Warning: Template '{}' missing description (required for skills)", template.frontmatter.name);
            continue;
        }

        let skill = SkillInfo {
            name: template.frontmatter.name.clone(),
            slug: slugify(&template.frontmatter.name),
            description: template.frontmatter.description.clone().unwrap(),
            allowed_tools: template.frontmatter.allowed_tools.clone()
                .unwrap_or_else(|| config.skill.allowed_tools.clone()),
            source_template: template.source_path.clone(),
            output_path: config.skill.output_dir.join(&slugify(&template.frontmatter.name)).join("SKILL.md"),
        };

        skills.push(skill);
    }

    if json {
        print_skills_json(&skills)?;
    } else {
        print_skills_human(&skills)?;
    }

    Ok(())
}
```

### 4. Caching for Performance

**Problem:** Repeatedly running `cagents render` re-parses config and templates every time.

**Solution:** Implement simple file-based cache.

**Cache Structure:**
```
.cAGENTS/.cache
  ├─ config-hash.json      # Hash of config.toml
  ├─ templates-parsed.bin  # Serialized templates
  └─ environment.json      # Evaluated environment variables
```

**Cache Invalidation:**
- Config file modified (mtime or hash changed)
- Any template file modified
- Manual `--no-cache` flag

**Implementation:**

```rust
#[derive(Serialize, Deserialize)]
struct Cache {
    config_hash: String,
    templates_mtime: SystemTime,
    templates: Vec<Template>,
    environment: HashMap<String, String>,
}

fn load_cache() -> Result<Option<Cache>> {
    let cache_path = Path::new(".cAGENTS/.cache/templates-parsed.bin");

    if !cache_path.exists() {
        return Ok(None);
    }

    let cache_data = fs::read(cache_path)?;
    let cache: Cache = bincode::deserialize(&cache_data)?;

    // Validate cache
    let current_config_hash = hash_file(".cAGENTS/config.toml")?;
    if cache.config_hash != current_config_hash {
        return Ok(None);  // Config changed, invalidate
    }

    let current_templates_mtime = get_templates_mtime()?;
    if cache.templates_mtime < current_templates_mtime {
        return Ok(None);  // Templates modified, invalidate
    }

    Ok(Some(cache))
}

fn save_cache(cache: &Cache) -> Result<()> {
    let cache_dir = Path::new(".cAGENTS/.cache");
    fs::create_dir_all(cache_dir)?;

    let cache_path = cache_dir.join("templates-parsed.bin");
    let cache_data = bincode::serialize(cache)?;
    fs::write(cache_path, cache_data)?;

    Ok(())
}

fn load_and_cache_templates(config: &ProjectConfig) -> Result<Vec<Template>> {
    let templates = load_templates(config)?;

    let cache = Cache {
        config_hash: hash_file(".cAGENTS/config.toml")?,
        templates_mtime: get_templates_mtime()?,
        templates: templates.clone(),
        environment: evaluate_environment(config)?,
    };

    save_cache(&cache)?;

    Ok(templates)
}
```

**Cache Management:**
- Automatically created on first `render` or `context` invocation
- Cleared by `cagents build` (since it regenerates)
- Can be manually cleared by deleting `.cAGENTS/.cache/`

### 5. Verbose Debugging

**Add `--verbose` / `-v` flag to all commands.**

**Behavior:**
- Show detailed condition evaluation
- Log file I/O operations
- Display template matching logic
- Timings for performance analysis

**Example:**
```bash
cagents build --verbose
```

Output:
```
[DEBUG] Loading config from .cAGENTS/config.toml
[DEBUG] Config loaded in 2ms
[DEBUG] Evaluating environment variables...
[DEBUG]   use_beads = command: "command -v bd >/dev/null && echo true || echo false"
[DEBUG]   Result: use_beads = "true"
[DEBUG] Loading templates from .cAGENTS/templates/
[DEBUG] Found 5 templates
[DEBUG] Processing template 'typescript-rules'...
[DEBUG]   Order: 10
[DEBUG]   Conditions: language=typescript
[DEBUG]   Matched: true
[DEBUG] Processing template 'beads-workflow'...
[DEBUG]   Conditions: use_beads=true
[DEBUG]   Matched: true (use_beads="true")
[DEBUG] Generating AGENTS.md...
[DEBUG] Written to AGENTS.md (1,245 bytes) in 5ms
[INFO] Build completed successfully
```

**Implementation:**

Use a global logger with configurable level:

```rust
use log::{debug, info, warn};

pub fn configure_logging(verbose: bool) {
    let level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    env_logger::Builder::new()
        .filter_level(level)
        .init();
}

// In command handlers:
debug!("Loading template: {}", template.name);
info!("Build completed successfully");
```

## Implementation Plan

### Phase 1: Context Command (Sprint 1, Week 1)

#### Task 1.1: Implement file context inference
**Location:** `crates/cagents-core/src/context.rs` (new module)

- [ ] Create `FileContext` struct (path, language, directory)
- [ ] Implement `infer_file_context(path)` function
- [ ] Detect language from file extension
- [ ] Write tests:
  - `test_infer_typescript()`
  - `test_infer_python()`
  - `test_infer_unknown()`

**Acceptance Criteria:**
- Correctly detects common languages
- Handles unknown extensions gracefully

#### Task 1.2: Implement template matching logic
**Location:** Same module

- [ ] Create `MatchResult` struct
- [ ] Implement `evaluate_template_match()` function
- [ ] Check glob matches
- [ ] Check when conditions
- [ ] Return detailed reasons for exclusion
- [ ] Write tests:
  - `test_match_glob_success()`
  - `test_match_glob_failure()`
  - `test_match_condition_success()`
  - `test_match_condition_failure()`

**Acceptance Criteria:**
- Correctly identifies applicable templates
- Provides clear exclusion reasons

#### Task 1.3: Implement context command
**Location:** `crates/cagents-cli/src/commands/context.rs` (new file)

- [ ] Create command handler
- [ ] Load config and templates
- [ ] Evaluate matches for given file
- [ ] Format human-readable output
- [ ] Implement `--json` flag
- [ ] Implement `--verbose` flag
- [ ] Write integration test `test_context_command()`

**Acceptance Criteria:**
- Command works end-to-end
- Output is clear and helpful
- JSON output is valid

### Phase 2: Enhanced Render (Sprint 1, Week 2)

#### Task 2.1: Add --section flag
**Location:** `crates/cagents-cli/src/commands/render.rs`

- [ ] Add `section: Option<String>` parameter
- [ ] Filter templates by name if specified
- [ ] Error if section not found
- [ ] Write test `test_render_section()`

**Acceptance Criteria:**
- Renders only specified section
- Clear error for non-existent section

#### Task 2.2: Add --json flag
**Location:** Same file

- [ ] Define JSON output schema
- [ ] Serialize applicable templates
- [ ] Include metadata (conditions, file context)
- [ ] Write test `test_render_json()`

**Acceptance Criteria:**
- JSON is valid and complete
- Schema is documented

#### Task 2.3: Update render command
**Location:** Same file

- [ ] Refactor to support new flags
- [ ] Update argument parsing
- [ ] Write integration tests

**Acceptance Criteria:**
- All flags work together correctly
- Backward compatible (existing usage still works)

### Phase 3: Skill Commands (Sprint 2, Week 3)

#### Task 3.1: Implement --validate-skills flag
**Location:** `crates/cagents-cli/src/commands/build.rs`

- [ ] Add validation-only mode
- [ ] Check required fields (description)
- [ ] Check for name collisions
- [ ] Report errors and warnings
- [ ] Write test `test_validate_skills()`

**Acceptance Criteria:**
- Validation runs without writing files
- Errors are actionable

#### Task 3.2: Implement list-skills command
**Location:** `crates/cagents-cli/src/commands/skills.rs` (new file)

- [ ] Create command handler
- [ ] Iterate templates and determine skills
- [ ] Format output (human and JSON)
- [ ] Write integration test `test_list_skills()`

**Acceptance Criteria:**
- Lists all skills correctly
- Shows relevant metadata

### Phase 4: Caching (Sprint 2, Week 3-4)

#### Task 4.1: Implement cache structure
**Location:** `crates/cagents-core/src/cache.rs` (new module)

- [ ] Define `Cache` struct
- [ ] Implement serialization (bincode)
- [ ] Implement cache path management
- [ ] Write tests:
  - `test_cache_serialize()`
  - `test_cache_deserialize()`

**Acceptance Criteria:**
- Cache can be saved and loaded
- Binary format is efficient

#### Task 4.2: Implement cache invalidation
**Location:** Same module

- [ ] Hash config file
- [ ] Get templates modification time
- [ ] Compare against cached values
- [ ] Write tests:
  - `test_cache_invalidation_config_change()`
  - `test_cache_invalidation_template_change()`

**Acceptance Criteria:**
- Cache invalidates when config or templates change
- Cache persists when nothing changed

#### Task 4.3: Integrate caching into render/context
**Location:** Command handlers

- [ ] Check cache before loading templates
- [ ] Use cached templates if valid
- [ ] Update cache after fresh load
- [ ] Add `--no-cache` flag
- [ ] Write integration test `test_cache_performance()`

**Acceptance Criteria:**
- Cached runs are significantly faster
- `--no-cache` forces fresh load

### Phase 5: Verbose Logging (Sprint 2, Week 4)

#### Task 5.1: Add logging infrastructure
**Location:** `crates/cagents-cli/src/main.rs`

- [ ] Add `env_logger` dependency
- [ ] Configure logger based on `--verbose` flag
- [ ] Add log statements throughout codebase

**Acceptance Criteria:**
- Verbose flag shows debug output
- Normal runs show only info/warn/error

#### Task 5.2: Add detailed logging to key operations
**Location:** Various modules

- [ ] Log template loading
- [ ] Log condition evaluation
- [ ] Log file I/O
- [ ] Log timings

**Acceptance Criteria:**
- Debug output is helpful for troubleshooting
- No performance impact when disabled

### Phase 6: Documentation (Sprint 3, Week 5)

#### Task 6.1: Update COMMANDS.md
**Location:** `COMMANDS.md`

- [ ] Document `context` command with examples
- [ ] Document `render` enhancements (--section, --json)
- [ ] Document `list-skills` command
- [ ] Document `--validate-skills` flag
- [ ] Document `--verbose` flag

**Acceptance Criteria:**
- All commands documented with examples
- Flags explained clearly

#### Task 6.2: Update README.md
**Location:** `README.md`

- [ ] Mention new commands in quick reference
- [ ] Link to COMMANDS.md for details

**Acceptance Criteria:**
- New features are discoverable

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_infer_file_context_typescript() {
    let context = infer_file_context(Path::new("src/app.ts")).unwrap();
    assert_eq!(context.language, "typescript");
}

#[test]
fn test_evaluate_template_match_glob_success() {
    let template = create_template_with_glob("src/**/*.ts");
    let file_context = FileContext { path: Path::new("src/app.ts"), language: "typescript" };
    let result = evaluate_template_match(&template, &file_context, &default_env());
    assert!(result.matched);
}

#[test]
fn test_evaluate_template_match_condition_failure() {
    let mut template = create_template();
    template.frontmatter.when = Some(WhenConditions {
        language: Some(vec!["python".to_string()]),
        ..Default::default()
    });
    let file_context = FileContext { language: "typescript", ..Default::default() };
    let result = evaluate_template_match(&template, &file_context, &default_env());
    assert!(!result.matched);
    assert!(result.reasons.contains(&"Condition failed: language"));
}
```

### Integration Tests

```rust
#[test]
fn test_context_command() {
    let project = TestProject::new();
    project.write_config(/* config */);
    project.write_template("ts.md", /* typescript template */);
    project.write_template("py.md", /* python template */);

    let output = run_command(&["context", "src/app.ts"], &project.root).unwrap();

    assert!(output.contains("typescript-rules"));
    assert!(output.contains("✓"));
    assert!(output.contains("python-style"));
    assert!(output.contains("✗"));
}

#[test]
fn test_render_section() {
    let project = TestProject::new();
    project.write_config(/* config */);
    project.write_template("ts.md", /* content */);

    let output = run_command(&["render", "--section", "typescript-rules"], &project.root).unwrap();

    assert!(output.contains("TypeScript Rules"));
    assert!(!output.contains("Other Section"));  // Only specified section
}

#[test]
fn test_list_skills() {
    let project = TestProject::new();
    project.write_config_with_skills();
    project.write_template("skill1.md", /* with description */);
    project.write_template("skill2.md", /* with description */);

    let output = run_command(&["list-skills"], &project.root).unwrap();

    assert!(output.contains("skill1"));
    assert!(output.contains("skill2"));
    assert!(output.contains("Total: 2 skills"));
}

#[test]
fn test_cache_performance() {
    let project = TestProject::new_with_many_templates(50);

    // First run (no cache)
    let start = Instant::now();
    run_command(&["render", "test.ts"], &project.root).unwrap();
    let first_duration = start.elapsed();

    // Second run (with cache)
    let start = Instant::now();
    run_command(&["render", "test.ts"], &project.root).unwrap();
    let second_duration = start.elapsed();

    // Cache should be significantly faster
    assert!(second_duration < first_duration / 2);
}
```

### Manual Testing

- [ ] Run `context` command on various files
- [ ] Verify `--json` output is valid JSON
- [ ] Test `render --section` with non-existent section (should error)
- [ ] Verify caching speeds up repeated invocations
- [ ] Test `--verbose` flag shows helpful debug info
- [ ] Verify `list-skills` matches what `build` generates

## Documentation Impact

### Files to Update
- [ ] `COMMANDS.md` - Document all new commands and flags
- [ ] `README.md` - Mention new capabilities
- [ ] `ADVANCED.md` - Document caching mechanism
- [ ] `CHANGELOG.md` - Note CLI enhancements

### Documentation Requirements
- [ ] Examples for each new command
- [ ] JSON schema documentation
- [ ] Performance tips (using cache)

## Alternatives Considered

### Alternative 1: Persistent Daemon

**Description:** Run cAGENTS as a background daemon that keeps templates loaded in memory, responding to queries via IPC.

**Pros:**
- Even faster than caching (no I/O at all)
- Could support watch mode
- More sophisticated IDE integration

**Cons:**
- Much more complex to implement
- Process management challenges
- Platform-specific (harder to support)

**Why not chosen:** Caching provides most of the performance benefit with much less complexity. Can revisit daemon approach in future if needed.

### Alternative 2: Query Language

**Description:** Allow users to query templates with a DSL, e.g., `cagents query "templates where language=typescript and core=true"`.

**Pros:**
- Very flexible
- Powerful for complex queries

**Cons:**
- Requires parser for query language
- Overkill for most use cases
- Steeper learning curve

**Why not chosen:** Simple command structure (`context`, `list-skills`) covers most needs. Query language is unnecessary complexity.

## Success Metrics

- [x] All commands implemented and tested
- [x] JSON output schemas documented
- [x] Caching improves performance (>2x speedup on repeated calls)
- [x] `--verbose` output is helpful for debugging
- [ ] Positive user feedback on IDE integration (post-release)

---

## Status Updates

- **2025-10-29:** Proposal created, status: Draft
