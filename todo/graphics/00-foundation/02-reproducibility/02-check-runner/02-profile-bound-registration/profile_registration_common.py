"""Shared strict JSON and generic-runner field validation for F05.2."""

from __future__ import annotations

import copy
import json
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[5]


class RegistrationError(ValueError):
    """The registration is stale, qualifying, or not runnable through F05.1."""


def reject(message: str) -> None:
    raise RegistrationError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def pairs(items: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in items:
        if key in value:
            reject(f"duplicate JSON key {key!r}")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, RegistrationError) as error:
        reject(f"registration catalog cannot be read: {error}")
    if not isinstance(value, dict):
        reject("registration catalog is not an object")
    return value


def text(value: object, label: str) -> str:
    if not isinstance(value, str) or not value.strip() or "\x00" in value:
        reject(f"{label} must be nonempty text")
    return value


def relative(value: object, label: str, exists: bool = False) -> str:
    value = text(value, label)
    path = Path(value)
    if path.is_absolute() or path == Path(".") or ".." in path.parts:
        reject(f"{label} must be a repository-relative path")
    if exists and not (REPO / path).is_file():
        reject(f"{label} is missing")
    return value


def argv(value: object, label: str) -> list[str]:
    if not isinstance(value, list) or not value:
        reject(f"{label} must be a nonempty argument array")
    return [text(item, f"{label} argument") for item in value]


def prerequisites(value: object) -> list[dict[str, object]]:
    if not isinstance(value, list) or not value:
        reject("prerequisites must be a nonempty array")
    allowed = {"executable", "asset", "browser", "hardware", "permission"}
    result: list[dict[str, object]] = []
    for item in value:
        if not isinstance(item, dict) or item.get("kind") not in allowed:
            reject("prerequisite kind is unsupported")
        kind, fields = item["kind"], {"kind", "value"}
        if kind == "permission":
            fields.add("access")
        if set(item) != fields:
            reject("prerequisite fields do not match the generic runner schema")
        path_value = relative(item["value"], "path prerequisite") if kind in {"asset", "permission"} else text(item["value"], "prerequisite")
        if kind == "hardware" and not re.fullmatch(r"[A-Z][A-Z0-9_]*", path_value):
            reject("hardware prerequisite must be an environment variable")
        if kind == "permission" and item.get("access") not in {"read", "write", "execute"}:
            reject("permission prerequisite access is unsupported")
        result.append(copy.deepcopy(item))
    return result


def no_claims(value: object, claims: frozenset[str]) -> bool:
    return isinstance(value, dict) and set(value) == claims and all(item is False for item in value.values())
