---
"cagents": patch
---
  1. Changed Linux build target from x86_64-unknown-linux-gnu (dynamic GLIBC) to x86_64-unknown-linux-musl (static musl) in both workflows
  2. Added musl-tools installation step before building the Linux binary
  3. Updated artifact paths to match the new target name
