# Cross-Platform Test Utilities

This module provides utilities for handling file paths in test configurations to ensure tests work correctly across all platforms (Windows, macOS, Linux).

## The Problem

Windows uses backslashes (`\`) in file paths, which causes two critical issues in tests:

1. **Escape Sequence Interpretation**: In TOML/YAML strings, backslashes are escape sequences
   - `C:\Users` becomes invalid unicode error (`\U` is interpreted as unicode escape)

2. **Shell Parsing**: Unquoted paths with spaces or special characters break shell interpretation
   - `C:\Program Files\script.py` is parsed as multiple arguments

## The Solution

This module provides utilities that:
1. Convert backslashes to forward slashes (Windows accepts both)
2. Quote paths to handle spaces and special characters
3. Use appropriate quoting for TOML (triple quotes) and YAML (single quotes)

## Usage

### Basic Command Construction

```rust
use test_utils::path_to_command;

let script_path = temp_dir.path().join("render.py");
let command = path_to_command("python3", &script_path);
// Returns: python3 "/path/to/render.py"
```

### TOML Configuration Files

```rust
use test_utils::format_toml_command;

let script_path = temp_dir.path().join("render.py");
let config_value = format_toml_command("python3", &script_path);

let config = format!(
    r#"
[defaults]
engine = {}
"#,
    config_value
);
// Uses triple-quoted strings: engine = """python3 "/path/to/render.py""""
```

### YAML Frontmatter

```rust
use test_utils::format_yaml_command;

let script_path = temp_dir.path().join("render.py");
let yaml_value = format_yaml_command("python3", &script_path);

let frontmatter = format!(
    "---\nname: test\nengine: {}\n---\n",
    yaml_value
);
// Uses single quotes: engine: 'python3 "/path/to/render.py"'
```

## API Reference

### `path_to_command(command: &str, path: &Path) -> String`

Converts a path to a cross-platform command string with proper quoting.

### `format_toml_command(command: &str, path: &Path) -> String`

Formats a command for use in TOML config files using triple-quoted strings.

### `format_yaml_command(command: &str, path: &Path) -> String`

Formats a command for use in YAML frontmatter using single-quoted strings.

### `path_to_toml_string(path: &Path) -> String`

Converts a path to a TOML-safe triple-quoted string.

### `path_to_yaml_string(path: &Path) -> String`

Converts a path to a YAML-safe single-quoted string.

## Testing

Run the cross-platform compatibility tests:

```bash
cargo test --package cagents-core --test cross_platform
```

## Best Practices

1. **Always use these utilities** when embedding paths in config strings
2. **Never manually escape** backslashes - let the utilities handle it
3. **Quote all paths** even if they don't currently have spaces
4. **Test on Windows CI** to catch platform-specific issues early

## See Also

- [Cross-platform tests](../cross_platform.rs) - Tests verifying correct behavior
- [Clippy configuration](../../../../.clippy.toml) - Lint rules for path handling
