"""Readable source-inventory JSON with compact rows and deterministic hashes."""

import json

from .source import reject


def pretty_json(value: dict[str, object]) -> str:
    lines, pairs = ["{"], sorted(value.items())
    for position, (key, item) in enumerate(pairs):
        comma = "," if position + 1 < len(pairs) else ""
        if key != "raw_entries":
            rendered = f"{json.dumps(key)}: {json.dumps(item, indent=2, sort_keys=True)}"
            chunk = [f"  {line}" for line in rendered.splitlines()]
            chunk[-1] += comma
            lines.extend(chunk)
            continue
        if not isinstance(item, list) or not all(isinstance(row, list) and len(row) == 7 for row in item):
            reject("inventory rows cannot be rendered")
        lines.append('  "raw_entries": [')
        for number, row in enumerate(item):
            tail = "," if number + 1 < len(item) else ""
            lines.extend((f"    [{json.dumps(row[0])}, {json.dumps(row[1])}, {json.dumps(row[2])},",
                          f"     {json.dumps(row[3])},",
                          f"     {json.dumps(row[4])}, {json.dumps(row[5])}, {json.dumps(row[6])}]{tail}"))
        lines.append(f"  ]{comma}")
    return "\n".join((*lines, "}"))
