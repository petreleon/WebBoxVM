#!/usr/bin/env python3
"""Readable, self-hashed artifact bundle for F03.2.2.5.3.1."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT_NAME = "opengl_generic_object_sync_raw_inventory.json"
FRAGMENTS = (("sync", "opengl_generic_object_sync_raw_sync.json"), ("query-lifecycle", "opengl_generic_object_sync_raw_query_lifecycle.json"),
             ("query-state", "opengl_generic_object_sync_raw_query_state.json"), ("query-buffer-state", "opengl_generic_object_sync_raw_query_buffer_state.json"))
MAX_SERIALIZED = 8 * 1024 * 1024


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def digest(value: object) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def serialized(value: object) -> str:
    return json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + "\n"


def exact(left: object, right: object) -> bool:
    if type(left) is not type(right): return False
    if isinstance(left, dict): return left.keys() == right.keys() and all(exact(left[key], right[key]) for key in left)
    if isinstance(left, list): return len(left) == len(right) and all(exact(a, b) for a, b in zip(left, right))
    return left == right


def pairs(items):
    value = {}
    for key, item in items:
        if key in value: raise ValueError("duplicate JSON field")
        value[key] = item
    return value


def sealed(identifier: str, declarations: list[dict[str, object]]) -> dict[str, object]:
    body = {"schema": 1, "kind": "webboxvm-opengl46-generic-object-sync-raw-fragment", "fragment_id": identifier,
            "declarations": declarations, "declaration_count": len(declarations), "declarations_sha256": digest(declarations)}
    return {**body, "artifact_sha256": digest(body)}


def receipt(identifier: str, filename: str, value: dict[str, object]) -> dict[str, str]:
    return {"fragment_id": identifier, "file": filename, "artifact_sha256": value["artifact_sha256"],
            "serialized_sha256": hashlib.sha256(serialized(value).encode("utf-8")).hexdigest()}


def bundle(core: dict[str, object], declarations: list[dict[str, object]]) -> dict[str, dict[str, object]]:
    groups = {identifier: [row for row in declarations if row["fragment"] == identifier] for identifier, _ in FRAGMENTS}
    if [len(groups[identifier]) for identifier, _ in FRAGMENTS] != [5, 8, 6, 4] or sum(map(len, groups.values())) != 23:
        raise ValueError("closed raw declaration groups changed")
    files = {filename: sealed(identifier, groups[identifier]) for identifier, filename in FRAGMENTS}
    body = {**core, "artifact_receipts": [receipt(identifier, filename, files[filename]) for identifier, filename in FRAGMENTS]}
    files[ROOT_NAME] = {**body, "raw_inventory_sha256": digest(body)}
    return files


def load(path: Path, reject):
    if path.is_symlink() or not path.is_file(): reject("raw artifact must be a regular file")
    try:
        raw = path.read_bytes()
        if len(raw) > MAX_SERIALIZED: reject("raw artifact exceeds its serialized-size cap")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, ValueError, json.JSONDecodeError) as error:
        reject(f"raw artifact cannot be read: {error}")
    if not isinstance(value, dict): reject("raw artifact is not a JSON object")
    return value, hashlib.sha256(raw).hexdigest()


def self_hash(value: dict[str, object], key: str, reject) -> None:
    if value.get(key) != digest({name: item for name, item in value.items() if name != key}):
        reject("raw artifact has a stale self hash")


def materialize(files: dict[str, dict[str, object]]) -> dict[str, object]:
    by_id = {value["fragment_id"]: value for name, value in files.items() if name != ROOT_NAME}
    rows = [row for identifier, _ in FRAGMENTS for row in by_id[identifier]["declarations"]]
    return {**files[ROOT_NAME], "declarations": sorted(rows, key=lambda row: row["source_order"])}


def validate(root_path: Path, expected: dict[str, dict[str, object]], reject) -> dict[str, object]:
    wanted, visible = set(expected), {path.name for path in root_path.parent.glob("opengl_generic_object_sync_raw_*.json")}
    if visible != ((wanted - {ROOT_NAME}) | {root_path.name}): reject("raw artifact set is missing a fragment or contains an extra fragment")
    root, _ = load(root_path, reject); self_hash(root, "raw_inventory_sha256", reject)
    if not exact(root, expected[ROOT_NAME]): reject("raw root artifact is stale, partial, reordered, or promoted")
    receipts = {item.get("file"): item for item in root["artifact_receipts"] if isinstance(item, dict)}
    if set(receipts) != wanted - {ROOT_NAME}: reject("raw root artifact does not bind exact fragment identities")
    actual = {ROOT_NAME: root}
    for name in wanted - {ROOT_NAME}:
        value, raw_sha256 = load(root_path.parent / name, reject); self_hash(value, "artifact_sha256", reject)
        if receipts[name].get("serialized_sha256") != raw_sha256 or not exact(value, expected[name]):
            reject("raw fragment is stale, partial, reordered, or promoted")
        actual[name] = value
    return materialize(actual)
