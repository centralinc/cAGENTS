# Research Prompt: Progressive Disclosure API Design for cAGENTS

## Context

**cAGENTS** is a cross-platform generator for `AGENTS.md` and compatible rule formats. It composes rule files (Markdown + YAML front-matter), renders them with a BYOC (Bring-Your-Own-Compiler) engine, and writes context-scoped outputs at the repo root and nested directories.

**Project Repository:** https://github.com/anthropics/cagents (internal paths provided below)

---

## Problem Statement

From RFC 0002 - Progressive Disclosure:

> Static instruction files (AGENTS.md, CLAUDE.md) currently dump all rules into the AI agent's context at once, regardless of relevance to the current task. This creates several problems:
>
> 1. **Context pollution**: 300+ line files overwhelm the model with potentially irrelevant information
> 2. **Poor signal-to-noise ratio**: Critical rules get buried in less relevant content
> 3. **Cognitive overload**: Both AI and developers struggle to parse large instruction files
> 4. **No prioritization**: All rules appear equally important

The original cAGENTS goal: deliver "the right amount of context at the right time" rather than a 300-line instruction dump.

---

## Current API Structure

### Configuration Model (`crates/cagents-core/src/model.rs`)

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub engine: Option<String>,
    pub globs: Option<Vec<String>>,
    pub order: Option<i32>,
    pub when: Option<When>,
    pub vars: Option<serde_json::Value>,
    pub merge: Option<Merge>,
    pub links: Option<Vec<Link>>,
    pub targets: Option<Vec<String>>,
    pub extends: Option<Vec<String>>,
    pub output_in: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct When {
    // Legacy fields
    pub env: Option<Vec<String>>,
    pub role: Option<Vec<String>>,
    pub language: Option<Vec<String>>,
    pub target: Option<Vec<String>>,

    // Arbitrary variables (all other fields)
    #[serde(flatten)]
    pub variables: std::collections::HashMap<String, serde_json::Value>,
}
```

### Config File Structure (`.cAGENTS/config.toml`)

```toml
[paths]
templatesDir = ".cAGENTS/templates"
outputRoot = "."

[defaults]
engine = "builtin:simple"
targets = ["agents-md"]
order = 100

[variables.command]
use_beads = "command -v bd >/dev/null && echo true || echo false"

[output]
targets = ["agents-md", "claude-md", "cursorrules"]
```

### Example Template Usage

```yaml
---
name: database-migrations
globs: ["migrations/**"]
when:
  language: ["typescript"]
  role: ["backend"]
order: 50
---
# Database Migration Rules

When creating migrations...
```

---

## RFC 0002 Proposed Approach

The RFC proposes adding:

1. **`core: bool` field** in RuleFrontmatter (default: true)
   - `core: true` → "Core Guidelines" section (always relevant)
   - `core: false` → "Additional Context-Specific Guidelines" section

2. **Output configuration options:**
   ```rust
   pub struct OutputConfig {
       pub progressive_hints: bool,        // Default: true
       pub toc_threshold: usize,           // Default: 10 (sections)
   }
   ```

3. **Enhanced AGENTS.md structure:**
   ```markdown
   # Project Rules

   ## Table of Contents
   - [Core Rule 1](#core-rule-1)
   - **Additional Context-Specific Guidelines:**
     - [DB Migrations](#db-migrations) *(when working with migrations/**)*

   ---

   ## Core Guidelines
   *(Universally applicable rules)*

   ### Core Rule 1
   ...

   ---

   ## Additional Context-Specific Guidelines
   *(The following sections apply in specific scenarios. Refer to them as needed.)*

   ### DB Migrations
   *(Applies to files in `migrations/**`)*
   ...
   ```

**Key insight:** This is a binary (core/additional) classification system with static sectioning.

---

## Research Goal

**Analyze and propose alternative API designs that optimize for:**

1. **Human iteration velocity** - Developers should be able to easily:
   - Mark rules as high/low priority
   - Understand what gets shown when
   - Quickly iterate on organization without complex syntax
   - Debug why rules appear/don't appear for AI agents

2. **AI agent discoverability** - AI agents should be able to:
   - Easily ask for more relevant context when needed
   - Understand what additional sections are available
   - Know when to request specific context (via natural language or structured queries)
   - Progressively load context without overwhelming token limits

---

## Specific Research Questions

### 1. Classification API

**Current proposal:** Binary `core: bool` field

**Research:**
- Is binary (core/additional) sufficient, or should we support priority levels (P0/P1/P2)?
- Should classification be explicit (field) or implicit (inferred from `when` conditions)?
- Alternative naming: `core`, `priority`, `visibility`, `relevance`, `tier`?
- Should there be shorthand syntax? E.g., `priority: high` vs `core: true`

**Examples to explore:**
```yaml
# Option A: Binary
core: false

# Option B: Priority levels
priority: 2  # or "low", "medium", "high"

# Option C: Visibility model
visibility: conditional  # or "always", "on-demand"

# Option D: Implicit (based on when clause)
when:
  scope: specific  # infers it's not core
```

### 2. Query/Discovery API

**Current proposal:** Static TOC with annotations like `*(when working with migrations/**)*`

**Research:**
- How should AI agents "ask for more context"?
- Should there be a structured query mechanism beyond static markdown?
- Could we support natural language hints? E.g., "For database work, see: [DB Migrations]"
- Should there be metadata that AI agents can parse programmatically?

**Examples to explore:**
```yaml
# Option A: Enhanced annotations in frontmatter
topics: ["database", "migrations", "postgres"]
keywords: ["schema", "migration", "db"]

# Option B: Discovery metadata
discovery:
  summary: "Rules for database schema migrations"
  when_needed: "modifying files in migrations/** or discussing database schema"

# Option C: Tags/categories
tags: ["backend", "database", "infrastructure"]
category: database-operations

# Option D: Conditional prompts
discover_when: "the AI agent is working with database migrations or schema changes"
```

### 3. Configuration UX

**Current proposal:** New top-level `[output]` section fields

**Research:**
- Where should progressive disclosure config live? Top-level, per-template, or both?
- Should defaults be global with template-level overrides?
- How do users control TOC generation, section headers, annotations?

**Examples to explore:**
```toml
# Option A: Global config
[output]
progressive_hints = true
toc_threshold = 10
section_names.core = "Core Guidelines"
section_names.additional = "Additional Context-Specific"

# Option B: Per-template control
# (in template frontmatter)
progressive:
  tier: additional
  show_in_toc: true
  annotation: "For database work"

# Option C: Feature flags
[features]
progressive_disclosure = true
auto_toc = true
contextual_annotations = true

# Option D: Strategy-based
[output.strategy]
type = "progressive"  # or "flat", "hierarchical"
core_threshold = 10  # show TOC if core > 10 sections
```

### 4. Annotation Generation

**Current proposal:** Auto-generate from `when` conditions and `globs`

**Research:**
- Should annotations be auto-generated, manually specified, or both?
- What information is most useful in annotations?
- How verbose should they be?

**Examples to explore:**
```yaml
# Option A: Auto-generated (current RFC)
# → *(when working with migrations/**, in production environment)*

# Option B: Manual override
annotation: "Use these rules when modifying database schemas"

# Option C: Template-based
annotation_template: "For {globs} in {environment}"

# Option D: Structured hints
hints:
  file_patterns: ["migrations/**"]
  environments: ["production"]
  tools_required: ["psql", "migrate"]
  summary: "Database migration guidelines"
```

### 5. CLI/Build Integration

**Research:**
- Should there be CLI commands to preview progressive disclosure?
- How do users debug which rules are core vs. additional?
- Should `cagents lint` validate progressive disclosure config?

**Examples to explore:**
```bash
# Preview modes
cagents preview --show-structure
cagents preview --filter core
cagents preview --filter additional

# Debugging
cagents explain <template-name>  # Why is this template core/additional?
cagents list --by-priority        # Show templates grouped

# Validation
cagents lint --check-progressive  # Warn if too many core, etc.
```

---

## Deliverables

Please provide:

1. **Comparison Matrix**
   - Compare 3-5 alternative API designs
   - Rate each on: simplicity, flexibility, AI-friendliness, human ergonomics
   - Include pros/cons for each approach

2. **Recommended API Design**
   - Specific syntax for config files and template frontmatter
   - Examples of common use cases (3-5 scenarios)
   - Migration path from current `when` conditions

3. **AI Agent Integration Examples**
   - How would Claude Code/Cursor/Cline consume this?
   - Mock examples of "AI asks for more context" workflows
   - Natural language patterns for discovery

4. **Edge Cases & Considerations**
   - What happens with 100+ templates?
   - How to handle templates that are conditionally core?
   - Monorepo scenarios (nested AGENTS.md files)

5. **Implementation Complexity**
   - Rate each alternative: low/medium/high complexity
   - Identify which parts can be done incrementally
   - Flag any breaking changes

---

## Reference Links

- **RFC 0002 Full Text:** `/Users/jordan/central/cagents/docs/rfcs/docs/rfcs/0002-progressive-disclosure.md`
- **Current Model:** `/Users/jordan/central/cagents/crates/cagents-core/src/model.rs`
- **Current Config:** `/Users/jordan/central/cagents/crates/cagents-core/src/config.rs`
- **README Examples:** `/Users/jordan/central/cagents/README.md`

---

## Success Criteria

The ideal API should:

✅ Be **immediately understandable** to developers (no manual reading required)
✅ Support **rapid iteration** (change priority without restructuring)
✅ Be **AI-parseable** (structured metadata for programmatic access)
✅ Enable **natural discovery** (AI agents can find relevant sections easily)
✅ Scale to **100+ templates** without degrading UX
✅ Maintain **backward compatibility** (or provide clear migration path)
✅ Be **implementation-feasible** in Rust with TOML/YAML/Markdown

---

## Additional Context

- **Target users:** Open-source developers, teams using AI coding agents
- **Primary AI tools:** Claude Code, Cursor, Cline, Windsurf, Codex, Aider
- **Language ecosystem:** Rust core, npm distribution, works with any language
- **Philosophy:** "Adaptable instruction for codegen models" - context should adapt to environment, not be static

**Key question:** How do we balance static file generation (AGENTS.md) with dynamic context needs (AI agents that want more info)?

---

## Notes

- This is pre-implementation research (no code written yet)
- RFC 0002 is in "Draft" status
- Other RFCs in flight: Claude Skills Support, Multi-Format Consistency, CLI Enhancements
- The team follows strict TDD (tests first, then implementation)

---

**Thank you for your research! Please be thorough, opinionated, and provide concrete examples.**
