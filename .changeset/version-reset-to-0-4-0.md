---
"cagents": minor
---

Version reset and workflow improvements. This release includes:

- Reset version to 0.4.0 following proper semver from 0.3.3
- Fixed canary releases to include correct version in compiled binaries
- Improved GitHub workflows to use centralbot with proper permissions
- Enhanced version syncing between npm package and Rust binaries
- All future canary and stable releases will show correct version in `--version` output
