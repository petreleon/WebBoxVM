#!/usr/bin/env python3
"""Streaming evidence checks for F02.5 content-addressed role artifacts."""

from __future__ import annotations

import hashlib
from pathlib import Path

from source_role_contract import validate_catalog
from source_role_records import artifact, reject

CHUNK_BYTES = 1024 * 1024


def root_path(value: object) -> Path:
    if not isinstance(value, Path) or not value.is_absolute() or value.is_symlink():
        reject("artifact root must be an absolute path")
    root = value.resolve(strict=False)
    if not root.is_dir():
        reject("artifact root must be an existing directory")
    return root


def target(root: Path, key: object, name: str) -> Path:
    key = artifact(key, name)
    current = root
    for part in Path(key).parts:
        current /= part
        if current.is_symlink():
            reject(f"{name} cannot be a symlink")
    try:
        path = (root / key).resolve(strict=True)
    except OSError:
        reject(f"{name} cannot be resolved")
    try:
        path.relative_to(root)
    except ValueError:
        reject(f"{name} escapes the artifact root")
    if not path.is_file():
        reject(f"{name} is not a regular file")
    return path


def digest_file(path: Path) -> tuple[int, str]:
    total, hasher = 0, hashlib.sha256()
    with path.open("rb") as handle:
        while chunk := handle.read(CHUNK_BYTES):
            total += len(chunk)
            hasher.update(chunk)
    return total, hasher.hexdigest()


def verify_file(root: Path, record: dict[str, object]) -> Path:
    path = target(root, record["artifact"], f"{record['id']} artifact")
    if digest_file(path) != (record["bytes"], record["sha256"]):
        reject(f"{record['id']} artifact does not match declared bytes and SHA-256")
    return path


def verify_builder(root: Path, record: dict[str, object]) -> None:
    builder = record["builder"]
    path = target(root, builder["artifact"], "builder artifact")
    if digest_file(path)[1] != builder["sha256"]:
        reject("builder artifact does not match declared SHA-256")


def verify_shards(records: dict[str, dict[str, object]], paths: dict[str, Path]) -> None:
    groups: dict[str, list[dict[str, object]]] = {}
    for record in records.values():
        if record["kind"] == "webboxvm-transform" and record["scope"] == "webboxvm-byte-preserving-shard":
            groups.setdefault(record["shard"]["source_id"], []).append(record)
    for source_id, pieces in groups.items():
        hasher, total = hashlib.sha256(), 0
        for piece in pieces:
            with paths[piece["id"]].open("rb") as handle:
                while chunk := handle.read(CHUNK_BYTES):
                    total += len(chunk)
                    hasher.update(chunk)
        source = records[source_id]
        if (total, hasher.hexdigest()) != (source["bytes"], source["sha256"]):
            reject("shard bytes do not reassemble to their upstream suite member")


def verify_catalog(value: object, artifact_root: Path) -> tuple[str, ...]:
    identifiers = validate_catalog(value)
    root = root_path(artifact_root)
    records = {record["id"]: record for record in value["records"]}
    paths = {identifier: verify_file(root, record) for identifier, record in records.items()}
    for record in records.values():
        if record["kind"] == "webboxvm-transform":
            verify_builder(root, record)
    verify_shards(records, paths)
    return identifiers
