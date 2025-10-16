---
"cagents": patch
---

**BREAKING:** Removed legacy CLI flags `--env`, `--role`, and `--language` from `cagents build` command.

**Migration:** Use config variables instead of CLI flags for conditional rule matching.

Before:
```bash
cagents build --env production --language rust
```

After:
```toml
# .cAGENTS/config.toml
[variables.static]
app_env = "production"
language = "rust"

# Or use dynamic variables
[variables.command]
app_env = "echo $APP_ENV"
```

Then use in template frontmatter:
```yaml
---
when:
  app_env: ["production"]
  language: ["rust"]
---
```

**Why:** Config-based variables provide more flexibility and eliminate the need for CLI-specific flags. Any variable can now be used in `when` clauses, not just the predefined env/role/language trio.
