#!/usr/bin/env python3
"""Read only the sealed auxiliary Vulkan registry identity for F03.4.1 v2."""

from __future__ import annotations

import hashlib
import importlib.util
import os
import re
import stat
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BINDINGS_API = HERE.parents[2] / "01-profile-scope" / "role_aware_bindings.py"
MAX_MEMBER_BYTES = 8 * 1024 * 1024
DIGEST = re.compile(r"^[0-9a-f]{64}$")
SOURCE_ID = "vulkan-registry"
SOURCE_CONTRACT_SHA256 = "d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3"
INVENTORY_LOCK_SHA256 = "44a0f280e0ed854091a33e458122bd8cd9a0f9fc2e4c51a7a5be92bf48d8c6f4"
REVISION = "f84d432d5b8912362f96f581f29bbc4f3c8c7843"
SHA256 = "cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06"
LICENSE = "Apache-2.0 OR MIT (vk.xml SPDX license choice)"
CLAIMS = frozenset(("khronos_selector", "api_support", "conformance", "certification", "profile_support", "performance"))


class RegistryError(ValueError):
    """The raw registry input cannot prove a Vulkan profile or CTS result."""


def reject(message: str) -> None:
    raise RegistryError(message)


def _binding_api():
    if BINDINGS_API.is_symlink() or not BINDINGS_API.is_file():
        reject("role-aware binding API must be a regular fixed-path file")
    private, previous = "f0341_role_aware_bindings", sys.modules.get("f0341_role_aware_bindings")
    try:
        spec = importlib.util.spec_from_file_location(private, BINDINGS_API)
        if spec is None or spec.loader is None:
            reject("cannot load the fixed role-aware binding API")
        module = importlib.util.module_from_spec(spec)
        sys.modules[private] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != BINDINGS_API.resolve():
            reject("role-aware binding API resolved from an unexpected path")
        return module
    except RegistryError:
        raise
    except Exception as error:
        reject(f"cannot load the role-aware binding API: {error}")
    finally:
        if previous is None:
            sys.modules.pop(private, None)
        else:
            sys.modules[private] = previous


def _claim_free(value: object) -> bool:
    return isinstance(value, dict) and set(value) == CLAIMS and all(item is False for item in value.values())


def identity_from_contract(value: object) -> dict[str, object]:
    """Project one fixed auxiliary record; it cannot discharge either required role."""
    if not isinstance(value, dict) or not _claim_free(value.get("claims")) or value.get("cts_executions") != 0:
        reject("sealed source contract promotes a qualification claim")
    if value.get("source_contract_sha256") != SOURCE_CONTRACT_SHA256 or value.get("inventory_lock_sha256") != INVENTORY_LOCK_SHA256:
        reject("sealed source contract has an unexpected identity")
    states = value.get("states")
    if not isinstance(states, dict) or states.get("profile_status") != "blocked" or states.get("blocker") != "matrix-incomplete":
        reject("sealed source contract promotes a profile")
    records = value.get("records")
    matches = [item for item in records if isinstance(item, dict) and item.get("id") == SOURCE_ID] if isinstance(records, list) else []
    if len(matches) != 1:
        reject("sealed source contract lacks exactly one Vulkan registry")
    record = matches[0]
    expected = {"kind": "upstream-source", "scope": "registry-metadata", "authority": "Khronos", "producer": "Khronos",
                "revision": REVISION, "sha256": SHA256, "bytes": 3309653, "license": LICENSE,
                "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{REVISION}/xml/vk.xml",
                "attribution": "Copyright 2015-2026 The Khronos Group Inc.; Vulkan-Docs xml/vk.xml; Apache-2.0 OR MIT",
                "artifact": f"webboxvm-graphics/f02/{SOURCE_ID}/{SHA256}.source"}
    if any(record.get(key) != item for key, item in expected.items()) or not _claim_free(record.get("claims")):
        reject("Vulkan registry differs from the sealed auxiliary record")
    auxiliary = value.get("auxiliary")
    matches = [item for item in auxiliary if isinstance(item, dict) and item.get("id") == SOURCE_ID] if isinstance(auxiliary, list) else []
    if len(matches) != 1 or matches[0].get("record_kind") != "upstream-source" or matches[0].get("scope") != "registry-metadata":
        reject("Vulkan registry is not an exact auxiliary record")
    entry = matches[0]
    if entry.get("sha256") != SHA256 or not _claim_free(entry.get("claims")) or entry.get("cts_executions") != 0 or entry.get("discharges_required_role") is not False:
        reject("Vulkan registry attempts to discharge a required role")
    bindings = value.get("bindings")
    if not isinstance(bindings, list) or any(isinstance(item, dict) and item.get("record_id") == SOURCE_ID for item in bindings):
        reject("Vulkan registry cannot substitute a normative or full-suite root")
    return {"source_contract_sha256": SOURCE_CONTRACT_SHA256, "inventory_lock_sha256": INVENTORY_LOCK_SHA256,
            "source_id": SOURCE_ID, "record_kind": "upstream-source", "scope": "registry-metadata",
            "authority": "Khronos", "producer": "Khronos", "revision": REVISION, "sha256": SHA256,
            "bytes": 3309653, "license": LICENSE, "immutable_url": expected["immutable_url"],
            "attribution": expected["attribution"], "artifact": expected["artifact"], "version_marker": "VK_VERSION_1_4",
            "claims": {key: False for key in sorted(CLAIMS)}, "cts_executions": 0,
            "admission": "auxiliary-registry-metadata", "discharges_required_role": False}


def registry_identity() -> dict[str, object]:
    try:
        return identity_from_contract(_binding_api().load_locked_source_contract())
    except RegistryError:
        raise
    except Exception as error:
        reject(f"sealed source contract cannot be loaded: {error}")


def checked_identity(value: object) -> dict[str, object]:
    expected = registry_identity()
    if not isinstance(value, dict) or value != expected:
        reject("registry identity differs from the sealed auxiliary source contract")
    return expected


def bounded_bytes(path: Path, limit: int, label: str) -> bytes:
    if type(limit) is not int or not 0 < limit <= MAX_MEMBER_BYTES:
        reject(f"{label} has an unsafe byte limit")
    try:
        if not isinstance(path, Path) or not stat.S_ISREG(path.lstat().st_mode):
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


def payload_bytes(path: Path, identity: object) -> bytes:
    checked = checked_identity(identity)
    raw = bounded_bytes(path, checked["bytes"], "registry payload")
    if len(raw) != checked["bytes"] or hashlib.sha256(raw).hexdigest() != checked["sha256"]:
        reject("registry payload does not match its sealed auxiliary identity")
    if b"SPDX-License-Identifier: Apache-2.0 OR MIT" not in raw:
        reject("registry payload lacks the sealed SPDX expression")
    return raw
