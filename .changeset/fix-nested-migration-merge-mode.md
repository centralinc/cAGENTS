---
"cagents": patch
---

Fix nested file detection in multi-format migration. When using 'merge all formats' option, nested AGENTS.md and CLAUDE.md files are now properly discovered and imported, matching the behavior of single-format migration.
