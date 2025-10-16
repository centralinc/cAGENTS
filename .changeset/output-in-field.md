---
"cagents": patch
---

Added new `outputIn` field for glob output control. More intuitive control over where AGENTS.md files are created.

- `outputIn: matched` - Create output IN matched directories (for dir globs with trailing slash)
- `outputIn: parent` - Create in parent of matched files (one per directory)
- `outputIn: common-parent` - Find common parent, create single output there (default)
- Directory glob support: Patterns ending with `/` now match directories
