---
"cagents": patch
---

Fixed output file cleanup. Output files (CLAUDE.md, .cursorrules) are now properly cleaned up when removed from `[output] targets` config.

- Previously only AGENTS.md files in different directories were cleaned up
- Now tracks both output directories and target formats in `.cAGENTS/.output-cache`
- Automatically removes orphaned output files on next build
