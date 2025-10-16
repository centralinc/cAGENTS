---
"cagents": patch
---

**BREAKING:** Removed `alwaysApply` field from template frontmatter. Rules without a `when` clause now implicitly apply in all contexts. This simplifies the API by removing redundancy.

**Migration:** Remove `alwaysApply: true` from all templates. Rules without `when` clauses will automatically apply everywhere.

**Behavior:** Rules are now filtered only by context (`when` clause) and file matching (globs). If a rule has no `when` clause, it applies in all contexts, but still respects glob patterns for file matching.
