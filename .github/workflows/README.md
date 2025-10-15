# GitHub Actions Workflows

This directory contains automated workflows for CI/CD.

## Workflows

### CI Workflow (`ci.yml`)

Runs on every push to `main` and on all pull requests.

**What it does:**
- Builds and tests on Ubuntu, macOS, and Windows
- Runs clippy with strict warnings (`-D warnings`)
- Builds Node.js wrapper packages

**Performance optimizations:**
- ✅ **Rust caching** - Caches Cargo dependencies and build artifacts (saves 2-3 minutes)
- ✅ **pnpm caching** - Caches npm packages for faster installs
- ✅ **Parallel matrix builds** - Tests run on all platforms simultaneously

**Debugging features:**
- `RUST_BACKTRACE=1` - Full stack traces on test failures
- `CARGO_TERM_COLOR=always` - Colored output for readability
- `--nocapture` - Shows all test output including `eprintln!`

### Release Workflow (`release.yml`)

Runs on every push to `main`. Handles versioning and publishing.

**What it does:**
1. **Version Phase** - Creates a PR with version bumps using changesets
2. **Publish Phase** - When PR is merged, publishes packages:
   - Publishes npm packages via `pnpm release`
   - Publishes to crates.io (`cagents-core` → `cagents-cli`)
   - Creates GitHub release with changelog

**Required Secrets:**
- `GITHUB_TOKEN` - Auto-provided by GitHub Actions
- `NPM_TOKEN` - NPM authentication token (for publishing)
- `CARGO_TOKEN` - Crates.io API token (for publishing)

**Permissions Required:**
```yaml
permissions:
  contents: write       # Push commits, create releases
  pull-requests: write  # Create/update version PR
  issues: write         # Comment on issues
```

**Performance optimizations:**
- ✅ **Rust caching** - Speeds up cargo builds before publishing
- ✅ **pnpm caching** - Faster dependency installation

## Common Issues

### "Resource not accessible by integration" Error

**Cause:** Missing permissions in workflow

**Fix:** Ensure the `permissions` block is present:
```yaml
jobs:
  release:
    permissions:
      contents: write
      pull-requests: write
```

### Caching Not Working

**Symptoms:** Every CI run reinstalls all dependencies

**Possible causes:**
1. Lock file changed (`pnpm-lock.yaml` or `Cargo.lock`)
2. Cache key changed (runner OS or dependencies updated)
3. Cache expired (GitHub caches expire after 7 days of no access)

**Verification:**
Check CI logs for:
- `Cache restored successfully from key: ...` ✅ Working
- `Cache not found for input keys: ...` ❌ Not working

### Tests Failing on Windows Only

**Common issues:**
1. **Path separators** - Use both `/` and `\` in assertions
2. **MAX_PATH limits** - Windows has 260 char path limit
3. **Line endings** - Git may convert CRLF/LF

**Debugging:**
- Check CI logs with `RUST_BACKTRACE=1` output
- Look for test diagnostic output (eprintln!)
- Run tests locally with `--nocapture`

## Updating Workflows

### Adding New Platforms

To test on additional platforms, update the matrix:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest, macos-13] # Add here
```

### Changing Rust Version

```yaml
- uses: dtolnay/rust-toolchain@stable  # or @nightly, @1.75, etc
```

### Updating Cache Configuration

Rust cache is automatic via `Swatinem/rust-cache@v2`.

For pnpm, update the cache key:
```yaml
key: ${{ runner.os }}-pnpm-store-${{ hashFiles('**/pnpm-lock.yaml') }}
```

## Best Practices

1. **Always use caching** for dependencies (Cargo, pnpm)
2. **Enable backtraces** for better debugging (`RUST_BACKTRACE=1`)
3. **Use explicit permissions** - Don't rely on defaults
4. **Test locally first** - Use `act` or local builds before pushing
5. **Keep workflows fast** - Target < 5 minutes for CI

## Monitoring

- **CI status**: Check the Actions tab in GitHub
- **Cache usage**: Settings → Actions → Caches
- **Workflow runs**: Actions → Select workflow → View runs

## Documentation

- [GitHub Actions Docs](https://docs.github.com/actions)
- [Changesets Action](https://github.com/changesets/action)
- [Rust Cache Action](https://github.com/Swatinem/rust-cache)
- [pnpm Action](https://github.com/pnpm/action-setup)
