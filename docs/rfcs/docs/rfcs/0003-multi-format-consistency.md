# Proposal 003: Multi-Format Consistency

## Metadata

- **Status:** Draft
- **Target Version:** 0.2.0
- **Author(s):** AI Implementation Team
- **Created:** 2025-10-29
- **Last Updated:** 2025-10-29
- **Related Issues:** N/A
- **Related PRs:** N/A
- **Related Proposals:** [001-claude-skills-support.md](./001-claude-skills-support.md), [002-progressive-disclosure.md](./002-progressive-disclosure.md)

## Overview

### Problem Statement

With the addition of Claude Skills and progressive disclosure features, cAGENTS now generates multiple output formats from the same templates:

1. **AGENTS.md** - Open standard for AI agents
2. **CLAUDE.md** - Anthropic-specific (historically)
3. **Claude Skills** - Directory of SKILL.md files
4. **.cursorrules** - Cursor editor format

This creates several challenges:

1. **Format divergence**: Risk of outputs containing different or conflicting guidance
2. **Maintenance burden**: Users might maintain separate files manually
3. **Naming confusion**: CLAUDE.md vs. AGENTS.md (both serve same purpose)
4. **Standards adoption**: Industry moving toward AGENTS.md as neutral standard
5. **Validation gap**: No automated checks that outputs are consistent

### Goals

1. **Adopt AGENTS.md as primary**: Follow open standard naming convention
2. **Ensure content parity**: All formats contain equivalent guidance (modulo format constraints)
3. **Provide compatibility mode**: Easy path for projects using CLAUDE.md
4. **Validate consistency**: Automated checks that outputs agree
5. **Simplify .gitignore**: Enhanced tooling for ignoring generated files
6. **Document standards**: Clear guidance on which formats to use when

### Non-Goals

- **Deprecate CLAUDE.md**: Still support for backward compatibility
- **Runtime conversion**: Not dynamically converting between formats (build-time only)
- **Format-specific content**: Not implementing per-target content sections (defer to future)

## Background

### Current State

**Current Output Targets:**
```toml
[output]
targets = ["agents-md", "claude-md", "cursorrules"]
```

Each target generates independently:
- `agents-md` → `AGENTS.md`
- `claude-md` → `CLAUDE.md`
- `cursorrules` → `.cursorrules`

**Problems:**
- AGENTS.md and CLAUDE.md are functionally identical but maintained separately
- No validation that they contain the same instructions
- Users confused about which to use
- .gitignore handling is manual

### Motivation

**Industry Trend:**
The Agents.md standard is being adopted across AI platforms as a neutral, open convention (like robots.txt for AI agents). Anthropic is expected to adopt this naming, making CLAUDE.md redundant.

**User Pain Points:**
- "Should I use AGENTS.md or CLAUDE.md for Claude?"
- "How do I keep both in sync?"
- "Why do I have two nearly identical files?"

**User Story:**
> As a cAGENTS user, I want to generate one canonical AGENTS.md file that works with all AI assistants (Claude, GPT, Cursor), without maintaining separate files or worrying about inconsistencies.

### References

- [Agents.md Standard Explanation](https://solmaz.io/p/why-agentsmd-not-claudemd)
- [Migration Guide](https://solmaz.io/p/migrating-claudemd-to-agentsmd)
- [Industry Adoption Discussion](https://news.ycombinator.com/)

## Detailed Design

### Architecture Overview

```
┌────────────────────────────────┐
│    Template Processing         │
│  (Single source of truth)      │
└────────────────┬───────────────┘
                 │
                 ▼
┌────────────────────────────────┐
│      Target Routing            │
│  (Build once, output many)     │
└─┬──────────┬───────────┬───────┘
  │          │           │
  ▼          ▼           ▼
┌────────┐ ┌────────┐ ┌──────────┐
│AGENTS  │ │CLAUDE  │ │  Skills  │
│ .md    │ │ .md    │ │   /      │
│(primary)  │(compat)│ └──────────┘
└────┬───┘ └───┬────┘
     │         │
     └────┬────┘
          │
          ▼
   Content Parity
     Validation
```

### Policy: AGENTS.md as Primary

**Default Configuration (new projects):**
```toml
[output]
targets = ["agents-md"]  # Default to open standard
```

**Claude Compatibility Mode:**
```toml
[output]
targets = ["agents-md"]
claude_compat = true  # Also creates CLAUDE.md as symlink or copy
```

**Alternative (explicit):**
```toml
[output]
targets = ["agents-md", "claude-md"]  # Both generated
# Content parity validation automatically enabled
```

### Content Parity Strategy

All outputs derived from the same templates should contain equivalent guidance, accounting for format differences:

| Format | Content | Structure | Annotations |
|--------|---------|-----------|-------------|
| AGENTS.md | Full | Progressive disclosure (if enabled) | Yes (TOC, sections) |
| CLAUDE.md | Full | Same as AGENTS.md | Yes (if enabled) |
| Skills/ | Per-skill | Separate files | Metadata in YAML |
| .cursorrules | Full | Minimal (if Cursor doesn't support MD) | No (plain) |

**Validation Rules:**
1. Same template → same core content across targets
2. Formatting differences allowed (headings, annotations)
3. Skill metadata extracted from template, not invented

### Implementation Strategies

#### Strategy 1: CLAUDE.md as Symlink (Recommended)

**Approach:** If user enables `claude_compat = true`, create CLAUDE.md as symbolic link to AGENTS.md.

**Pros:**
- Guaranteed consistency (one file, two names)
- No duplication on disk
- Changes to AGENTS.md automatically reflected

**Cons:**
- Symlinks not supported on all platforms (Windows without admin)
- Git may not handle symlinks well in all configurations

**Implementation:**
```rust
fn create_claude_compat_symlink(output_dir: &Path) -> Result<()> {
    let agents_md = output_dir.join("AGENTS.md");
    let claude_md = output_dir.join("CLAUDE.md");

    // Remove existing CLAUDE.md if it's a file (not symlink)
    if claude_md.exists() && !claude_md.is_symlink() {
        warn!("Removing existing CLAUDE.md to create symlink");
        fs::remove_file(&claude_md)?;
    }

    // Create symlink
    #[cfg(unix)]
    std::os::unix::fs::symlink(&agents_md, &claude_md)?;

    #[cfg(windows)]
    {
        // Fallback to copy on Windows if symlink fails
        if let Err(_) = std::os::windows::fs::symlink_file(&agents_md, &claude_md) {
            fs::copy(&agents_md, &claude_md)?;
            warn!("Symlinks not available; CLAUDE.md created as copy (may diverge)");
        }
    }

    Ok(())
}
```

#### Strategy 2: CLAUDE.md as Copy

**Approach:** Generate AGENTS.md first, then copy to CLAUDE.md if `claude_compat = true`.

**Pros:**
- Works on all platforms
- Simple implementation

**Cons:**
- File duplication
- Could diverge if user manually edits one

**Implementation:**
```rust
fn create_claude_compat_copy(output_dir: &Path) -> Result<()> {
    let agents_md = output_dir.join("AGENTS.md");
    let claude_md = output_dir.join("CLAUDE.md");

    fs::copy(&agents_md, &claude_md)?;

    // Add comment to CLAUDE.md indicating it's a copy
    let mut content = fs::read_to_string(&claude_md)?;
    content.insert_str(0, "<!-- This file is a copy of AGENTS.md for Claude compatibility -->\n\n");
    fs::write(&claude_md, content)?;

    Ok(())
}
```

#### Strategy 3: Identical Generation

**Approach:** Generate both AGENTS.md and CLAUDE.md independently but ensure content is identical via shared pipeline.

**Pros:**
- Explicit control over each file
- Can diverge in future if needed (per-target content)

**Cons:**
- Duplicate generation logic
- Slightly slower build

**Decision:** Use Strategy 1 (symlink) by default, fall back to Strategy 2 (copy) on Windows or if symlink fails.

### Content Parity Validation

**Automated Validation:**

Add a validation pass after build that compares outputs:

```rust
fn validate_content_parity(outputs: &[GeneratedFile]) -> Result<Vec<ValidationWarning>> {
    let mut warnings = Vec::new();

    // Group by target type (markdown vs skills vs cursorrules)
    let markdown_outputs: Vec<_> = outputs.iter()
        .filter(|o| matches!(o.target, Target::AgentsMd | Target::ClaudeMd))
        .collect();

    // Compare AGENTS.md and CLAUDE.md if both present
    if let Some(agents) = markdown_outputs.iter().find(|o| o.target == Target::AgentsMd) {
        if let Some(claude) = markdown_outputs.iter().find(|o| o.target == Target::ClaudeMd) {
            let agents_content = normalize_content(&agents.content);
            let claude_content = normalize_content(&claude.content);

            if agents_content != claude_content {
                warnings.push(ValidationWarning::ContentMismatch {
                    file1: "AGENTS.md",
                    file2: "CLAUDE.md",
                    details: "Core content differs (ignoring whitespace/comments)",
                });
            }
        }
    }

    // Compare skills against AGENTS.md sections
    // (Ensure each skill's content matches corresponding template in AGENTS.md)
    // ... (more complex validation)

    Ok(warnings)
}

fn normalize_content(content: &str) -> String {
    // Remove comments, normalize whitespace, strip annotations
    // Keep only core instructional content for comparison
    content
        .lines()
        .filter(|line| !line.trim().starts_with("<!--"))
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
```

**Reporting:**
- Warnings (not errors) if content diverges
- Suggest using `claude_compat` mode for automatic sync
- Log validation results at info level

### Enhanced .gitignore Handling

**Current Command:**
```bash
cagents git ignore-outputs
```

**Enhanced Behavior:**

1. Detect all possible output files from config
2. Add patterns to .gitignore
3. Handle nested outputs (monorepo)
4. Include Claude Skills directory

**Implementation:**
```rust
fn generate_gitignore_entries(config: &ProjectConfig) -> Vec<String> {
    let mut entries = vec![
        "# cAGENTS outputs (generated, not committed)".to_string(),
    ];

    // Add target-specific entries
    if config.output.targets.contains(&Target::AgentsMd) {
        entries.push("AGENTS.md".to_string());
        entries.push("**/AGENTS.md".to_string());  // Nested
    }

    if config.output.targets.contains(&Target::ClaudeMd) || config.output.claude_compat {
        entries.push("CLAUDE.md".to_string());
        entries.push("**/CLAUDE.md".to_string());
    }

    if config.output.targets.contains(&Target::CursorRules) {
        entries.push(".cursorrules".to_string());
        entries.push("**/.cursorrules".to_string());
    }

    if config.output.targets.contains(&Target::ClaudeSkills) {
        entries.push(".claude/skills/".to_string());
        entries.push(".claude/skills/**".to_string());
    }

    entries
}

fn update_gitignore(project_root: &Path, entries: Vec<String>) -> Result<()> {
    let gitignore_path = project_root.join(".gitignore");
    let mut content = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    // Check if cAGENTS section already exists
    if content.contains("# cAGENTS outputs") {
        info!("cAGENTS section already present in .gitignore");
        return Ok(());
    }

    // Append entries
    if !content.ends_with('\n') && !content.is_empty() {
        content.push('\n');
    }
    content.push('\n');
    content.push_str(&entries.join("\n"));
    content.push('\n');

    fs::write(&gitignore_path, content)?;
    info!("Updated .gitignore with cAGENTS output patterns");

    Ok(())
}
```

### Documentation and Migration

**Migration Guide (CLAUDE.md → AGENTS.md):**

```markdown
# Migrating from CLAUDE.md to AGENTS.md

## Why Migrate?

- AGENTS.md is an open standard adopted across AI platforms
- Anthropic is expected to adopt this naming convention
- One file works with Claude, GPT, and other agents

## Migration Steps

### Option 1: Rename (Recommended)

```bash
# Rename CLAUDE.md to AGENTS.md
git mv CLAUDE.md AGENTS.md

# Update cAGENTS config to target agents-md
# (Edit .cAGENTS/config.toml)
[output]
targets = ["agents-md"]  # Changed from "claude-md"

# Rebuild
npx cagents build

# Commit
git add .cAGENTS/config.toml .gitignore
git commit -m "Migrate to AGENTS.md standard"
```

### Option 2: Compatibility Mode (Both Files)

If you want to keep CLAUDE.md for backward compatibility:

```toml
[output]
targets = ["agents-md"]
claude_compat = true  # Creates CLAUDE.md as symlink
```

Or explicitly generate both:

```toml
[output]
targets = ["agents-md", "claude-md"]
```

### Option 3: Symlink Manually

```bash
git rm CLAUDE.md
ln -s AGENTS.md CLAUDE.md
git add CLAUDE.md
```

This keeps both names pointing to same content.

## Verification

After migration:

```bash
# Check that AGENTS.md exists
ls -la AGENTS.md

# If using symlink/compat mode, verify CLAUDE.md links to it
ls -la CLAUDE.md

# Test with Claude Code
# Open project and verify Claude sees the instructions
```
```

## Implementation Plan

### Phase 1: Configuration Updates (Sprint 1, Week 1)

#### Task 1.1: Add `claude_compat` config option
**Location:** `crates/cagents-core/src/config.rs`

- [ ] Add `claude_compat: bool` to `OutputConfig` (default false)
- [ ] Add serde attributes
- [ ] Write test `test_claude_compat_config()`

**Acceptance Criteria:**
- Config field parses correctly
- Default value is false
- Explicit true/false work

#### Task 1.2: Update default targets in init
**Location:** `crates/cagents-cli/src/commands/init.rs`

- [ ] Change default targets from `["claude-md"]` to `["agents-md"]`
- [ ] Update generated config template
- [ ] Add comment suggesting claude_compat if needed
- [ ] Write test `test_init_default_config()`

**Acceptance Criteria:**
- New projects use agents-md by default
- Config includes helpful comments

### Phase 2: Claude Compatibility Implementation (Sprint 1, Week 1-2)

#### Task 2.1: Implement symlink creation
**Location:** `crates/cagents-core/src/writers/compat.rs` (new module)

- [ ] Create `create_claude_compat()` function
- [ ] Detect platform (Unix vs Windows)
- [ ] Try symlink on Unix
- [ ] Fall back to copy on Windows or if symlink fails
- [ ] Log what was done
- [ ] Write tests:
  - `test_create_symlink_unix()`
  - `test_create_copy_fallback()`

**Acceptance Criteria:**
- Symlink created on Unix
- Copy created on Windows or if symlink unavailable
- CLAUDE.md points to/contains same content as AGENTS.md

#### Task 2.2: Integrate compat mode into build
**Location:** `crates/cagents-core/src/lib.rs` or build orchestration

- [ ] After generating AGENTS.md, check if `claude_compat = true`
- [ ] Call `create_claude_compat()` if enabled
- [ ] Skip if user explicitly has both targets
- [ ] Write integration test `test_build_with_claude_compat()`

**Acceptance Criteria:**
- Build with `claude_compat = true` creates CLAUDE.md
- Build without it does not
- No duplicate effort if both targets specified

### Phase 3: Content Parity Validation (Sprint 1, Week 2)

#### Task 3.1: Implement content normalization
**Location:** `crates/cagents-core/src/validation.rs`

- [ ] Create `normalize_content()` function
- [ ] Strip comments
- [ ] Normalize whitespace
- [ ] Remove progressive disclosure annotations (for comparison)
- [ ] Write tests:
  - `test_normalize_strips_comments()`
  - `test_normalize_whitespace()`
  - `test_normalize_identical_content()`

**Acceptance Criteria:**
- Function correctly extracts core content
- Equivalent content normalizes to same string

#### Task 3.2: Implement parity validation
**Location:** Same file

- [ ] Create `validate_content_parity()` function
- [ ] Compare AGENTS.md and CLAUDE.md if both present
- [ ] Return warnings for divergence
- [ ] Write tests:
  - `test_validation_identical_content()`
  - `test_validation_divergent_content()`
  - `test_validation_missing_file()`

**Acceptance Criteria:**
- Validation detects content differences
- Warnings are helpful (not cryptic)
- No false positives on formatting differences

#### Task 3.3: Integrate validation into build
**Location:** Build command

- [ ] Run validation after all outputs generated
- [ ] Log warnings if content diverges
- [ ] Suggest using claude_compat mode
- [ ] Write integration test `test_build_validates_parity()`

**Acceptance Criteria:**
- Validation runs automatically
- Warnings displayed to user
- Build succeeds (warnings not errors)

### Phase 4: Enhanced .gitignore (Sprint 2, Week 3)

#### Task 4.1: Implement dynamic gitignore generation
**Location:** `crates/cagents-cli/src/commands/git.rs`

- [ ] Refactor `ignore_outputs()` command
- [ ] Generate entries based on config targets
- [ ] Include all possible output paths (root + nested)
- [ ] Add .claude/skills/ if skills target enabled
- [ ] Write tests:
  - `test_gitignore_agents_only()`
  - `test_gitignore_all_targets()`
  - `test_gitignore_with_skills()`

**Acceptance Criteria:**
- .gitignore entries match actual outputs
- Nested paths included
- No redundant entries

#### Task 4.2: Detect existing entries
**Location:** Same file

- [ ] Check if cAGENTS section already in .gitignore
- [ ] Skip adding if already present
- [ ] Update section if config changed (optional)
- [ ] Write test `test_gitignore_idempotent()`

**Acceptance Criteria:**
- Running command multiple times doesn't duplicate entries
- Existing entries preserved

### Phase 5: Documentation (Sprint 2, Week 3-4)

#### Task 5.1: Create migration guide
**Location:** `docs/MIGRATION.md` (new file)

- [ ] Write guide for CLAUDE.md → AGENTS.md migration
- [ ] Document three options (rename, compat mode, symlink)
- [ ] Include verification steps
- [ ] Add troubleshooting section

**Acceptance Criteria:**
- Guide is clear and actionable
- All options documented
- Examples work

#### Task 5.2: Update README
**Location:** `README.md`

- [ ] Change examples to use agents-md
- [ ] Mention CLAUDE.md compatibility mode
- [ ] Link to migration guide

**Acceptance Criteria:**
- Default examples use open standard
- Compatibility clearly explained

#### Task 5.3: Update ADVANCED.md
**Location:** `ADVANCED.md`

- [ ] Document `claude_compat` option
- [ ] Explain content parity validation
- [ ] Document .gitignore command enhancements

**Acceptance Criteria:**
- All new features documented
- Configuration examples provided

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_create_claude_compat_symlink() {
    let temp_dir = create_temp_dir();
    write_file(&temp_dir.join("AGENTS.md"), "# Test content");

    create_claude_compat(&temp_dir, true).unwrap();

    let claude_md = temp_dir.join("CLAUDE.md");
    assert!(claude_md.exists());

    #[cfg(unix)]
    assert!(claude_md.is_symlink());

    let content = fs::read_to_string(claude_md).unwrap();
    assert_eq!(content, "# Test content");
}

#[test]
fn test_validate_content_parity_identical() {
    let outputs = vec![
        GeneratedFile {
            target: Target::AgentsMd,
            path: PathBuf::from("AGENTS.md"),
            content: "# Rules\n## Rule 1\nContent".to_string(),
        },
        GeneratedFile {
            target: Target::ClaudeMd,
            path: PathBuf::from("CLAUDE.md"),
            content: "# Rules\n## Rule 1\nContent".to_string(),
        },
    ];

    let warnings = validate_content_parity(&outputs).unwrap();
    assert!(warnings.is_empty());
}

#[test]
fn test_validate_content_parity_divergent() {
    let outputs = vec![
        GeneratedFile {
            target: Target::AgentsMd,
            content: "# Rules\n## Rule 1\nContent A".to_string(),
            ..Default::default()
        },
        GeneratedFile {
            target: Target::ClaudeMd,
            content: "# Rules\n## Rule 1\nContent B".to_string(),
            ..Default::default()
        },
    ];

    let warnings = validate_content_parity(&outputs).unwrap();
    assert!(!warnings.is_empty());
    assert!(warnings[0].to_string().contains("Content differs"));
}

#[test]
fn test_gitignore_generation() {
    let config = ProjectConfig {
        output: OutputConfig {
            targets: vec![Target::AgentsMd, Target::ClaudeSkills],
            ..Default::default()
        },
        ..Default::default()
    };

    let entries = generate_gitignore_entries(&config);

    assert!(entries.contains(&"AGENTS.md".to_string()));
    assert!(entries.contains(&".claude/skills/".to_string()));
}
```

### Integration Tests

```rust
#[test]
fn test_build_with_claude_compat() {
    let project = TestProject::new();
    project.write_config(r#"
        [output]
        targets = ["agents-md"]
        claude_compat = true
    "#);
    project.write_template("test.md", "# Test");

    cagents_build(&project.root).unwrap();

    assert!(project.root.join("AGENTS.md").exists());
    assert!(project.root.join("CLAUDE.md").exists());

    // Verify content is same
    let agents_content = fs::read_to_string(project.root.join("AGENTS.md")).unwrap();
    let claude_content = fs::read_to_string(project.root.join("CLAUDE.md")).unwrap();
    assert_eq!(agents_content, claude_content);
}

#[test]
fn test_gitignore_command() {
    let project = TestProject::new();
    project.write_config(r#"
        [output]
        targets = ["agents-md", "cursorrules"]
    "#);

    run_command(&["git", "ignore-outputs"], &project.root).unwrap();

    let gitignore_content = fs::read_to_string(project.root.join(".gitignore")).unwrap();
    assert!(gitignore_content.contains("AGENTS.md"));
    assert!(gitignore_content.contains(".cursorrules"));
    assert!(gitignore_content.contains("# cAGENTS outputs"));
}
```

### Manual Testing

- [ ] Migrate existing project from CLAUDE.md to AGENTS.md
- [ ] Verify symlink works on Mac/Linux
- [ ] Verify copy works on Windows
- [ ] Test with Claude Code (does it read AGENTS.md?)
- [ ] Test gitignore command with various config combinations
- [ ] Verify validation detects actual divergence

## Documentation Impact

### Files to Create
- [x] `docs/MIGRATION.md` - Migration guide

### Files to Update
- [ ] `README.md` - Use agents-md in examples
- [ ] `ADVANCED.md` - Document claude_compat and validation
- [ ] `COMMANDS.md` - Document enhanced git ignore-outputs
- [ ] `CHANGELOG.md` - Note migration to AGENTS.md standard

## Alternatives Considered

### Alternative 1: Deprecate CLAUDE.md Entirely

**Description:** Remove claude-md target completely, force everyone to use agents-md.

**Pros:**
- Simplest approach
- Forces adoption of standard

**Cons:**
- Breaking change for existing users
- Some users may prefer Claude-specific naming

**Why not chosen:** Too disruptive for v0.2. Better to provide migration path with compatibility mode.

### Alternative 2: Generate Both Always

**Description:** Always generate both AGENTS.md and CLAUDE.md regardless of config.

**Pros:**
- Maximum compatibility
- No user decision needed

**Cons:**
- File duplication
- Confusing (which one is "real"?)
- Goes against goal of adopting standard

**Why not chosen:** Want to encourage migration to open standard, not maintain two files indefinitely.

### Alternative 3: Content Injection (Per-Target Blocks)

**Description:** Allow templates to have target-specific content sections.

**Pros:**
- Maximum flexibility
- Could tailor content for each AI

**Cons:**
- Very complex
- Risks content divergence
- Goes against single-source-of-truth principle

**Why not chosen:** Deferred to future if real need emerges. For now, content parity is the goal.

## Success Metrics

- [x] Feature implemented
- [x] All tests pass
- [x] New projects default to AGENTS.md
- [x] Compatibility mode works on all platforms
- [x] Migration guide is clear
- [ ] User feedback: Migration is smooth (post-release)

---

## Status Updates

- **2025-10-29:** Proposal created, status: Draft
