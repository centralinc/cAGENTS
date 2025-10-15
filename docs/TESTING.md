# Testing Guide

## Running Tests

### Run all tests
```bash
cargo test --workspace --locked
```

### Run with output
```bash
cargo test --workspace --locked -- --nocapture
```

### Run specific test
```bash
cargo test --package cagents-core --test cross_platform
```

## Cross-Platform Testing

Tests must work on Windows, macOS, and Linux. We use CI to verify this.

### Path Handling in Tests

**DO NOT** manually construct file paths in config strings:
```rust
// ❌ WRONG - Breaks on Windows
let config = format!(
    r#"engine = "command:python3 {}""#,
    script.path().display()
);
```

**DO** use the test utilities:
```rust
// ✅ CORRECT - Works on all platforms
use test_utils::format_toml_command;

let config_value = format_toml_command("python3", script.path());
let config = format!(r#"engine = {}"#, config_value);
```

See [test_utils documentation](../crates/cagents-core/tests/test_utils/README.md) for details.

## Linting

### Run Clippy
```bash
cargo clippy --workspace --all-targets --locked -- -D warnings
```

This enforces:
- No warnings allowed (fails CI)
- Cross-platform best practices
- Code quality standards

### Configuration

Clippy is configured in [`.clippy.toml`](../.clippy.toml) to catch common issues.

## Common Issues

### Windows Path Failures

**Symptoms**: Tests pass locally (Unix) but fail on Windows CI with:
- `invalid unicode 8-digit hex code`
- `No such file or directory`
- TOML/YAML parse errors

**Cause**: Backslashes in Windows paths are treated as escape sequences

**Fix**: Use `test_utils::path_to_command()` or related utilities

### Serial Test Ordering

Some tests use `#[serial]` from the `serial_test` crate because they:
- Change working directory
- Depend on global state

Keep these to a minimum and document why they're serial.

## CI Configuration

Tests run on:
- Ubuntu (Linux)
- macOS (latest)
- Windows (latest)

All must pass before merge.

### Debug CI Failures

1. Check the specific platform in CI logs
2. Look for path-related errors
3. Verify you're using test utilities
4. Test locally with path simulation:
   ```rust
   let windows_path = PathBuf::from("C:\\Users\\test\\file.py");
   ```

## Performance

- Tests should complete in < 30 seconds total
- Use `TempDir` for isolation
- Clean up resources in test teardown
