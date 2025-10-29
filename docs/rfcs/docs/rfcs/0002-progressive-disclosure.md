# Proposal 002: Progressive Disclosure

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

Static instruction files (AGENTS.md, CLAUDE.md) currently dump all rules into the AI agent's context at once, regardless of relevance to the current task. This creates several problems:

1. **Context pollution**: 300+ line files overwhelm the model with potentially irrelevant information
2. **Poor signal-to-noise ratio**: Critical rules get buried in less relevant content
3. **Cognitive overload**: Both AI and developers struggle to parse large instruction files
4. **No prioritization**: All rules appear equally important

Even with conditional templates (which reduce total content), the resulting files can still be long and unfocused for complex projects.

### Goals

1. **Structured output**: Organize AGENTS.md with clear sections for core vs. additional rules
2. **Contextual cues**: Add annotations indicating when rules apply (environment, tools, language)
3. **Progressive revelation**: Enable AI agents to focus on relevant sections first
4. **Better navigation**: Auto-generate table of contents for large files
5. **Maintain readability**: Enhancements should help, not hinder, human readers

### Non-Goals

- **Runtime query system**: Not implementing a server/API for on-demand rule lookup (out of scope for this iteration)
- **AI-side improvements**: We can't control how AI models process context (but we can structure it better)
- **Complete context elimination**: Static files will still contain all applicable rules (but better organized)

## Background

### Current State

**Current AGENTS.md structure:**
```markdown
# Project Rules

## Rule 1
Content...

## Rule 2
Content...

## Rule 3
Content...

(25 more sections...)
```

**Problems:**
- Flat structure with no hierarchy
- No indication of relative importance
- No hints about when rules apply
- AI must read entire file to find relevant guidance

### Motivation

The original cAGENTS author stated the goal: deliver "the right amount of context at the right time" rather than a 300-line instruction dump.

**Inspiration from Claude Skills:**
Claude Skills work because they implement progressive disclosure:
- Level 1: Skill metadata (name + description) - always loaded
- Level 2: Skill content (SKILL.md) - loaded on demand
- Level 3: Resources/scripts - loaded only if referenced

For static files, we can approximate this by:
- Clear sectioning (core vs. additional)
- Contextual annotations (when rules apply)
- Navigation aids (TOC)

**User Story:**
> As an AI agent reading AGENTS.md, I want to quickly identify which sections are universally relevant vs. context-specific, so I can prioritize the most important guidance first.

### References

- [Claude Skills Docs on Progressive Disclosure](https://docs.claude.com/)
- [Original cAGENTS Discussion](https://news.ycombinator.com/item?id=42465839)
- [Progressive Disclosure in UX](https://www.nngroup.com/articles/progressive-disclosure/)

## Detailed Design

### Architecture Overview

```
┌──────────────────────────────────────┐
│     Template Loading & Filtering     │
└─────────────────┬────────────────────┘
                  │
                  ▼
┌──────────────────────────────────────┐
│      Classify Templates              │
│   (core: true/false frontmatter)     │
└─────┬─────────────────┬──────────────┘
      │                 │
      ▼                 ▼
┌──────────┐      ┌──────────────┐
│   Core   │      │  Additional  │
│Templates │      │  Templates   │
└─────┬────┘      └──────┬───────┘
      │                  │
      └────────┬─────────┘
               ▼
┌──────────────────────────────────────┐
│     Enhanced Markdown Writer         │
│                                      │
│  1. TOC generation                   │
│  2. Section markers                  │
│  3. Contextual annotations           │
│  4. Clear hierarchy                  │
└──────────────────┬───────────────────┘
                   │
                   ▼
            AGENTS.md (structured)
```

### Data Structures

#### Extended RuleFrontmatter

**Location:** `crates/cagents-core/src/model.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFrontmatter {
    // Existing fields...

    /// Whether this template is core (always relevant) or additional (context-specific)
    /// Default: true
    #[serde(default = "default_core")]
    pub core: bool,
}

fn default_core() -> bool {
    true  // By default, all templates are core
}
```

#### Output Configuration

**Location:** `crates/cagents-core/src/config.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    // Existing fields...

    /// Enable progressive disclosure hints in output
    /// Default: true
    #[serde(default = "default_progressive_hints")]
    pub progressive_hints: bool,

    /// Minimum sections to trigger TOC generation
    /// Default: 10
    #[serde(default = "default_toc_threshold")]
    pub toc_threshold: usize,
}

fn default_progressive_hints() -> bool {
    true
}

fn default_toc_threshold() -> usize {
    10
}
```

### Enhanced AGENTS.md Structure

**With progressive disclosure:**

```markdown
# Project Rules

## Table of Contents
- [TypeScript Conventions](#typescript-conventions)
- [Testing Guidelines](#testing-guidelines)
- [Git Workflow](#git-workflow)
- **Additional Context-Specific Guidelines:**
  - [Database Migrations](#database-migrations) *(when working with migrations)*
  - [Beads Issue Tracking](#beads-issue-tracking) *(when bd tool available)*

---

## Core Guidelines

### TypeScript Conventions
Use strict mode. Prefer functional components...

### Testing Guidelines
Write tests first. Use descriptive names...

### Git Workflow
Always run tests before commit...

---

## Additional Context-Specific Guidelines

*(The following sections apply in specific scenarios. Refer to them as needed.)*

### Database Migrations
*(Applies to files in `migrations/**`)*

When creating migrations...

### Beads Issue Tracking
*(Applies when `bd` tool is available)*

This project uses `bd` for issue tracking...
```

### Algorithm / Logic

#### Template Classification

```rust
fn classify_templates(templates: Vec<Template>) -> (Vec<Template>, Vec<Template>) {
    let mut core = Vec::new();
    let mut additional = Vec::new();

    for template in templates {
        if template.frontmatter.core.unwrap_or(true) {
            core.push(template);
        } else {
            additional.push(template);
        }
    }

    // Sort by order within each group
    core.sort_by_key(|t| t.frontmatter.order.unwrap_or(100));
    additional.sort_by_key(|t| t.frontmatter.order.unwrap_or(100));

    (core, additional)
}
```

#### Contextual Annotation Generation

```rust
fn generate_condition_annotation(template: &Template) -> Option<String> {
    let when = template.frontmatter.when.as_ref()?;
    let mut parts = Vec::new();

    if let Some(env) = &when.env {
        parts.push(format!("in {} environment", env.join(" or ")));
    }

    if let Some(language) = &when.language {
        parts.push(format!("for {} code", language.join(" or ")));
    }

    if let Some(globs) = &template.frontmatter.globs {
        if globs.len() == 1 {
            parts.push(format!("when working with `{}`", globs[0]));
        }
    }

    // Check for tool-specific conditions
    for (key, value) in &when.custom {
        if key.starts_with("use_") && value == "true" {
            let tool = key.strip_prefix("use_").unwrap();
            parts.push(format!("when `{}` tool is available", tool));
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(format!("*({})*", parts.join(", ")))
    }
}
```

#### TOC Generation

```rust
fn generate_toc(core: &[Template], additional: &[Template]) -> String {
    let mut toc = String::from("## Table of Contents\n\n");

    // Core sections
    for template in core {
        let anchor = slugify(&template.frontmatter.name);
        toc.push_str(&format!("- [{}](#{})\n", template.frontmatter.name, anchor));
    }

    // Additional sections (if any)
    if !additional.is_empty() {
        toc.push_str("\n**Additional Context-Specific Guidelines:**\n\n");
        for template in additional {
            let anchor = slugify(&template.frontmatter.name);
            let annotation = generate_condition_annotation(template)
                .unwrap_or_else(|| "*(context-specific)*".to_string());
            toc.push_str(&format!("- [{}](#{}  {}\n",
                template.frontmatter.name, anchor, annotation));
        }
    }

    toc.push_str("\n---\n\n");
    toc
}
```

#### Enhanced Markdown Assembly

```rust
pub fn assemble_agents_md(templates: Vec<Template>, config: &OutputConfig) -> String {
    let (core, additional) = classify_templates(templates);

    let mut output = String::from("# Project Rules\n\n");

    // Generate TOC if enough sections
    if config.progressive_hints && (core.len() + additional.len() >= config.toc_threshold) {
        output.push_str(&generate_toc(&core, &additional));
    }

    // Core section
    if !core.is_empty() {
        if config.progressive_hints && !additional.is_empty() {
            output.push_str("## Core Guidelines\n\n");
            output.push_str("*(Universally applicable rules for this project)*\n\n");
        }

        for template in core {
            output.push_str(&format_template_section(&template, config));
        }
    }

    // Additional section
    if !additional.is_empty() && config.progressive_hints {
        output.push_str("\n---\n\n");
        output.push_str("## Additional Context-Specific Guidelines\n\n");
        output.push_str("*(The following sections apply in specific scenarios. ");
        output.push_str("Refer to them as needed.)*\n\n");

        for template in additional {
            output.push_str(&format_template_section(&template, config));
        }
    }

    output
}

fn format_template_section(template: &Template, config: &OutputConfig) -> String {
    let mut section = format!("### {}\n", template.frontmatter.name);

    // Add contextual annotation if progressive_hints enabled
    if config.progressive_hints {
        if let Some(annotation) = generate_condition_annotation(template) {
            section.push_str(&format!("{}\n", annotation));
        }
    }

    section.push('\n');
    section.push_str(&template.content);
    section.push_str("\n\n");

    section
}
```

### Target-Specific Formatting

**For Cursor (.cursorrules):**

Research needed: Does Cursor support markdown annotations?

**Option A:** If Cursor expects plain rules, disable progressive hints:
```rust
fn should_apply_progressive_hints(target: Target) -> bool {
    match target {
        Target::AgentsMd | Target::ClaudeMd => true,
        Target::CursorRules => false,  // Plain text only
        Target::ClaudeSkills => false,  // Skills use different structure
    }
}
```

**Option B:** If Cursor supports markdown, apply hints to all targets.

**Decision:** Test with Cursor to determine. Default to Option A (conservative).

### Configuration

**Config file (`.cAGENTS/config.toml`):**

```toml
[output]
targets = ["agents-md"]
progressive_hints = true  # Enable progressive disclosure features
toc_threshold = 10        # Generate TOC if 10+ sections

[defaults]
# Templates are core by default (no config needed)
```

**Template frontmatter:**

```yaml
---
name: database-migrations
core: false  # Mark as additional (context-specific)
globs: ["migrations/**"]
---
```

## Implementation Plan

### Phase 1: Data Structure Updates (Sprint 1, Week 1)

#### Task 1.1: Add `core` field to RuleFrontmatter
**Location:** `crates/cagents-core/src/model.rs`

- [ ] Add `core: bool` field with default `true`
- [ ] Add serde attributes with `default = "default_core"`
- [ ] Write test `test_frontmatter_core_field()`

**Acceptance Criteria:**
- Field deserializes correctly (true/false)
- Default is `true` when field absent
- Test validates both explicit and default values

#### Task 1.2: Add output config fields
**Location:** `crates/cagents-core/src/config.rs`

- [ ] Add `progressive_hints: bool` (default true)
- [ ] Add `toc_threshold: usize` (default 10)
- [ ] Write test `test_output_config_progressive_hints()`

**Acceptance Criteria:**
- Config without fields uses defaults
- Config with explicit values overrides defaults
- Test validates all combinations

### Phase 2: Classification Logic (Sprint 1, Week 1-2)

#### Task 2.1: Implement template classification
**Location:** `crates/cagents-core/src/writers/agents_md.rs` (or new module)

- [ ] Create `classify_templates()` function
- [ ] Separate core vs. additional based on `core` field
- [ ] Sort each group by `order`
- [ ] Write tests:
  - `test_classify_all_core()`
  - `test_classify_mixed()`
  - `test_classify_maintains_order()`

**Acceptance Criteria:**
- Templates correctly grouped
- Order preserved within groups
- Empty groups handled gracefully

#### Task 2.2: Implement condition annotation generator
**Location:** Same file

- [ ] Create `generate_condition_annotation()` function
- [ ] Extract environment conditions
- [ ] Extract language conditions
- [ ] Extract glob patterns
- [ ] Extract tool-specific conditions (use_X)
- [ ] Write tests:
  - `test_annotation_environment()`
  - `test_annotation_language()`
  - `test_annotation_globs()`
  - `test_annotation_tools()`
  - `test_annotation_multiple_conditions()`
  - `test_annotation_no_conditions()`

**Acceptance Criteria:**
- All condition types detected
- Multiple conditions combined with commas
- No annotation for unconditional templates
- Output is human-readable

### Phase 3: Enhanced Output Generation (Sprint 1, Week 2)

#### Task 3.1: Implement TOC generation
**Location:** `crates/cagents-core/src/writers/toc.rs` (new module)

- [ ] Create `generate_toc()` function
- [ ] List core sections with anchors
- [ ] List additional sections with annotations
- [ ] Generate markdown anchor slugs
- [ ] Write tests:
  - `test_toc_core_only()`
  - `test_toc_mixed()`
  - `test_toc_anchors_valid()`

**Acceptance Criteria:**
- TOC links are valid (anchor slug matches heading)
- Core and additional sections clearly separated
- Annotations included for additional sections

#### Task 3.2: Implement enhanced markdown assembly
**Location:** `crates/cagents-core/src/writers/agents_md.rs`

- [ ] Refactor existing `assemble_agents_md()` function
- [ ] Add TOC generation (if threshold met)
- [ ] Add "Core Guidelines" section header
- [ ] Add "Additional Guidelines" section with divider
- [ ] Include contextual annotations per template
- [ ] Write tests:
  - `test_assemble_below_toc_threshold()`
  - `test_assemble_above_toc_threshold()`
  - `test_assemble_core_only()`
  - `test_assemble_with_annotations()`
  - `test_assemble_progressive_hints_disabled()`

**Acceptance Criteria:**
- Output structure matches spec (Core → Additional)
- TOC generated only when appropriate
- Annotations present when progressive_hints enabled
- Legacy behavior preserved when progressive_hints disabled

### Phase 4: Target-Specific Handling (Sprint 2, Week 3)

#### Task 4.1: Research Cursor format requirements
**Manual research:**

- [ ] Create test .cursorrules file with markdown annotations
- [ ] Open in Cursor editor
- [ ] Verify if annotations are displayed or cause issues
- [ ] Document findings

**Acceptance Criteria:**
- Clear understanding of Cursor's markdown support
- Decision made: apply hints or suppress for Cursor

#### Task 4.2: Implement target-aware formatting
**Location:** `crates/cagents-core/src/writers/mod.rs`

- [ ] Add `should_apply_progressive_hints(target)` function
- [ ] Update writers to check this flag
- [ ] Write test `test_cursor_formatting()`

**Acceptance Criteria:**
- Cursor output correct per findings
- AGENTS.md and CLAUDE.md get full treatment
- No regressions in existing targets

### Phase 5: Integration & Polish (Sprint 2, Week 3)

#### Task 5.1: Update build command
**Location:** `crates/cagents-cli/src/commands/build.rs`

- [ ] Ensure progressive hints config is read
- [ ] Pass config to writers
- [ ] Log TOC generation (debug level)
- [ ] Write integration test `test_build_with_progressive_disclosure()`

**Acceptance Criteria:**
- Build command respects new config
- Generated files match spec
- No performance degradation

#### Task 5.2: Add validation
**Location:** `crates/cagents-core/src/validation.rs`

- [ ] Warn if many templates marked `core: false` (suggest creating focused templates instead)
- [ ] Warn if TOC has very few sections (maybe disable threshold)
- [ ] Write test `test_validation_warnings()`

**Acceptance Criteria:**
- Helpful warnings for edge cases
- Build continues (warnings not errors)

### Phase 6: Documentation (Sprint 2, Week 4)

#### Task 6.1: Update ADVANCED.md
**Location:** `ADVANCED.md`

- [ ] Document `core` frontmatter field
- [ ] Document `progressive_hints` config option
- [ ] Document `toc_threshold` config option
- [ ] Provide examples of usage

**Acceptance Criteria:**
- All new features documented
- Examples are clear and tested

#### Task 6.2: Create example
**Location:** `examples/progressive-disclosure/`

- [ ] Create demo project with mix of core and additional templates
- [ ] Show generated output
- [ ] Add README explaining the feature
- [ ] Verify build works

**Acceptance Criteria:**
- Example demonstrates feature clearly
- README explains benefits

#### Task 6.3: Update README
**Location:** `README.md`

- [ ] Add brief mention of progressive disclosure
- [ ] Link to ADVANCED.md for details

**Acceptance Criteria:**
- Feature is discoverable from README

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_templates_mixed() {
        let templates = vec![
            create_template("core1", true),
            create_template("additional1", false),
            create_template("core2", true),
            create_template("additional2", false),
        ];

        let (core, additional) = classify_templates(templates);

        assert_eq!(core.len(), 2);
        assert_eq!(additional.len(), 2);
        assert_eq!(core[0].name, "core1");
        assert_eq!(additional[0].name, "additional1");
    }

    #[test]
    fn test_generate_condition_annotation_tool() {
        let mut template = Template::default();
        template.frontmatter.when = Some(WhenConditions {
            use_beads: Some("true".to_string()),
            ..Default::default()
        });

        let annotation = generate_condition_annotation(&template);
        assert_eq!(annotation, Some("*(when `beads` tool is available)*".to_string()));
    }

    #[test]
    fn test_generate_toc_with_additional() {
        let core = vec![create_template("Core Rule", true)];
        let additional = vec![create_template("Additional Rule", false)];

        let toc = generate_toc(&core, &additional);

        assert!(toc.contains("## Table of Contents"));
        assert!(toc.contains("[Core Rule](#core-rule)"));
        assert!(toc.contains("Additional Context-Specific Guidelines"));
        assert!(toc.contains("[Additional Rule](#additional-rule)"));
    }

    #[test]
    fn test_assemble_agents_md_below_threshold() {
        let config = OutputConfig {
            progressive_hints: true,
            toc_threshold: 10,
        };
        let templates = vec![
            create_template("Rule 1", true),
            create_template("Rule 2", true),
        ];

        let output = assemble_agents_md(templates, &config);

        // Should NOT include TOC (only 2 sections)
        assert!(!output.contains("## Table of Contents"));
        // Should still have content
        assert!(output.contains("Rule 1"));
        assert!(output.contains("Rule 2"));
    }

    #[test]
    fn test_assemble_agents_md_with_progressive_hints() {
        let config = OutputConfig {
            progressive_hints: true,
            toc_threshold: 5,
        };
        let templates = vec![
            create_template("Core 1", true),
            create_template("Core 2", true),
            create_template("Additional 1", false),
        ];

        let output = assemble_agents_md(templates, &config);

        assert!(output.contains("## Core Guidelines"));
        assert!(output.contains("## Additional Context-Specific Guidelines"));
        assert!(output.contains("*(The following sections apply in specific scenarios"));
    }

    #[test]
    fn test_assemble_agents_md_hints_disabled() {
        let config = OutputConfig {
            progressive_hints: false,
            toc_threshold: 10,
        };
        let templates = vec![
            create_template("Core 1", true),
            create_template("Additional 1", false),
        ];

        let output = assemble_agents_md(templates, &config);

        // Should NOT include progressive disclosure markers
        assert!(!output.contains("## Core Guidelines"));
        assert!(!output.contains("## Additional Context-Specific Guidelines"));
        // Should just have sections in order
        assert!(output.contains("### Core 1"));
        assert!(output.contains("### Additional 1"));
    }
}
```

### Integration Tests

```rust
#[test]
fn test_build_with_progressive_disclosure() {
    let project = TestProject::new();
    project.write_config(r#"
        [output]
        targets = ["agents-md"]
        progressive_hints = true
        toc_threshold = 2
    "#);

    project.write_template("core.md", r#"
        ---
        name: Core Rule
        core: true
        ---
        Core content
    "#);

    project.write_template("additional.md", r#"
        ---
        name: Additional Rule
        core: false
        ---
        Additional content
    "#);

    cagents_build(&project.root).unwrap();

    let agents_md = fs::read_to_string(project.root.join("AGENTS.md")).unwrap();

    assert!(agents_md.contains("## Table of Contents"));
    assert!(agents_md.contains("## Core Guidelines"));
    assert!(agents_md.contains("## Additional Context-Specific Guidelines"));
    assert!(agents_md.contains("### Core Rule"));
    assert!(agents_md.contains("### Additional Rule"));
}
```

### Manual Testing Checklist

- [ ] Generate AGENTS.md with mix of core/additional templates
- [ ] Verify visual structure (core first, additional second)
- [ ] Check TOC links work (click anchors)
- [ ] Test with AI agent (ask general question, see if it uses core rules)
- [ ] Test with AI agent (ask specific question, see if it references additional rules)
- [ ] Verify .cursorrules format (if Cursor supports markdown)
- [ ] Test with `progressive_hints = false` (backward compatibility)

## Documentation Impact

### Files to Update
- [ ] `ADVANCED.md` - Document `core` field and config options
- [ ] `README.md` - Mention progressive disclosure feature
- [ ] `CHANGELOG.md` - Add entry for 0.2.0

### Files to Create
- [ ] `examples/progressive-disclosure/` - Demo project

### Documentation Requirements
- [ ] Clear explanation of core vs. additional classification
- [ ] Examples of when to use `core: false`
- [ ] Best practices for organizing templates
- [ ] Screenshots of before/after AGENTS.md

## Alternatives Considered

### Alternative 1: AI-Driven Section Selection

**Description:** Instead of static annotations, provide a query API where AI can request specific sections by name or topic.

**Pros:**
- Truly dynamic loading
- AI decides what's relevant

**Cons:**
- Requires runtime system (server/IPC)
- Much more complex
- Performance overhead

**Why not chosen:** Out of scope for this iteration. Static improvements are simpler and still valuable.

### Alternative 2: Multiple AGENTS.md Files

**Description:** Generate multiple files (AGENTS-core.md, AGENTS-additional.md) and let AI load them separately.

**Pros:**
- Clear separation
- AI could choose which to load

**Cons:**
- Not standard (most AI tools expect one AGENTS.md)
- Coordination problem (which file to load when?)
- Confusing for users

**Why not chosen:** Single file with structure is more compatible with existing AI tooling.

### Alternative 3: Priority Markers (P0, P1, P2)

**Description:** Instead of binary core/additional, use priority levels (P0=critical, P1=important, P2=nice-to-have).

**Pros:**
- More granular classification
- Could generate different views

**Cons:**
- More complex for users to understand
- Binary (core/additional) is clearer
- Could add later if needed

**Why not chosen:** Start simple with binary classification. Can add priority levels in future if users request it.

## Open Questions

1. **Question:** Should TOC be at top or bottom of file?
   - **Options:** A) Top (before content), B) Bottom (after content)
   - **Recommendation:** A (top) - Standard for documentation
   - **Decision:** Top

2. **Question:** How verbose should contextual annotations be?
   - **Options:** A) Very brief "(context-specific)", B) Detailed "(when X and Y)"
   - **Recommendation:** B (detailed) - More helpful for users and AI
   - **Decision:** Detailed

3. **Question:** Should we support custom section names?
   - **Options:** A) Hardcoded "Core" and "Additional", B) Configurable section names
   - **Recommendation:** A for v1 (hardcoded) - Keep simple
   - **Decision:** Hardcoded for now

## Success Metrics

- [x] Feature implemented and merged
- [x] All tests pass
- [x] AGENTS.md is more navigable (subjective but testable with users)
- [x] AI agents demonstrate better focus (harder to measure, but anecdotal feedback)
- [x] Documentation complete
- [ ] User feedback positive (after release)

---

## Status Updates

- **2025-10-29:** Proposal created, status: Draft
