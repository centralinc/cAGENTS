# cagents

## 0.4.1

### Patch Changes

- 686ecb1: Fix nested file detection in multi-format migration. When using 'merge all formats' option, nested AGENTS.md and CLAUDE.md files are now properly discovered and imported, matching the behavior of single-format migration.
- 4c77de2: fix deploy

## 0.4.0

### Minor Changes

- e030236: Version reset and workflow improvements. This release includes:

  - Reset version to 0.4.0 following proper semver from 0.3.3
  - Fixed canary releases to include correct version in compiled binaries
  - Improved GitHub workflows to use centralbot with proper permissions
  - Enhanced version syncing between npm package and Rust binaries
  - All future canary and stable releases will show correct version in `--version` output

## 0.1.0

### Major Changes

- Version reset to 0.1.0 for alpha release cycle. Configured changesets for beta/RC releases.
