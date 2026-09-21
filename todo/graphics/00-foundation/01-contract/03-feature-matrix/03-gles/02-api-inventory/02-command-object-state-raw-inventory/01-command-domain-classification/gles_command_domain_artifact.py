"""Strict JSON, hash, and no-promotion helpers for F03.3.2.2.1 artifacts."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

MAX_SERIALIZED = 1024 * 1024
FORBIDDEN = ("api", "matrix", "cts", "support", "conformance", "certification", "performance", "guest", "browser", "owner", "test", "evidence")


class ArtifactError(ValueError):
    """A map fragment is malformed, promoted, or no longer immutable."""


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def exact(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return set(left) == set(right) and all(exact(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(exact(a, b) for a, b in zip(left, right))
    return left == right


def pairs(items):
    value = {}
    for key, item in items:
        if key in value:
            raise ArtifactError("source-family JSON has duplicate fields")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    if path.is_symlink() or not path.is_file():
        raise ArtifactError("source-family artifact must be a regular file")
    try:
        raw = path.read_bytes()
        if len(raw) > MAX_SERIALIZED:
            raise ArtifactError("source-family artifact exceeds its serialized-size cap")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ArtifactError(f"source-family artifact cannot be read: {error}") from error
    if not isinstance(value, dict):
        raise ArtifactError("source-family artifact is not a JSON object")
    return value


def self_hashed(value: dict[str, object], field: str) -> None:
    body = {key: item for key, item in value.items() if key != field}
    if value.get(field) != hashlib.sha256(canonical(body)).hexdigest():
        raise ArtifactError("source-family artifact has a stale self hash")


def forbidden(value: object) -> None:
    if isinstance(value, dict):
        for key, item in value.items():
            if any(word in key.lower() for word in FORBIDDEN):
                raise ArtifactError("source-family artifact contains a prohibited qualification field")
            forbidden(item)
    elif isinstance(value, list):
        for item in value:
            forbidden(item)
