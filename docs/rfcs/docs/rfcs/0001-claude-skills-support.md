# Proposal 001: Claude Skills Support

## Metadata

- **Status:** Draft
- **Target Version:** 0.2.0
- **Author(s):** AI Implementation Team
- **Created:** 2025-10-29
- **Last Updated:** 2025-10-29
- **Related Issues:** N/A (new feature)
- **Related PRs:** N/A

## Overview

### Problem Statement

cAGENTS currently generates static instruction files (AGENTS.md, CLAUDE.md) that are fully loaded into AI agents' context at session start. For complex projects, this results in:

1. **Context overload**: 300+ line instruction files overwhelm the model with potentially irrelevant information
2. **Inefficient token usage**: All instructions consume context regardless of actual relevance to the current task
3. **No dynamic loading**: Instructions can't be loaded on-demand based on what the agent is working on
4. **Missed opportunities**: Anthropic's new Claude Skills feature enables progressive disclosure, but cAGENTS doesn't support it

### Goals

1. **Enable Claude Skills generation** from cAGENTS templates as a new output target
2. **Maintain single source of truth**: Same templates generate both static files and skills
3. **Support conditional skills**: Only generate skills when relevant (based on `when` conditions)
4. **Preserve backwards compatibility**: Existing workflows continue working unchanged
5. **Facilitate progressive disclosure**: Claude loads instructions only when needed

### Non-Goals

- **Multi-file skill assets** (reference docs, scripts): Deferred to future version
- **Runtime skill loading**: Build-time only for this iteration
- **Skill grouping**: One-template-one-skill for v1 (grouping deferred)
- **Non-Claude skill formats**: Focus on Anthropic's Claude Skills only

## Background

### Current State

**cAGENTS Architecture:**
```
Templates (.cAGENTS/templates/*.md)
    ↓
  [Config + Conditions Evaluation]
    ↓
  [Rendering Engine]
    ↓
  Outputs:
    - AGENTS.md
    - CLAUDE.md
    - .cursorrules
```

**Limitations:**
- All outputs are single files (or nested single files)
- Content is static once generated
- No mechanism for on-demand loading

### Motivation

**Anthropic's Claude Skills** (introduced late 2024/early 2025) enable:

1. **Dynamic context loading**: Claude scans skill metadata (name + description) and loads full content only when relevant
2. **Reduced context pollution**: Irrelevant skills never enter the context window
3. **Better specialization**: Each skill is a focused bundle of expertise
4. **Automatic discovery**: Claude Code auto-loads skills from `.claude/skills/` directory

**User Story:**
> As a cAGENTS user working on a large monorepo, I want Claude to only see TypeScript rules when working on TS files, and database migration rules when working on migrations—not a giant CLAUDE.md with everything mixed together.

### References

- [Anthropic Skills Documentation](https://docs.claude.com/)
- [Claude Skills Developer Guide](https://leehanchung.github.io/claude-skills-guide/)
- [AgentBuild Newsletter on Skills](https://newsletter.agentbuild.ai/)
- [cAGENTS Original Discussion](https://news.ycombinator.com/item?id=42465839)

## Detailed Design

### Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│              Template Loading                       │
│  (.cAGENTS/templates/*.md + config.toml)           │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│         Condition Evaluation (Planner)              │
│  (Evaluate `when` clauses per environment)          │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│             Target Routing (Build)                  │
│  (Route to appropriate writer per target)           │
└───┬─────────────┬──────────────┬────────────────────┘
    │             │              │
    ▼             ▼              ▼
┌────────┐   ┌─────────┐   ┌──────────────────┐
│ Agents │   │ Claude  │   │  ClaudeSkills    │  ← NEW
│  .md   │   │  .md    │   │  Writer          │
└────────┘   └─────────┘   └────────┬─────────┘
                                     │
                                     ▼
                            .claude/skills/
                              ├─ skill-1/
                              │   └─ SKILL.md
                              ├─ skill-2/
                              │   └─ SKILL.md
                              └─ skill-n/
                                  └─ SKILL.md
```

### Data Structures

#### Extended RuleFrontmatter

**Location:** `crates/cagents-core/src/model.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFrontmatter {
    // Existing fields
    pub name: String,
    pub order: Option<i32>,
    pub when: Option<WhenConditions>,
    pub globs: Option<Vec<String>>,
    pub targets: Option<Vec<String>>,
    pub extends: Option<String>,
    pub merge: Option<String>,
    pub output_in: Option<String>,

    // New fields for Claude Skills
    /// One-line description for skill matching (required for claude-skills target)
    pub description: Option<String>,

    /// Override auto-generated skill folder name
    pub skill_name: Option<String>,

    /// Tools this skill is allowed to use (default from config)
    pub allowed_tools: Option<Vec<String>>,

    /// Skill version (defaults to project version)
    pub skill_version: Option<String>,
}
```

#### SkillDefaults Config

**Location:** `crates/cagents-core/src/config.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefaults {
    /// Output directory for skills (default: ".claude/skills")
    #[serde(default = "default_skill_output_dir")]
    pub output_dir: PathBuf,

    /// Default allowed tools for all skills
    #[serde(default = "default_allowed_tools")]
    pub allowed_tools: Vec<String>,

    /// Skill version (defaults to project version if available)
    pub version: Option<String>,

    /// License reference (optional)
    pub license: Option<String>,
}

fn default_skill_output_dir() -> PathBuf {
    PathBuf::from(".claude/skills")
}

fn default_allowed_tools() -> Vec<String> {
    vec!["Bash".to_string(), "Read".to_string(), "Write".to_string(), "Edit".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    // Existing fields...

    /// Skill-specific configuration
    #[serde(default)]
    pub skill: SkillDefaults,
}
```

#### SKILL.md Structure

**Format:**
```markdown
---
name: typescript-rules
description: TypeScript coding conventions and best practices for this project
version: 0.1.0
allowed_tools:
  - Bash
  - Read
  - Write
---

# TypeScript Rules

Use strict mode. Prefer functional components...

(rest of template content)
```

### Algorithm / Logic

#### Skill Generation Flow

**Pseudocode:**

```
function build_claude_skills(config, templates):
    skill_writer = ClaudeSkillsWriter::new(config.skill)
    skills_to_generate = []

    for template in templates:
        # Evaluate conditions
        if not evaluate_conditions(template.when, environment):
            log("Skipping skill '{}' (condition not met)", template.name)
            continue

        # Validate required fields
        if not template.description:
            error("Template '{}' requires 'description' field for claude-skills target", template.name)

        # Determine skill name
        skill_name = template.skill_name or slugify(template.name)

        # Check for name collisions
        if skill_name in skills_to_generate:
            error("Duplicate skill name '{}' from templates '{}' and '{}'", skill_name, ...)

        skills_to_generate.push((skill_name, template))

    # Generate skill directories
    ensure_dir_exists(config.skill.output_dir)

    for (skill_name, template) in skills_to_generate:
        skill_dir = config.skill.output_dir.join(skill_name)
        ensure_dir_exists(skill_dir)

        skill_md = generate_skill_md(template, config.skill)
        write_file(skill_dir.join("SKILL.md"), skill_md)

    # Clean up stale skills (optional, behind flag)
    if config.clean_stale_skills:
        cleanup_stale_skills(config.skill.output_dir, skills_to_generate)
```

#### SKILL.md Generation

```rust
fn generate_skill_md(template: &Template, defaults: &SkillDefaults) -> String {
    // Generate YAML frontmatter
    let frontmatter = SkillFrontmatter {
        name: template.skill_name.clone()
            .unwrap_or_else(|| slugify(&template.name)),
        description: template.description.clone()
            .expect("description required for skills"),
        version: template.skill_version.clone()
            .or(defaults.version.clone())
            .unwrap_or_else(|| "0.1.0".to_string()),
        allowed_tools: template.allowed_tools.clone()
            .unwrap_or_else(|| defaults.allowed_tools.clone()),
        license: defaults.license.clone(),
    };

    let yaml = serde_yaml::to_string(&frontmatter)?;
    let content = template.rendered_content(); // Body without template's own frontmatter

    format!("---\n{}---\n\n{}", yaml, content)
}

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
        .trim_matches('-')
        .to_string()
}
```

### API / Interface Changes

#### CLI

**No new commands** (uses existing `cagents build`)

**Config Change:**
```toml
[output]
targets = ["agents-md", "claude-skills"]  # Add "claude-skills" target

[skill]
output_dir = ".claude/skills"
allowed_tools = ["Bash", "Read", "Write", "Edit"]
version = "0.1.0"
# license = "MIT"  # optional
```

**Template Change:**
```markdown
---
name: typescript-rules
description: "TypeScript coding conventions and best practices"  # ← Required for skills
skill_name: ts-conventions  # ← Optional override
allowed_tools: ["Bash", "Read"]  # ← Optional override
---
```

#### Rust API

**New Module:** `crates/cagents-core/src/writers/claude_skills.rs`

```rust
pub struct ClaudeSkillsWriter {
    output_dir: PathBuf,
    defaults: SkillDefaults,
}

impl ClaudeSkillsWriter {
    pub fn new(output_dir: PathBuf, defaults: SkillDefaults) -> Self { }

    /// Generate all skills from templates
    pub fn write_skills(&self, templates: Vec<&Template>) -> Result<Vec<PathBuf>> { }

    /// Generate SKILL.md content for a single template
    fn generate_skill_md(&self, template: &Template) -> Result<String> { }

    /// Convert template name to skill folder name
    fn slugify(&self, name: &str) -> String { }
}
```

### File Structure Changes

```
crates/cagents-core/src/
  writers/
    mod.rs                 # Add `pub mod claude_skills;`
    claude_skills.rs       # NEW: ClaudeSkillsWriter implementation
  model.rs                 # Extend RuleFrontmatter
  config.rs                # Add SkillDefaults
  target.rs                # Add Target::ClaudeSkills enum variant

.gitignore                 # Add .claude/skills/ (via `cagents git ignore-outputs`)

examples/
  skills-demo/             # NEW: Example project with skills
    .cAGENTS/
      templates/
        typescript.md
        testing.md
      config.toml
    README.md
```

### Error Handling

**Errors:**

1. **Missing description for skills target**
   - **Detection**: Template has no `description` field but `claude-skills` in targets
   - **Message**: `Template 'X' requires 'description' field for claude-skills target. Add: description: "Brief summary of this skill"`
   - **Recovery**: Build fails (required field)

2. **Skill name collision**
   - **Detection**: Two templates produce same `skill_name` slug
   - **Message**: `Duplicate skill name 'X' from templates 'A' and 'B'. Use 'skill_name' field to disambiguate.`
   - **Recovery**: Build fails (ambiguity)

3. **SKILL.md too large**
   - **Detection**: Rendered content > 5000 tokens (approx)
   - **Message**: `Warning: Skill 'X' is very large (~Y tokens). Consider splitting or using resource files (future feature).`
   - **Recovery**: Continues with warning (not fatal)

4. **Invalid output directory**
   - **Detection**: `skill.output_dir` points outside project
   - **Message**: `Skill output directory must be within project: got 'X'`
   - **Recovery**: Build fails (security)

**Logging:**
- Info: `Generating skill 'X' from template 'Y'`
- Debug: `Skill 'X' allowed tools: [Bash, Read]`
- Warn: `Skipping skill 'X' (condition not met: use_beads=false)`

### Performance Considerations

**Build Time:**
- Generating 50 skills: ~100ms additional (mostly file I/O)
- Parallel file writing: Possible optimization if needed

**Memory:**
- Each skill holds metadata + content in memory during build
- Expected: < 10MB for typical projects (50 skills × 200KB average)

**Disk Space:**
- One directory per skill + one SKILL.md file
- Expected: < 1MB for typical projects

### Security Considerations

1. **Output directory validation**: Ensure `.claude/skills/` is within project root
2. **Filename sanitization**: Slugify prevents directory traversal (`../../etc`)
3. **No command execution in metadata**: YAML frontmatter is data only
4. **Skill content is user-controlled**: Same trust model as existing template system

## Implementation Plan

### Phase 1: Foundation (Sprint 1, Week 1)

#### Task 1.1: Extend data structures
**Location:** `crates/cagents-core/src/model.rs`

- [ ] Add `description`, `skill_name`, `allowed_tools`, `skill_version` to `RuleFrontmatter`
- [ ] Add serde attributes with proper defaults
- [ ] Write unit test `test_rule_frontmatter_with_skill_fields()`

**Acceptance Criteria:**
- Fields deserialize from YAML without errors
- Optional fields default to `None`
- Test parses both old and new frontmatter formats

#### Task 1.2: Add skill configuration
**Location:** `crates/cagents-core/src/config.rs`

- [ ] Define `SkillDefaults` struct
- [ ] Add `skill: SkillDefaults` to `ProjectConfig`
- [ ] Implement default functions (`default_skill_output_dir`, etc.)
- [ ] Write test `test_config_skill_defaults()`

**Acceptance Criteria:**
- Config without `[skill]` section uses defaults
- Config with `[skill]` section overrides defaults
- Test validates default values

#### Task 1.3: Add Target::ClaudeSkills enum
**Location:** `crates/cagents-core/src/target.rs` (or wherever Target is defined)

- [ ] Add `ClaudeSkills` variant to `Target` enum
- [ ] Implement `FromStr` to parse `"claude-skills"` from config
- [ ] Update `Display` implementation
- [ ] Write test `test_target_parse_claude_skills()`

**Acceptance Criteria:**
- `"claude-skills".parse::<Target>()` succeeds
- Round-trip test: parse → display → parse

### Phase 2: Core ClaudeSkillsWriter (Sprint 1, Week 1-2)

#### Task 2.1: Create ClaudeSkillsWriter module
**Location:** `crates/cagents-core/src/writers/claude_skills.rs`

- [ ] Create file and module declaration
- [ ] Define `ClaudeSkillsWriter` struct
- [ ] Implement `new()` constructor
- [ ] Add to `writers/mod.rs`

**Acceptance Criteria:**
- Module compiles
- Can instantiate writer with config

#### Task 2.2: Implement slugify function
**Location:** Same file

- [ ] Write `slugify(&str) -> String` function
- [ ] Handle: spaces, special chars, leading/trailing hyphens
- [ ] Write tests:
  - `test_slugify_simple()` - "TypeScript Rules" → "typescript-rules"
  - `test_slugify_special_chars()` - "API / REST" → "api-rest"
  - `test_slugify_leading_trailing()` - " -test- " → "test"

**Acceptance Criteria:**
- All test cases pass
- Function is pure (no side effects)

#### Task 2.3: Implement generate_skill_md()
**Location:** Same file

- [ ] Create function signature: `fn generate_skill_md(&self, template: &Template) -> Result<String>`
- [ ] Build YAML frontmatter struct (name, description, version, tools, license)
- [ ] Serialize frontmatter with `serde_yaml`
- [ ] Extract template body (rendered content without its own frontmatter)
- [ ] Concatenate: `---\n{yaml}---\n\n{content}`
- [ ] Write tests:
  - `test_generate_skill_md_minimal()` - Required fields only
  - `test_generate_skill_md_full()` - All fields present
  - `test_generate_skill_md_yaml_valid()` - Parse generated YAML to verify

**Acceptance Criteria:**
- Generated YAML is valid (can be parsed by `serde_yaml::from_str`)
- Content section contains template body
- Frontmatter delimiters (`---`) are correct

#### Task 2.4: Implement write_skills()
**Location:** Same file

- [ ] Iterate through templates
- [ ] For each: determine skill name (use `skill_name` or slugify `name`)
- [ ] Check for name collisions (track seen names)
- [ ] Create skill directory: `output_dir/skill_name/`
- [ ] Write `SKILL.md` file
- [ ] Return list of generated paths
- [ ] Write tests:
  - `test_write_skills_creates_directories()` - Check dirs exist
  - `test_write_skills_name_collision()` - Error on duplicate names
  - `test_write_skills_returns_paths()` - Verify return value

**Acceptance Criteria:**
- Creates one directory per skill
- Each directory contains `SKILL.md`
- Returns correct paths
- Handles errors gracefully

### Phase 3: Integration with Build Pipeline (Sprint 1, Week 2)

#### Task 3.1: Add skills routing to build command
**Location:** `crates/cagents-core/src/lib.rs` (or build orchestration module)

- [ ] Detect `Target::ClaudeSkills` in targets list
- [ ] Instantiate `ClaudeSkillsWriter` with config
- [ ] Filter templates by conditions (reuse planner logic)
- [ ] Call `writer.write_skills(templates)`
- [ ] Log results
- [ ] Write integration test `test_build_with_skills_target()`

**Acceptance Criteria:**
- Build with `targets = ["claude-skills"]` generates skills
- Build with `targets = ["agents-md", "claude-skills"]` generates both
- Build without skills target doesn't call ClaudeSkillsWriter

#### Task 3.2: Validate template requirements
**Location:** Build orchestration or planner

- [ ] Before generating skills, check each template has `description` field
- [ ] If missing, return `Err` with helpful message
- [ ] Write test `test_skills_require_description()`

**Acceptance Criteria:**
- Build fails with clear error if description missing
- Error message suggests fix: "Add: description: \"...\""

#### Task 3.3: Implement condition-based skill filtering
**Location:** Planner or build module

- [ ] For each template, evaluate `when` conditions
- [ ] If conditions fail, skip skill generation for that template
- [ ] Log skipped skills at debug level
- [ ] Write test `test_conditional_skill_skipped()`

**Acceptance Criteria:**
- Template with `when.use_beads = "true"` skipped if `use_beads = "false"`
- Skipped skills don't create directories
- Log message indicates reason for skipping

### Phase 4: Validation and Cleanup (Sprint 2, Week 3)

#### Task 4.1: Implement size warning
**Location:** `ClaudeSkillsWriter::generate_skill_md()`

- [ ] Estimate token count (rough: chars / 4)
- [ ] If > 5000 tokens, log warning
- [ ] Write test `test_large_skill_warning()`

**Acceptance Criteria:**
- Warning logged for large skills
- Build continues (not fatal)

#### Task 4.2: Add stale skill cleanup (optional)
**Location:** `ClaudeSkillsWriter` or build command

- [ ] Track generated skill names during build
- [ ] After writing, scan `.claude/skills/` directory
- [ ] Remove directories not in generated list (optional, behind flag or always-on)
- [ ] Add safety: only remove dirs with cAGENTS marker comment in SKILL.md
- [ ] Write test `test_cleanup_stale_skills()`

**Acceptance Criteria:**
- Old skill directories are removed
- User-created skills (no marker) are preserved
- Logged: "Removed stale skill 'old-skill'"

#### Task 4.3: Update .gitignore command
**Location:** `cagents git ignore-outputs` command

- [ ] Add `.claude/skills/` to generated .gitignore entries
- [ ] Add `.claude/skills/**` pattern
- [ ] Write test `test_gitignore_includes_skills()`

**Acceptance Criteria:**
- Running command adds skill directory to .gitignore
- Pattern matches all nested files

### Phase 5: Documentation and Examples (Sprint 2, Week 3-4)

#### Task 5.1: Create skills-demo example
**Location:** `examples/skills-demo/`

- [ ] Create directory structure
- [ ] Add 3-4 sample templates (typescript, testing, database, tooling)
- [ ] Add config with `targets = ["claude-skills"]`
- [ ] Add README explaining the demo
- [ ] Test: run `cagents build` and verify skills generated

**Acceptance Criteria:**
- Example builds without errors
- README clearly explains purpose
- Demonstrates various skill configurations

#### Task 5.2: Update README.md
**Location:** Root `README.md`

- [ ] Add "Claude Skills Support" section
- [ ] Explain what skills are and benefits
- [ ] Show config snippet enabling skills
- [ ] Include before/after comparison
- [ ] Link to Anthropic documentation

**Acceptance Criteria:**
- Section is clear and concise
- Code examples work

#### Task 5.3: Document new fields in ADVANCED.md
**Location:** `ADVANCED.md`

- [ ] Document `description` frontmatter field
- [ ] Document `skill_name` override
- [ ] Document `allowed_tools` field
- [ ] Document `[skill]` config section
- [ ] Provide examples

**Acceptance Criteria:**
- All new fields documented
- Examples are correct and tested

#### Task 5.4: Create SKILLS.md guide
**Location:** `docs/SKILLS.md` (new file)

- [ ] Introduction to Claude Skills concept
- [ ] How cAGENTS generates skills
- [ ] Best practices for skill descriptions
- [ ] Troubleshooting: "My skill isn't triggering"
- [ ] Link to Anthropic docs

**Acceptance Criteria:**
- Comprehensive guide for users
- Addresses common questions

## Testing Strategy

### Unit Tests

**Location:** `crates/cagents-core/src/writers/claude_skills.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_simple() {
        assert_eq!(slugify("TypeScript Rules"), "typescript-rules");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("API / REST"), "api-rest");
    }

    #[test]
    fn test_generate_skill_md_minimal() {
        let template = create_test_template(/* minimal fields */);
        let writer = ClaudeSkillsWriter::new(/* defaults */);
        let result = writer.generate_skill_md(&template).unwrap();

        // Assert YAML frontmatter exists
        assert!(result.starts_with("---\n"));
        // Assert has description
        assert!(result.contains("description:"));
        // Parse YAML to validate structure
        let parts: Vec<&str> = result.split("---").collect();
        let yaml = parts[1];
        let _parsed: HashMap<String, Value> = serde_yaml::from_str(yaml).unwrap();
    }

    #[test]
    fn test_write_skills_creates_directories() {
        let temp_dir = create_temp_dir();
        let writer = ClaudeSkillsWriter::new(temp_dir.clone(), defaults());
        let templates = vec![create_test_template()];

        writer.write_skills(&templates).unwrap();

        assert!(temp_dir.join("test-skill/SKILL.md").exists());
    }

    #[test]
    fn test_write_skills_name_collision() {
        let templates = vec![
            create_test_template_named("Test Skill"),
            create_test_template_named("Test-Skill"),  // Same slug
        ];

        let result = writer.write_skills(&templates);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate skill name"));
    }
}
```

**Required Tests:**
- [x] Slugify edge cases (spaces, special chars, unicode)
- [x] YAML generation (minimal, full, validation)
- [x] Directory creation
- [x] Name collision detection
- [x] Size warning threshold
- [x] Conditional template filtering

### Integration Tests

**Location:** `crates/cagents-core/tests/claude_skills_test.rs`

```rust
#[test]
fn test_build_with_skills_target() {
    let project = TestProject::new();
    project.write_config(r#"
        [output]
        targets = ["claude-skills"]
    "#);
    project.write_template("test.md", r#"
        ---
        name: test-skill
        description: "Test skill"
        ---
        # Test Content
    "#);

    let result = cagents_build(&project.root);
    assert!(result.is_ok());

    let skill_file = project.root.join(".claude/skills/test-skill/SKILL.md");
    assert!(skill_file.exists());

    let content = fs::read_to_string(skill_file).unwrap();
    assert!(content.contains("name: test-skill"));
    assert!(content.contains("description: Test skill"));
    assert!(content.contains("# Test Content"));
}

#[test]
fn test_multi_target_with_skills() {
    let project = TestProject::new();
    project.write_config(r#"
        [output]
        targets = ["agents-md", "claude-skills"]
    "#);
    project.write_template("test.md", r#"
        ---
        name: test-skill
        description: "Test skill"
        ---
        Content
    "#);

    cagents_build(&project.root).unwrap();

    assert!(project.root.join("AGENTS.md").exists());
    assert!(project.root.join(".claude/skills/test-skill/SKILL.md").exists());
}

#[test]
fn test_conditional_skill_excluded() {
    let project = TestProject::new();
    project.write_config(r#"
        [output]
        targets = ["claude-skills"]

        [variables.static]
        use_tool = "false"
    "#);
    project.write_template("conditional.md", r#"
        ---
        name: conditional-skill
        description: "Conditional skill"
        when:
          use_tool: "true"
        ---
        Content
    "#);

    cagents_build(&project.root).unwrap();

    // Skill should NOT be generated
    assert!(!project.root.join(".claude/skills/conditional-skill").exists());
}
```

**Required Tests:**
- [x] Build with skills target only
- [x] Build with multiple targets (agents-md + skills)
- [x] Conditional template excluded from skills
- [x] Template without description fails build
- [x] Multiple templates generate multiple skills
- [x] Skill name override works

### Manual Testing Checklist

- [ ] **Test with Claude Code:**
  1. Generate skills in test project
  2. Open project in Claude Code
  3. Ask: "What are the coding conventions?" (should trigger skill)
  4. Verify Claude uses skill content in response

- [ ] **Test conditional skills:**
  1. Create template with `when.use_beads = "true"`
  2. Build without `bd` installed → skill not generated
  3. Install `bd` → build → skill appears

- [ ] **Test error messages:**
  1. Template without description → build fails with helpful error
  2. Duplicate skill names → build fails with disambiguation suggestion
  3. Invalid output directory → build fails with security warning

- [ ] **Test backwards compatibility:**
  1. Existing project without skills target → build unchanged
  2. Add skills target → both old and new outputs generated

- [ ] **Performance test:**
  1. Project with 50+ templates → build time < 1s additional
  2. Memory usage reasonable (< 50MB increase)

## Documentation Impact

### Files to Create
- [x] `docs/SKILLS.md` - Comprehensive skills guide
- [x] `examples/skills-demo/` - Working example project
- [x] `docs/rfcs/001-claude-skills-support.md` - This file

### Files to Update
- [ ] `README.md` - Add skills to core concepts
- [ ] `ADVANCED.md` - Document skill-specific frontmatter fields
- [ ] `ADVANCED.md` - Document `[skill]` config section
- [ ] `CHANGELOG.md` - Add entry for 0.2.0

### Documentation Requirements
- [ ] Usage examples with real skill descriptions
- [ ] Table of configuration options
- [ ] Troubleshooting section ("Skills not triggering")
- [ ] Migration guide (static CLAUDE.md → skills)
- [ ] Best practices for skill naming and descriptions

## Alternatives Considered

### Alternative 1: Skill Grouping (Multiple Templates → One Skill)

**Description:** Allow templates to specify `skill_group: "groupname"` to combine multiple templates into a single SKILL.md.

**Pros:**
- Reduces number of skill files (less clutter)
- Logical grouping (e.g., all TypeScript rules in one skill)

**Cons:**
- Complexity in ordering and merging templates
- Less granular skill triggering by Claude
- Harder to debug which template contributed what

**Why not chosen:** Start simple with one-to-one mapping. Grouping can be added later if users request it. Most templates are already cohesive units.

### Alternative 2: Runtime Skill Generation

**Description:** Instead of build-time, generate skills on-the-fly when Claude requests them (via a cAGENTS server).

**Pros:**
- Always up-to-date (no need to rebuild)
- Could incorporate runtime context (current file, etc.)

**Cons:**
- Significantly more complex (requires server, IPC)
- Performance overhead (template evaluation on every request)
- Harder to debug and test
- Deviates from cAGENTS's simple file-based model

**Why not chosen:** Build-time generation aligns with cAGENTS philosophy (static generation from templates). Runtime generation is a separate, much larger feature.

### Alternative 3: Monolithic SKILL.md with Sections

**Description:** Generate one large SKILL.md with all templates as sections (instead of separate skill files).

**Pros:**
- Simpler implementation (like current AGENTS.md)
- Only one file to manage

**Cons:**
- Defeats the purpose of Claude Skills (no progressive loading)
- Claude would load entire file regardless of relevance
- Same problem we're trying to solve

**Why not chosen:** Misses the core benefit of skills (on-demand loading).

## Open Questions

1. **Question:** Should we auto-generate descriptions from template content if missing?
   - **Options:**
     - A) Require explicit description (fail build if missing)
     - B) Auto-generate from first heading or first sentence
     - C) Allow empty description (but warn)
   - **Recommendation:** A (require explicit) - Ensures quality descriptions that Claude can match effectively
   - **Decision:** [To be decided]

2. **Question:** How do we handle glob-scoped templates for skills?
   - **Options:**
     - A) Ignore globs for skills (all global)
     - B) Incorporate glob context into skill description
     - C) Support per-directory skill loading (future)
   - **Recommendation:** B for now (mention scope in description) - Example: "TypeScript rules for the API package"
   - **Decision:** [To be decided]

3. **Question:** Should we support skill groups in v1 or defer?
   - **Options:**
     - A) Defer to v2 (keep v1 simple)
     - B) Implement basic grouping via `skill_group` field
   - **Recommendation:** A (defer) - Start with one-to-one mapping, add grouping based on user feedback
   - **Decision:** Defer to 0.3.0

4. **Question:** How aggressive should stale skill cleanup be?
   - **Options:**
     - A) Always remove skills not in current build
     - B) Never remove (leave cleanup to user)
     - C) Remove only if marker comment present (cAGENTS-generated)
   - **Recommendation:** C (safe cleanup with marker) - Protects user-created skills
   - **Decision:** [To be decided]

## Migration Path

### Breaking Changes
- **None** - This is an additive feature

### Backwards Compatibility
- Existing configs without `claude-skills` target: **No change**
- Existing templates without skill fields: **Work as before**
- Build command: **Unchanged behavior** for non-skills targets

### Upgrade Steps

For users wanting to adopt skills:

1. **Update config** to add `claude-skills` target:
   ```toml
   [output]
   targets = ["agents-md", "claude-skills"]  # Add claude-skills
   ```

2. **Add descriptions** to templates:
   ```yaml
   ---
   name: typescript-rules
   description: "TypeScript coding conventions"  # ← Add this
   ---
   ```

3. **Run build**:
   ```bash
   npx cagents build
   ```

4. **Verify** skills generated in `.claude/skills/`

5. **Update .gitignore**:
   ```bash
   npx cagents git ignore-outputs
   ```

6. **Test with Claude Code** - Open project and ask questions to trigger skills

## Success Metrics

- [x] Feature implemented and merged
- [x] All unit tests pass (>90% coverage for new code)
- [x] All integration tests pass
- [x] Manual testing with Claude Code successful (skills trigger correctly)
- [x] Documentation complete (README, ADVANCED, SKILLS.md, examples)
- [x] Performance acceptable (< 1s build time increase for 50 templates)
- [x] No regressions in existing functionality (all old tests pass)
- [ ] User feedback positive (after release, community discussion)

## Implementation Notes

*This section will be filled in during implementation to track deviations and learnings.*

### Deviations from Original Design
- TBD

### Lessons Learned
- TBD

### Related PRs
- TBD

---

## Status Updates

- **2025-10-29:** Proposal created, status: Draft
- **TBD:** Implementation started
- **TBD:** Completed and merged
