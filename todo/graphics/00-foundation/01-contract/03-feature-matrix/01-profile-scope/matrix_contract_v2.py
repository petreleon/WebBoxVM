#!/usr/bin/env python3
"""Validate F03 v2 rows through the sealed role-aware binding API."""

from __future__ import annotations

import importlib.util
import json
import re
import sys
from pathlib import Path

from role_aware_bindings import BindingError, ROW_FIELDS, STATUSES, binding_for, role_bindings

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[6]
VULKAN_PROFILE = "vulkan-1.4-core"
VULKAN_BOUNDARY = HERE.parent / "04-vulkan-core/02-provenance-diagnostics/01-source-channel-boundary/vulkan_source_channels.py"
MATRIX_FIELDS = frozenset(("schema", "source_contract_sha256", "inventory_lock_sha256", "rows"))
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9-]*$")
TASK = re.compile(r"^[A-Z]+[0-9]+(?:\.[0-9]+)*$")
PLACEHOLDERS = frozenset(("none", "pending", "tbd", "todo", "n/a", "na"))


class MatrixError(ValueError):
    """A feature-matrix row escapes its sealed source-role boundary."""


def reject(message: str) -> None:
    raise MatrixError(message)


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"{path.name} cannot be read: {error}")
    if not isinstance(value, dict):
        reject(f"{path.name} is not a JSON object")
    return value


def text(row: dict[str, object], field: str) -> str:
    value = row[field]
    if not isinstance(value, str) or not value.strip() or value.strip().lower() in PLACEHOLDERS:
        reject(f"matrix row has an empty {field}")
    return value


def evidence(path: Path, row: dict[str, object]) -> str:
    value = text(row, "evidence")
    reference = Path(value.partition("#")[0])
    target = (path.parent / reference).resolve()
    if reference.is_absolute() or target.name != "evidence.md" or REPO not in target.parents or not target.is_file():
        reject("matrix row needs a local evidence receipt")
    return value


def vulkan_boundary():
    if VULKAN_BOUNDARY.is_symlink() or not VULKAN_BOUNDARY.is_file():
        reject("Vulkan source-channel boundary is unavailable")
    name, prior, old_path = "f03421_vulkan_source_channels", sys.modules.get("f03421_vulkan_source_channels"), list(sys.path)
    try:
        sys.path.insert(0, str(VULKAN_BOUNDARY.parent))
        spec = importlib.util.spec_from_file_location(name, VULKAN_BOUNDARY)
        if spec is None or spec.loader is None:
            reject("cannot load the Vulkan source-channel boundary")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != VULKAN_BOUNDARY.resolve():
            reject("Vulkan source-channel boundary resolved from an unexpected path")
        return module
    except MatrixError:
        raise
    except Exception as error:
        reject(f"cannot load the Vulkan source-channel boundary: {error}")
    finally:
        sys.path[:] = old_path
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


def validate_vulkan_ingress(row: dict[str, object]) -> None:
    try:
        vulkan_boundary().matrix_ingress(row)
    except ValueError as error:
        reject(str(error))


def validate_matrix(path: Path, source_contract: dict[str, object], profiles: set[str]) -> int:
    try:
        role_bindings(source_contract)
    except BindingError as error:
        reject(str(error))
    value = document(path)
    if set(value) != MATRIX_FIELDS or value.get("schema") != 2:
        reject("matrix does not match schema version 2")
    if (value.get("source_contract_sha256") != source_contract["source_contract_sha256"]
            or value.get("inventory_lock_sha256") != source_contract["inventory_lock_sha256"]):
        reject("matrix has a stale sealed source identity")
    rows = value.get("rows")
    if not isinstance(rows, list) or not rows:
        reject("matrix has no rows")
    seen = set()
    for row in rows:
        if not isinstance(row, dict) or set(row) != set(ROW_FIELDS):
            reject("matrix row does not match the reviewed schema")
        profile = text(row, "profile")
        kind, name, condition = (text(row, field) for field in ("requirement_kind", "name", "condition"))
        if profile not in profiles or not IDENTIFIER.fullmatch(kind):
            reject("matrix row has an invalid profile or requirement kind")
        if type(row["mandatory"]) is not bool:
            reject("matrix row mandatory must be boolean")
        if row["source_role"] != "normative-root" or row["test_source_role"] != "full-suite-root":
            reject("matrix row does not use the exact mandatory source roles")
        try:
            binding_for(source_contract, profile, row["source_role"])
            binding_for(source_contract, profile, row["test_source_role"])
        except BindingError as error:
            reject(str(error))
        locator, owner = (text(row, field) for field in ("source_locator", "owner_task"))
        evidence(path, row)
        if not TASK.fullmatch(owner) or not locator:
            reject("matrix row has an invalid owner or locator")
        if profile == VULKAN_PROFILE:
            validate_vulkan_ingress(row)
        if row["status"] not in STATUSES or row["status"] != "blocked" or row["blocker"] != "matrix-incomplete":
            reject("matrix rows remain blocked by matrix-incomplete")
        key = (profile, kind, name, condition)
        if key in seen:
            reject("matrix has duplicate requirement rows")
        seen.add(key)
    return len(rows)
