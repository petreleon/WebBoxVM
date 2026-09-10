"""Render capture metadata compactly while keeping each closure member inspectable."""

from __future__ import annotations

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))
from vcts_capture_input import reject


def render(value: dict[str, object]) -> str:
    if not isinstance(value, dict):
        reject("JSON writer requires an object")
    members = value.get("members")
    if not isinstance(members, list):
        return json.dumps(value, indent=2, sort_keys=True) + "\n"
    lines = ["{"]
    for key in sorted(item for item in value if item != "members"):
        lines.append(f"  {json.dumps(key)}: {json.dumps(value[key], sort_keys=True)},")
    lines.append('  "members": [')
    lines.extend("    " + json.dumps(item, sort_keys=True, separators=(",", ":")) + ("," if index + 1 < len(members) else "")
                 for index, item in enumerate(members))
    return "\n".join((*lines, "  ]", "}")) + "\n"


def write_json(path: Path, value: dict[str, object]) -> None:
    path.write_text(render(value), encoding="utf-8")
