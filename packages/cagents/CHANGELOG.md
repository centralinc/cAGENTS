# cagents

## 0.3.1

### Patch Changes

- b09f4b8: Added arbitrary variables in `when` clauses. You can now use any variable from config in conditional rules.

  - Define variables in config: `[variables.command] app_env = "echo $APP_ENV"`
  - Use in when clauses: `when: { app_env: ["production", "staging"] }`
  - Config variables are evaluated and available in `when` conditionals
  - Use config variables for flexible conditional rule matching

- b09f4b8: Added cAGENTS usage footer to all generated files. This footer includes best practices:

  - Explains `cagents build` and `cagents context` workflow
  - Helps AI agents work more efficiently with cAGENTS-managed codebases

- b09f4b8: Fixed output file cleanup. Output files (CLAUDE.md, .cursorrules) are now properly cleaned up when removed from `[output] targets` config.

  - Previously only AGENTS.md files in different directories were cleaned up
  - Now tracks both output directories and target formats in `.cAGENTS/.output-cache`
  - Automatically removes orphaned output files on next build

- b09f4b8: Added new `outputIn` field for glob output control. More intuitive control over where AGENTS.md files are created.

  - `outputIn: matched` - Create output IN matched directories (for dir globs with trailing slash)
  - `outputIn: parent` - Create in parent of matched files (one per directory)
  - `outputIn: common-parent` - Find common parent, create single output there (default)
  - Directory glob support: Patterns ending with `/` now match directories

- b09f4b8: **BREAKING:** Removed `alwaysApply` field from template frontmatter. Rules without a `when` clause now implicitly apply in all contexts. This simplifies the API by removing redundancy.

  **Migration:** Remove `alwaysApply: true` from all templates. Rules without `when` clauses will automatically apply everywhere.

  **Behavior:** Rules are now filtered only by context (`when` clause) and file matching (globs). If a rule has no `when` clause, it applies in all contexts, but still respects glob patterns for file matching.

- b09f4b8: **BREAKING:** Removed legacy CLI flags `--env`, `--role`, and `--language` from `cagents build` command.

  **Migration:** Use config variables instead of CLI flags for conditional rule matching.

  Before:

  ```bash
  cagents build --env production --language rust
  ```

  After:

  ```toml
  # .cAGENTS/config.toml
  [variables.static]
  app_env = "production"
  language = "rust"

  # Or use dynamic variables
  [variables.command]
  app_env = "echo $APP_ENV"
  ```

  Then use in template frontmatter:

  ```yaml
  ---
  when:
    app_env: ["production"]
    language: ["rust"]
  ---
  ```

  **Why:** Config-based variables provide more flexibility and eliminate the need for CLI-specific flags. Any variable can now be used in `when` clauses, not just the predefined env/role/language trio.

- b09f4b8: **BREAKING:** Removed `simplifyGlobsToParent` field from template frontmatter. Use the new `outputIn` field instead for clearer glob output semantics.

  **Migration:**

  - Replace `simplifyGlobsToParent: true` with `outputIn: common-parent`
  - Replace `simplifyGlobsToParent: false` with `outputIn: parent`
  - If omitted, defaults to `common-parent` (same as old default)

  **New `outputIn` values:**

  - `common-parent`: Find common parent directory, create single output there
  - `parent`: Create output in parent directory of each matched file
  - `matched`: Create output IN matched directories (for directory globs with trailing slash)

  This change makes glob output behavior more intuitive and explicit.

- b09f4b8: Simplified and updated README documentation. Removed outdated examples, focused on getting started, configuration, and command reference. All information is now accurate and reflects the actual implementation.

## 0.3.0

### Minor Changes

- cd28fdf: fix package not executable

## 0.2.0

### Minor Changes

- 675096e: release

### Patch Changes

- ea609fe: fix tests
