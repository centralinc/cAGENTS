---
name: rust-specific
description: Rust-specific rules for Rust source files
globs:
  - "**/*.rs"
order: 15
---
## Rust Code Guidelines

**For this directory:**

- Use `rustfmt` before committing
- Run `cargo clippy` and fix all warnings
- Add doc comments (`///`) for public items
- Use `#[must_use]` for functions that return important values
