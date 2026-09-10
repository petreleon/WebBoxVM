"""Strict JSON and bounded primitives for proof-lineage records."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path, PurePosixPath

from lineage_model import MAX_INPUT_BYTES, reject

HERE = Path(__file__).resolve().parent
if str(HERE.parent) not in sys.path: sys.path.insert(0, str(HERE.parent))

from vulkan_docs_member_ids import member_id as stable_member_id

IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9-]*$")
DIGEST = re.compile(r"^[0-9a-f]{64}$")


def pairs(rows):
    result = dict(rows)
    if len(result) != len(rows):
        reject("lineage JSON has duplicate object fields")
    return result


def nonfinite(value: str):
    reject(f"lineage JSON has a non-finite value: {value}")


def document(path: Path) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=pairs, parse_constant=nonfinite)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"lineage JSON cannot be read: {error}")


def canonical(value: object, domain: str) -> str:
    try:
        encoded = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False)
    except (TypeError, ValueError) as error:
        reject(f"lineage JSON cannot be canonicalized: {error}")
    return hashlib.sha256(domain.encode("ascii") + b"\0" + encoded.encode("utf-8")).hexdigest()


def exact(value: object, fields: frozenset[str], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != fields:
        reject(f"{label} has an invalid schema")
    return value


def text(value: object, label: str) -> str:
    if not isinstance(value, str) or not value:
        reject(f"{label} must be a nonempty string")
    return value


def identifier(value: object, label: str) -> str:
    result = text(value, label)
    if not IDENTIFIER.fullmatch(result):
        reject(f"{label} has an invalid identifier")
    return result


def digest(value: object, label: str) -> str:
    result = text(value, label)
    if not DIGEST.fullmatch(result) or result == "0" * 64:
        reject(f"{label} has an invalid SHA-256")
    return result


def positive(value: object, label: str, maximum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or not 0 < value <= maximum:
        reject(f"{label} is outside its explicit bound")
    return value


def member_bytes(value: object, label: str) -> int:
    return positive(value, label, MAX_INPUT_BYTES)


def selector(value: object, label: str) -> str:
    result = text(value, label)
    item = PurePosixPath(result)
    if (not result.isascii() or "\x00" in result or "\\" in result or item.is_absolute() or str(item) != result
            or not item.name or any(part in (".", "..") for part in item.parts)):
        reject(f"{label} is not a safe relative selector")
    return result


def trace_path(value: object, label: str) -> str:
    result = selector(value, label)
    allowed = result.startswith("temporary/") or result.startswith("generated/")
    if not allowed or result == "generated/out" or result.startswith("generated/out/"):
        reject(f"{label} escapes the proof-only generated/temporary roots")
    return result


def derived_selector(value: object, label: str) -> str:
    result = trace_path(value, label)
    if not result.startswith("generated/"):
        reject(f"{label} is not a final generated selector")
    return result


def raw_selector(value: object, label: str) -> str:
    result = selector(value, label)
    if result.startswith("generated/") or result.startswith("temporary/"):
        reject(f"{label} is not a source selector")
    return result


def canonical_member_id(source: object, value: object, selected: str, label: str) -> str:
    kind = {"raw": "raw-source-input", "derived": "derived-source-input"}.get(source)
    if kind is None: reject(f"{label} has an invalid source kind")
    result = identifier(value, f"{label} input id")
    try: expected = stable_member_id(kind, selected)
    except ValueError as error: reject(str(error))
    if result != expected: reject(f"{label} input id is not the canonical selector identity")
    return result
