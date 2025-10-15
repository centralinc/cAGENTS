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
- Support for arbitrary variables in `when` clauses (complements the removal of `alwaysApply`)

### Fixed
- Output files (CLAUDE.md, .cursorrules) are now properly cleaned up when removed from `[output] targets` config
  - Previously only AGENTS.md files in different directories were cleaned up
  - Now tracks both output directories and target formats in `.cAGENTS/.output-cache`
  - Automatically removes orphaned output files on next build

## [0.3.0] - Previous release

(Add previous version history here as needed)
