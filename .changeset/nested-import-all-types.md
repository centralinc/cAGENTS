---
"cagents": patch
---

Added consistent nested file support across all import types. The `import` command now recursively discovers and imports nested files for all formats:

- `AGENTS.md` files in subdirectories are imported as separate templates (e.g., `backend/AGENTS.md` → `agents-backend.md`)
- `CLAUDE.md` files in subdirectories are imported as separate templates (e.g., `frontend/CLAUDE.md` → `agents-frontend.md`)
- `.cursorrules` files in subdirectories are imported as separate templates (e.g., `backend/.cursorrules` → `agents-cursor-backend.md`)

Root-level files are named with `-root` suffix (e.g., `agents-root.md`, `agents-cursor-root.md`) for consistency with nested files.
