# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **BREAKING:** Removed `alwaysApply` field from template frontmatter. Rules without a `when` clause now implicitly apply in all contexts. This simplifies the API by removing redundancy.
  - **Migration:** Remove `alwaysApply: true` from all templates. Rules without `when` clauses will automatically apply everywhere.
  - **Behavior:** Rules are now filtered only by context (`when` clause) and file matching (globs). If a rule has no `when` clause, it applies in all contexts, but still respects glob patterns for file matching.

### Added
- **Arbitrary variables in `when` clauses:** You can now use any variable from config in conditional rules, not just the legacy `env`/`role`/`language` fields
  - Define variables in config: `[variables.command] app_env = "echo $APP_ENV"`
  - Use in when clauses: `when: { app_env: ["production", "staging"] }`
  - Legacy CLI flags (`--env`, `--role`, `--language`) still work but are now optional
  - Config variables are evaluated and available in `when` conditionals
  - Migration: Start using config variables instead of CLI flags for more flexibility
- **New `outputIn` field for glob output control:** More intuitive control over where AGENTS.md files are created
  - `outputIn: matched` - Create output IN matched directories (for dir globs with trailing slash)
  - `outputIn: parent` - Create in parent of matched files (one per directory)
  - `outputIn: common-parent` - Find common parent, create single output there
  - Backward compatible: `simplifyGlobsToParent` still works (maps to outputIn values)
  - Directory glob support: Patterns ending with `/` now match directories
- **cAGENTS usage footer:** All generated files now include a footer with best practices
  - Explains `cagents build` and `cagents context` workflow
  - Helps AI agents work more efficiently with cAGENTS-managed codebases

### Fixed
- Output files (CLAUDE.md, .cursorrules) are now properly cleaned up when removed from `[output] targets` config
  - Previously only AGENTS.md files in different directories were cleaned up
  - Now tracks both output directories and target formats in `.cAGENTS/.output-cache`
  - Automatically removes orphaned output files on next build

## [0.3.0] - Previous release

(Add previous version history here as needed)
