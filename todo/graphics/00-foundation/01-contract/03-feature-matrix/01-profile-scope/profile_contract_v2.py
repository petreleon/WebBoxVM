#!/usr/bin/env python3
"""Fail-closed F03 v2 profile scope bound to the sealed role catalog."""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

from role_aware_bindings import (BindingError, ROW_FIELDS, SOURCE_CONTRACT, SOURCE_LOCK, STATUSES,
                                 load_locked_source_contract, role_bindings)

HERE = Path(__file__).resolve().parent
SCOPE = HERE / "profile_scope_v2.json"
REQUIREMENTS_PATH = HERE / "source_requirements_v2.json"
PROFILE_FIELDS = frozenset(("id", "api", "version", "scope", "compatibility", "extensions", "status", "blocker"))
ROOT_FIELDS = frozenset(("schema", "source_contract_sha256", "inventory_lock_sha256", "profiles", "row_fields",
                         "statuses"))
PROFILES = (
    ("opengl-4.6-core", "OpenGL", "4.6", "core-only", "excluded", "not-promised"),
    ("gles-3.2", "GLES", "3.2", "core-only", "desktop-compatibility-excluded", "not-promised"),
    ("vulkan-1.4-core", "Vulkan", "1.4", "core-only", "wsi-and-portability-separate", "promoted-provenance-only"),
)


class ScopeError(ValueError):
    """An F03 v2 document cannot safely consume the sealed source contract."""


def reject(message: str) -> None:
    raise ScopeError(message)


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"{path.name} cannot be read: {error}")
    if not isinstance(value, dict):
        reject(f"{path.name} is not a JSON object")
    return value


def source_headers(value: dict[str, object], contract: dict[str, object], label: str) -> None:
    if (value.get("source_contract_sha256") != contract["source_contract_sha256"]
            or value.get("inventory_lock_sha256") != contract["inventory_lock_sha256"]):
        reject(f"{label} has a stale sealed source identity")


def validate_scope(value: dict[str, object], contract: dict[str, object]) -> dict[str, dict[str, object]]:
    if set(value) != ROOT_FIELDS or value.get("schema") != 2:
        reject("profile scope does not match schema version 2")
    source_headers(value, contract, "profile scope")
    if value.get("row_fields") != list(ROW_FIELDS) or value.get("statuses") != list(STATUSES):
        reject("profile scope has an unexpected row schema or status catalog")
    records = value.get("profiles")
    if not isinstance(records, list):
        reject("profile scope has malformed profiles")
    actual = []
    for profile in records:
        if not isinstance(profile, dict) or set(profile) != PROFILE_FIELDS:
            reject("profile scope has malformed profile fields")
        if profile["status"] != "blocked" or profile["blocker"] != "matrix-incomplete":
            reject("profile scope must retain the matrix-incomplete blocker")
        actual.append(tuple(profile[field] for field in ("id", "api", "version", "scope", "compatibility", "extensions")))
    if tuple(actual) != PROFILES:
        reject("profile scope does not match the reviewed final targets")
    return {str(profile["id"]): profile for profile in records}


def validate_requirements(value: dict[str, object], contract: dict[str, object], profiles: set[str]) -> None:
    if set(value) != {"schema", "source_contract_sha256", "inventory_lock_sha256", "requirements"} or value.get("schema") != 2:
        reject("source requirements do not match schema version 2")
    source_headers(value, contract, "source requirements")
    requirements = value.get("requirements")
    if not isinstance(requirements, list) or {item.get("profile") for item in requirements if isinstance(item, dict)} != profiles:
        reject("source requirements have malformed profile coverage")
    if requirements != role_bindings(contract):
        reject("source requirements differ from the locked role binding catalog")


def load_matrix_validator():
    spec = importlib.util.spec_from_file_location("f03_matrix_contract_v2", HERE / "matrix_contract_v2.py")
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load the F03 v2 matrix contract")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    try:
        spec.loader.exec_module(module)
    except Exception:
        sys.modules.pop(spec.name, None)
        raise
    return module.validate_matrix


def validate(
    scope_path: Path = SCOPE, requirements_path: Path = REQUIREMENTS_PATH,
    source_contract: Path = SOURCE_CONTRACT, source_lock: Path = SOURCE_LOCK, matrix_path: Path | None = None,
) -> tuple[()]:
    try:
        contract = load_locked_source_contract(source_contract, source_lock)
    except BindingError as error:
        reject(str(error))
    profiles = validate_scope(document(scope_path), contract)
    validate_requirements(document(requirements_path), contract, set(profiles))
    if matrix_path is not None:
        try:
            load_matrix_validator()(matrix_path, contract, set(profiles))
        except ValueError as error:
            reject(str(error))
    return ()
