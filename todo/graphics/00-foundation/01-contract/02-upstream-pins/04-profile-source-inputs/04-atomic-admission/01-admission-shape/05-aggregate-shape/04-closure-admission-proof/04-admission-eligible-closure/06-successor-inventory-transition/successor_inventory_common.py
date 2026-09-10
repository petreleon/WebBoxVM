"""Small fail-closed JSON helpers shared by the design and candidate contracts."""

from __future__ import annotations

import hashlib
import json
import os
import re
import stat
from pathlib import Path

MAX_DOCUMENT_BYTES = 64 * 1024
DIGEST = re.compile(r"^[0-9a-f]{64}$")


class ContractError(ValueError):
    """A bounded contract document cannot be trusted."""


def reject(message: str) -> None:
    raise ContractError(message)


def pairs(items: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in items:
        if key in result:
            reject(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


def bounded_bytes(path: Path, label: str, limit: int = MAX_DOCUMENT_BYTES) -> bytes:
    try:
        if not stat.S_ISREG(path.lstat().st_mode):
            reject(f"{label} is not a regular file")
        descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(descriptor, "rb") as source:
            if not stat.S_ISREG(os.fstat(descriptor).st_mode):
                reject(f"{label} is not a regular file")
            raw = source.read(limit + 1)
    except OSError as error:
        reject(f"{label} cannot be read: {error}")
    if len(raw) > limit:
        reject(f"{label} exceeds its bounded size")
    return raw


def decoded(raw: bytes, label: str) -> dict[str, object]:
    try:
        value = json.loads(raw, object_pairs_hook=pairs)
    except (UnicodeDecodeError, json.JSONDecodeError, ContractError) as error:
        reject(f"{label} is invalid: {error}")
    if not isinstance(value, dict):
        reject(f"{label} must be an object")
    return value


def document(path: Path, label: str = "contract", limit: int = MAX_DOCUMENT_BYTES) -> dict[str, object]:
    return decoded(bounded_bytes(path, label, limit), label)


def exact(value: object, fields: frozenset[str], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != fields:
        reject(f"{label} has an invalid schema")
    return value


def digest(value: object, label: str) -> str:
    if not isinstance(value, str) or not DIGEST.fullmatch(value) or value == "0" * 64:
        reject(f"{label} has an invalid sha256")
    return value


def fixed(value: dict[str, object], expected: tuple[tuple[str, object, type], ...]) -> bool:
    return all(type(value.get(key)) is kind and value[key] == item for key, item, kind in expected)


def canonical(value: dict[str, object], hash_field: str) -> bytes:
    return json.dumps({key: item for key, item in value.items() if key != hash_field},
                      sort_keys=True, separators=(",", ":")).encode()


def anchored(path: Path, expected: str, label: str) -> dict[str, object]:
    raw = bounded_bytes(path, label)
    if hashlib.sha256(raw).hexdigest() != expected:
        reject(f"{label} no longer has its reviewed byte identity")
    return decoded(raw, label)
