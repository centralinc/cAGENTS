---
name: backend-rust
description: Backend-specific Rust guidelines
when:
  role: ["backend", "fullstack"]
  language: ["rust"]
order: 20
---
## Backend Rust Guidelines

**Error Handling:**
- Use `anyhow::Result` for application errors
- Use `thiserror` for library error types
- Always add context with `.context()` or `.with_context()`

**Async:**
- Prefer `tokio` runtime
- Use `#[tokio::main]` or `#[tokio::test]`

**Database:**
- Use connection pooling (sqlx, deadpool)
- Always use prepared statements
- Handle transaction rollbacks properly
