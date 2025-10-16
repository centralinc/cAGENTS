---
"cagents": patch
---

Fixed import of .cursor/rules to recursively process all Markdown files in subdirectories. Previously only top-level rules were imported, now nested rules are correctly discovered and imported with flattened names (e.g., `subdir/file.md` becomes `agents-subdir-file.md`).
