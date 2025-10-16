---
"cagents": patch
---

Added arbitrary variables in `when` clauses. You can now use any variable from config in conditional rules.

- Define variables in config: `[variables.command] app_env = "echo $APP_ENV"`
- Use in when clauses: `when: { app_env: ["production", "staging"] }`
- Config variables are evaluated and available in `when` conditionals
- Use config variables for flexible conditional rule matching
