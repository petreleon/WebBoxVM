"""Small JSON and shape helpers for F03.2.2.5.4.1."""

from __future__ import annotations

import json
from pathlib import Path


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def pairs(items, reject):
    value = {}
    for key, item in items:
        if key in value:
            reject("inventory artifact has duplicate JSON fields")
        value[key] = item
    return value


def document(path: Path, maximum: int, reject) -> dict[str, object]:
    try:
        raw = path.read_bytes()
        if len(raw) > maximum:
            reject("inventory artifact exceeds the serialized-size cap")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=lambda items: pairs(items, reject))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"inventory artifact cannot be read: {error}")
    if not isinstance(value, dict):
        reject("inventory artifact is not a JSON object")
    return value


def fences(value: dict[str, object], top_level, forbidden, rules, claims, reject) -> None:
    rows, pages = value.get("declarations"), value.get("source_pages")
    page_shape = {"physical_page", "section", "declaration_ids", "section_witness_physical_page"}
    if (set(value) != top_level or not isinstance(rows, list) or not isinstance(pages, list)
            or forbidden & set(value) or any(not isinstance(row, dict) or forbidden & set(row) for row in rows)):
        reject("inventory artifact has a forbidden promotion or an unclosed shape")
    if (len(rows) != 6 or len(pages) != 6 or [row.get("source_order") for row in rows] != list(range(1, 7))
            or [row.get("id") for row in rows] != [item[0] for item in rules.DECLARATIONS]
            or any(not isinstance(page, dict) or set(page) != page_shape for page in pages)):
        reject("inventory artifact omits, duplicates, or extends its fixed source declarations")
    if value.get("claims") != claims or value.get("cts_executions") != 0 or value.get("matrix_row_count") != 0:
        reject("inventory artifact promotes claims, CTS, or Matrix rows")
