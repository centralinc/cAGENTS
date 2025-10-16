---
"cagents": patch
---

Fix platform-specific binary distribution to prevent "cannot execute binary file" errors

**Changes:**
- Build cross-platform binaries (Linux x64, macOS x64/ARM64, Windows x64) in CI via GitHub Actions matrix
- Update postinstall script to select the correct binary for the current platform
- Include all platform binaries in npm package (selected at install time)
- Add integration tests to verify binary works after publish and prevent regression
- Fix crates.io publish error by adding required description and license metadata to cagents-core

**Migration:**
No user action required. The correct binary will be automatically selected during `npm install`.

**Related:**
- Fixes issue where Linux binary from CI couldn't run on macOS/Windows
- Prevents "cannot execute binary file" and "unsupported platform" errors
