#!/usr/bin/env python3
"""Exact, bounded access to the project-pinned Vulkan registry payload."""

from __future__ import annotations

import hashlib
import importlib.util
import os
import re
import stat
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONTRACT = HERE.parents[2]
MANIFEST = CONTRACT / "02-upstream-pins/01-input-inventory/manifest.toml"
BOUNDARY = CONTRACT / "02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/04-closure-admission-proof/04-admission-eligible-closure/03-source-release-boundary/source_release_boundary.py"
MAX_MEMBER_BYTES = 8 * 1024 * 1024
DIGEST = re.compile(r"^[0-9a-f]{64}$")


class RegistryError(ValueError):
    """A technical inventory would overstate what its input proves."""


def reject(message: str) -> None:
    raise RegistryError(message)


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        reject(f"cannot load {path.name}")
    module, previous = importlib.util.module_from_spec(spec), sys.modules.get(name)
    sys.modules[name] = module
    try:
        spec.loader.exec_module(module)
    except Exception:
        if previous is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = previous
        raise
    return module


def registry_identity() -> dict[str, object]:
    boundary = load_module(BOUNDARY, "f03_registry_boundary")
    try:
        boundary.boundary()
        layout = load_module(MANIFEST.with_name("inventory_layout.py"), "f03_registry_layout")
        inventory = layout.load_inventory(MANIFEST)
    except Exception as error:
        reject(f"reviewed source boundary is invalid: {error}")
    matches = [entry for entry in inventory.inputs if entry["id"] == "vulkan-registry"]
    if len(matches) != 1:
        reject("inventory does not have exactly one Vulkan registry")
    entry, expected = matches[0], boundary.VK_XML
    actual = (entry["revision"], entry["sha256"], entry["bytes"], entry["license"])
    if actual != expected[1:5] or entry["source_family"] != "vulkan":
        reject("inventory Vulkan registry differs from the reviewed source boundary")
    return {"source_id": entry["id"], "revision": entry["revision"], "sha256": entry["sha256"],
            "bytes": entry["bytes"], "license": entry["license"], "version_marker": expected[5]}


def bounded_bytes(path: Path, limit: int, label: str) -> bytes:
    if type(limit) is not int or not 0 < limit <= MAX_MEMBER_BYTES:
        reject(f"{label} has an unsafe byte limit")
    try:
        if not stat.S_ISREG(path.lstat().st_mode):
            reject(f"{label} is not a regular file")
        flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0)
        descriptor = os.open(path, flags)
        with os.fdopen(descriptor, "rb") as stream:
            if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
                reject(f"{label} is not a regular file")
            raw = stream.read(limit + 1)
    except OSError as error:
        reject(f"{label} cannot be read: {error}")
    if len(raw) > limit:
        reject(f"{label} exceeds its bounded size")
    return raw


def payload_bytes(path: Path, identity: dict[str, object]) -> bytes:
    size, digest = identity["bytes"], identity["sha256"]
    if type(size) is not int or type(digest) is not str or not DIGEST.fullmatch(digest):
        reject("registry identity has an invalid size or digest")
    raw = bounded_bytes(path, size, "registry payload")
    if len(raw) != size or hashlib.sha256(raw).hexdigest() != digest:
        reject("registry payload does not match its exact F02 identity")
    return raw
