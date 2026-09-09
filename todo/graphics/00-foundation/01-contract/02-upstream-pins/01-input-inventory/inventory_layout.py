"""Fail-closed schema-v1/v2 graphics inventory layout loader."""

from __future__ import annotations

import hashlib
import os
import re
import stat
import tomllib
from dataclasses import dataclass
from pathlib import Path

V1_FIELDS = frozenset(("schema", "cache_root", "cache_note", "required_families", "inputs"))
V2_FIELDS = frozenset(("schema", "cache_root", "cache_note", "required_families", "input_files"))
INPUT_FIELDS = frozenset(("id", "source_family", "immutable_url", "revision", "sha256", "bytes", "license", "local_cache", "generated_code_role", "provenance"))
FAMILIES = frozenset(("linux-uapi", "mesa-virgl", "mesa-venus", "virglrenderer", "venus-protocol", "gl-gles-registry", "glsl", "essl", "vulkan", "spirv", "webgpu", "wgsl", "vk-gl-cts", "webgpu-cts", "piglit"))
PART = re.compile(r"inputs/part-[0-9]{4}\.toml$")
LOCK_HEADER = b"webboxvm-f02-inventory-lock-v2\n"


class InventoryLayoutError(ValueError):
    """The inventory layout cannot safely provide an authoritative input."""


@dataclass(frozen=True)
class Inventory:
    schema: int
    inputs: tuple[dict[str, object], ...]
    revision: str
    revision_path: Path


def reject(message: str) -> None:
    raise InventoryLayoutError(message)


def _root(path: Path) -> Path:
    if not isinstance(path, Path):
        reject("inventory path must be a Path")
    path = path.absolute()
    if path.name != "manifest.toml":
        reject("inventory root must be named manifest.toml")
    return path


def _raw(path: Path) -> bytes:
    try:
        for candidate in (path, *path.parents):
            if candidate.is_symlink():
                reject(f"inventory path has a symlink: {candidate}")
        if not stat.S_ISREG(os.lstat(path).st_mode):
            reject(f"inventory path is not a regular file: {path}")
        return path.read_bytes()
    except OSError as error:
        reject(f"inventory path cannot be read: {error}")


def _toml(raw: bytes, name: str) -> dict[str, object]:
    try:
        document = tomllib.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        reject(f"{name} is not UTF-8 TOML: {error}")
    if not isinstance(document, dict):
        reject(f"{name} must be a TOML table")
    return document


def _metadata(document: dict[str, object], fields: frozenset[str]) -> tuple[str, ...]:
    if set(document) != fields:
        reject("inventory root has unexpected fields")
    if document["cache_root"] != "$XDG_CACHE_HOME" or not isinstance(document["cache_note"], str) or not document["cache_note"]:
        reject("inventory root must require the external cache root")
    families = document["required_families"]
    if not isinstance(families, list) or any(not isinstance(item, str) or not item for item in families):
        reject("inventory root has malformed required_families")
    if len(set(families)) != len(families) or set(families) != FAMILIES:
        reject("inventory root has an incomplete required source-family catalog")
    return tuple(families)


def _inputs(value: object, required: tuple[str, ...]) -> tuple[dict[str, object], ...]:
    if not isinstance(value, list) or not value:
        reject("inventory component has no inputs")
    entries = tuple(value)
    ids, families = set(), set()
    for entry in entries:
        if not isinstance(entry, dict) or set(entry) != INPUT_FIELDS:
            reject("inventory input does not match the maintained schema")
        identifier, family = entry["id"], entry["source_family"]
        if not isinstance(identifier, str) or not identifier or not isinstance(family, str) or not family:
            reject("inventory input has an invalid id or source family")
        if identifier in ids or family in families:
            reject("inventory input has a duplicate id or source family")
        if not isinstance(entry["bytes"], int) or isinstance(entry["bytes"], bool) or entry["bytes"] <= 0:
            reject("inventory input has an invalid byte count")
        if any(not isinstance(entry[field], str) or not entry[field] for field in INPUT_FIELDS - {"id", "source_family", "bytes"}):
            reject("inventory input has an empty required field")
        ids.add(identifier)
        families.add(family)
    if families != set(required) or len(entries) != len(FAMILIES):
        reject("inventory inputs have incomplete source-family coverage")
    return entries


def _parts(document: dict[str, object]) -> tuple[str, ...]:
    names = document["input_files"]
    if not isinstance(names, list) or not names or any(not isinstance(name, str) or not PART.fullmatch(name) for name in names):
        reject("inventory root has unsafe component paths")
    if len(set(names)) != len(names):
        reject("inventory root has duplicate component paths")
    if names != sorted(names):
        reject("inventory root component paths are not sorted")
    expected = [f"inputs/part-{number:04d}.toml" for number in range(1, len(names) + 1)]
    if names != expected:
        reject("inventory root component paths are not contiguous canonical names")
    return tuple(names)


def _v2_parts(path: Path) -> tuple[bytes, tuple[tuple[str, bytes], ...], tuple[dict[str, object], ...]]:
    root = _raw(path)
    document = _toml(root, "inventory root")
    if type(document.get("schema")) is not int or document["schema"] != 2:
        reject("inventory root is not schema version 2")
    required, names = _metadata(document, V2_FIELDS), _parts(document)
    parts, entries = [], []
    for name in names:
        raw = _raw(path.parent / name)
        fragment = _toml(raw, name)
        if set(fragment) != {"inputs"}:
            reject(f"{name} has unexpected fields")
        value = fragment["inputs"]
        if not isinstance(value, list):
            reject(f"{name} has malformed inputs")
        parts.append((name, raw))
        entries.extend(value)
    return root, tuple(parts), _inputs(entries, required)


def _lock(root: bytes, parts: tuple[tuple[str, bytes], ...]) -> bytes:
    lines = [LOCK_HEADER, b"manifest.toml sha256=" + hashlib.sha256(root).hexdigest().encode() + b"\n"]
    lines.extend(name.encode("ascii") + b" sha256=" + hashlib.sha256(raw).hexdigest().encode() + b"\n" for name, raw in parts)
    return b"".join(lines)


def render_v2_lock(path: Path) -> bytes:
    """Render the one canonical raw lock byte sequence for a valid v2 closure."""
    root, parts, _ = _v2_parts(_root(path))
    return _lock(root, parts)


def load_inventory(path: Path, *, allow_v1: bool = False) -> Inventory:
    """Load a v2 lock-verified inventory, or an explicitly permitted v1 inventory."""
    path = _root(path)
    raw = _raw(path)
    document = _toml(raw, "inventory root")
    schema = document.get("schema")
    if type(schema) is not int:
        reject("inventory root schema must be an integer")
    if schema == 1:
        if not allow_v1:
            reject("schema version 1 requires explicit compatibility")
        if set(document) != V1_FIELDS:
            reject("inventory root has unexpected fields")
        return Inventory(1, _inputs(document["inputs"], _metadata(document, V1_FIELDS)), hashlib.sha256(raw).hexdigest(), path)
    if schema != 2:
        reject("inventory root has an unsupported schema version")
    root, parts, inputs = _v2_parts(path)
    lock = path.with_name("inventory.lock")
    expected, actual = _lock(root, parts), _raw(lock)
    if actual != expected:
        reject("inventory.lock does not bind the root and component bytes")
    return Inventory(2, inputs, hashlib.sha256(actual).hexdigest(), lock)
