---
"cagents": patch
---

**BREAKING:** Removed `simplifyGlobsToParent` field from template frontmatter. Use the new `outputIn` field instead for clearer glob output semantics.

**Migration:**
- Replace `simplifyGlobsToParent: true` with `outputIn: common-parent`
- Replace `simplifyGlobsToParent: false` with `outputIn: parent`
- If omitted, defaults to `common-parent` (same as old default)

**New `outputIn` values:**
- `common-parent`: Find common parent directory, create single output there
- `parent`: Create output in parent directory of each matched file
- `matched`: Create output IN matched directories (for directory globs with trailing slash)

This change makes glob output behavior more intuitive and explicit.
