"""Readable source-inventory JSON with compact rows and deterministic hashes."""

import json

from .source import reject


def metadata_json(value, depth=0):
    """Keep short metadata records readable without expanding every scalar row."""
    inline = json.dumps(value, sort_keys=True)
    if len(inline) + 2 * depth <= 120 or not isinstance(value, (list, dict)):
        return inline
    pairs = sorted(value.items()) if isinstance(value, dict) else list(enumerate(value))
    left, right = ("{", "}") if isinstance(value, dict) else ("[", "]")
    lines = [left]
    for position, (key, item) in enumerate(pairs):
        prefix = json.dumps(key) + ": " if isinstance(value, dict) else ""
        rendered = (prefix + metadata_json(item, depth + 1)).splitlines()
        chunk = ["  " + line for line in rendered]
        chunk[-1] += "," if position + 1 < len(pairs) else ""
        lines.extend(chunk)
    return "\n".join((*lines, right))


def pretty_json(value: dict[str, object]) -> str:
    lines, pairs = ["{"], sorted(value.items())
    custom = len(value.get("raw_entry_fields", [])) == 9
    for position, (key, item) in enumerate(pairs):
        comma = "," if position + 1 < len(pairs) else ""
        if key != "raw_entries":
            encoded = metadata_json(item) if custom else json.dumps(item, indent=2, sort_keys=True)
            rendered = f"{json.dumps(key)}: {encoded}"
            chunk = [f"  {line}" for line in rendered.splitlines()]
            chunk[-1] += comma
            lines.extend(chunk)
            continue
        fields = value.get("raw_entry_fields", [None] * 7)
        if not isinstance(item, list) or not all(isinstance(row, list) and len(row) == len(fields) for row in item):
            reject("inventory rows cannot be rendered")
        lines.append('  "raw_entries": [')
        for number, row in enumerate(item):
            tail = "," if number + 1 < len(item) else ""
            if len(row) == 9 and isinstance(row[5], list):
                lines.extend((f"    [{json.dumps(row[0])}, {json.dumps(row[1])}, {json.dumps(row[2])},",
                              f"     {json.dumps(row[3])}, {json.dumps(row[4])}, ["))
                lines.extend(f"       {json.dumps(part)}{',' if n + 1 < len(row[5]) else ''}" for n, part in enumerate(row[5]))
                lines.append(f"     ], {json.dumps(row[6])}, {json.dumps(row[7])}, {json.dumps(row[8])}]{tail}")
            else:
                lines.extend((f"    [{json.dumps(row[0])}, {json.dumps(row[1])}, {json.dumps(row[2])},",
                              f"     {json.dumps(row[3])},",
                              f"     {', '.join(json.dumps(part) for part in row[4:])}]{tail}"))
        lines.append(f"  ]{comma}")
    return "\n".join((*lines, "}"))
