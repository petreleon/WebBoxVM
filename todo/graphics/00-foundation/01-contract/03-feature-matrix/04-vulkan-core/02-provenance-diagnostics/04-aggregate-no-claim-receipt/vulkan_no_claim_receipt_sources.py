#!/usr/bin/env python3
"""Fixed loaders and raw-registry checks for F03.4.2.4."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[7]
DIAGNOSTICS = HERE.parent
REGISTRY = DIAGNOSTICS.parent / "01-registry-inventory" / "v2" / "registry_inventory.py"
MAX_BYTES, ROWS = 8 * 1024 * 1024, 1458
ROWS_SHA256 = "5835f804af840c6df88ee907ea98d6bec0ec2dbb9821e4e0a58a3df6864bab15"
SOURCE_CONTRACT = "d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3"
INVENTORY_LOCK = "44a0f280e0ed854091a33e458122bd8cd9a0f9fc2e4c51a7a5be92bf48d8c6f4"
INVENTORY_SCOPE = "raw-direct-cumulative-vulkan-1.0-through-1.4-structural-blocks"
BOUNDARIES = ["extensions-excluded", "wsi-excluded", "external-memory-extension-paths-excluded",
              "spirv-grammar-separate", "bring-up-unimplemented"]
EFFECTS = ("normative_docs_role", "full_suite_root", "cts_selection", "project_complete_docs_role",
           "project_cts_selector", "project_release_claim", "supported", "emulated", "conformant",
           "certified", "near_native")
ROW_BLOCKER = "source-contract-owner-and-independent-test-plan-unassigned"
REGISTRY_ID = {
    "source_id": "vulkan-registry", "record_kind": "upstream-source", "scope": "registry-metadata",
    "revision": "f84d432d5b8912362f96f581f29bbc4f3c8c7843",
    "sha256": "cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06", "bytes": 3309653,
    "admission": "auxiliary-registry-metadata", "discharges_required_role": False, "authority": "Khronos",
    "producer": "Khronos", "license": "Apache-2.0 OR MIT (vk.xml SPDX license choice)",
    "attribution": "Copyright 2015-2026 The Khronos Group Inc.; Vulkan-Docs xml/vk.xml; Apache-2.0 OR MIT",
    "immutable_url": "https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/f84d432d5b8912362f96f581f29bbc4f3c8c7843/xml/vk.xml",
    "artifact": "webboxvm-graphics/f02/vulkan-registry/cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06.source",
    "source_contract_sha256": SOURCE_CONTRACT, "inventory_lock_sha256": INVENTORY_LOCK, "version_marker": "VK_VERSION_1_4",
    "claims": {"api_support": False, "certification": False, "conformance": False, "khronos_selector": False,
               "performance": False, "profile_support": False}, "cts_executions": 0}


class ReceiptError(ValueError):
    """A provenance input does not justify an aggregate planning receipt."""


def reject(message: str) -> None:
    raise ReceiptError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")


def exact(actual: object, expected: object) -> bool:
    if type(actual) is not type(expected):
        return False
    if isinstance(expected, dict):
        return actual.keys() == expected.keys() and all(exact(actual[key], item) for key, item in expected.items())
    if isinstance(expected, list):
        return len(actual) == len(expected) and all(exact(left, right) for left, right in zip(actual, expected))
    return actual == expected


def pairs(items: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in items:
        if key in value:
            reject(f"duplicate JSON key {key!r}")
        value[key] = item
    return value


def document(path: Path, label: str) -> dict[str, object]:
    if path.is_symlink() or not path.is_file() or path.stat().st_size > MAX_BYTES:
        reject(f"{label} must be a bounded regular file")
    try:
        value = json.loads(path.read_bytes(), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, ReceiptError) as error:
        reject(f"{label} is not strict UTF-8 JSON: {error}")
    if not isinstance(value, dict):
        reject(f"{label} must contain an object")
    return value


def fixed_module(path: Path, name: str):
    if path.is_symlink() or not path.is_file():
        reject(f"{name} must be a fixed regular file")
    prior = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            reject(f"cannot load {name}")
        module = importlib.util.module_from_spec(spec); sys.modules[name] = module; spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            reject(f"{name} resolved from an unexpected path")
        return module
    except ReceiptError:
        raise
    except Exception as error:
        reject(f"cannot load {name}: {error}")
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


def modules():
    return tuple(fixed_module(DIAGNOSTICS / folder / file, name) for folder, file, name in (
        ("01-source-channel-boundary", "vulkan_source_channels.py", "f03424_channel"),
        ("02-raw-docs-provenance", "vulkan_raw_docs_citations.py", "f03424_docs"),
        ("03-full-suite-diagnostics", "vulkan_full_suite_diagnostics.py", "f03424_vcts")))


def external_root(path: Path, label: str) -> Path:
    if not isinstance(path, Path) or not path.is_absolute() or path.is_symlink():
        reject(f"{label} must be an absolute nonsymlink cache path")
    root = path.resolve()
    if root == REPO or REPO in root.parents:
        reject(f"{label} must remain external to the repository")
    return root


def registry_summary(value: object) -> dict[str, object]:
    fields = {"schema", "contract", "status", "source_contract_sha256", "inventory_lock_sha256", "registry",
              "scope", "boundaries", "rows", "rows_sha256", "effects"}
    if not isinstance(value, dict) or set(value) != fields or not isinstance(value.get("rows"), list):
        reject("registry inventory has no raw rows")
    rows = value["rows"]
    if any(not isinstance(row, dict) for row in rows):
        reject("registry inventory has a malformed raw row")
    orders, locators = [row.get("source_order") for row in rows], [row.get("source_locator") for row in rows]
    if len(set(locators)) != len(locators):
        reject("registry inventory has duplicate raw locators")
    if orders != list(range(1, len(rows) + 1)):
        reject("registry inventory has missing, duplicate, or reordered raw rows")
    if (len(rows), value.get("rows_sha256"), hashlib.sha256(canonical(rows)).hexdigest()) != (ROWS, ROWS_SHA256, ROWS_SHA256):
        reject("registry inventory has a stale, partial, or unknown raw-row digest")
    if (value.get("source_contract_sha256"), value.get("inventory_lock_sha256")) != (SOURCE_CONTRACT, INVENTORY_LOCK):
        reject("registry inventory has stale source-contract or lock identities")
    policy = {"status": "blocked", "implementation_owner": None, "independent_test_plan": None,
              "blocker": ROW_BLOCKER}
    if type(value.get("schema")) is not int or value["schema"] != 2:
        reject("registry inventory has an unknown schema")
    if value.get("contract") != "vulkan-registry-technical-inventory-v2" or value.get("status") != "blocked" or any(
            {key: row.get(key) for key in policy} != policy for row in rows):
        reject("registry inventory has relabeled scope or promoted raw rows")
    if value.get("scope") != INVENTORY_SCOPE or value.get("boundaries") != BOUNDARIES:
        reject("registry inventory has an unsafe scope or boundary")
    registry, effects = value.get("registry"), value.get("effects")
    if not exact(registry, REGISTRY_ID):
        reject("registry inventory differs from the sealed auxiliary role")
    if not exact(effects, {effect: False for effect in EFFECTS}):
        reject("registry inventory promotes an effect")
    return {**REGISTRY_ID, "source_contract_sha256": SOURCE_CONTRACT, "inventory_lock_sha256": INVENTORY_LOCK,
            "raw_row_count": ROWS, "source_order": "1..1458", "rows_sha256": ROWS_SHA256, "row_policy": policy,
            "inventory_scope": INVENTORY_SCOPE, "boundaries": BOUNDARIES}


def registry_fact(cache_root: Path) -> dict[str, object]:
    root = external_root(cache_root, "selector cache")
    result = subprocess.run([sys.executable, str(REGISTRY), "--selector-cache-root", str(root), "--emit-json"],
                            cwd=REGISTRY.parent, capture_output=True, text=True,
                            env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}, check=False)
    if result.returncode != 0:
        reject("sealed registry inventory could not be reproduced from the selector cache")
    try:
        return registry_summary(json.loads(result.stdout, object_pairs_hook=pairs))
    except (json.JSONDecodeError, ReceiptError) as error:
        reject(f"sealed registry inventory is invalid: {error}")
