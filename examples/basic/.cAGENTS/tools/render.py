#!/usr/bin/env python3
"""Minimal BYOB renderer for cAGENTS examples.

Supports:
- {{variable}} substitution using payload.data
- {{#if variable}} ... {{/if}} blocks

The renderer intentionally implements only the features used by example templates.
"""

import json
import re
import sys
from typing import Any


def _truthy(value: Any) -> bool:
    if value is None:
        return False
    if isinstance(value, bool):
        return value
    if isinstance(value, (list, dict, set, tuple)):
        return len(value) > 0
    if isinstance(value, (int, float)):
        return value != 0
    if isinstance(value, str):
        return value != ""
    return True


def _stringify(value: Any) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    if value is None:
        return ""
    return str(value)


def render(template: str, data: dict[str, Any]) -> str:
    # Handle simple {{#if key}} ... {{/if}} blocks
    if_pattern = re.compile(r"{{#if ([^}]+)}}(.*?){{/if}}", re.DOTALL)
    while True:
        match = if_pattern.search(template)
        if not match:
            break
        key = match.group(1).strip()
        block = match.group(2)
        value = data.get(key)
        replacement = block if _truthy(value) else ""
        template = template[: match.start()] + replacement + template[match.end():]

    # Replace simple variables
    def _replace_var(match: re.Match[str]) -> str:
        key = match.group(1).strip()
        return _stringify(data.get(key))

    return re.sub(r"{{([^#/][^}]*)}}", _replace_var, template)


def main() -> None:
    payload = json.load(sys.stdin)
    template = payload.get("templateSource", "")
    data = payload.get("data", {}) or {}
    frontmatter = payload.get("frontmatter", {}) or {}

    # Merge frontmatter.vars into data so templates can reference them
    fm_vars = frontmatter.get("vars")
    if isinstance(fm_vars, dict):
        merged = dict(data)
        merged.update(fm_vars)
        data = merged

    content = render(template, data)
    json.dump({"content": content}, sys.stdout, ensure_ascii=False)


if __name__ == "__main__":
    main()
