#!/usr/bin/env python3
"""Fail-closed F03.1 profile scope and source-sufficiency contract."""

from __future__ import annotations

import importlib.util
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONTRACT = HERE.parents[1]
MANIFEST = CONTRACT / "02-upstream-pins/01-input-inventory/manifest.toml"
SCOPE = HERE / "profile_scope.json"
REQUIREMENTS_PATH = HERE / "source_requirements.json"


def load_inventory_api():
    """Load the reviewed sibling without accepting a cached basename collision."""
    path = CONTRACT / "02-upstream-pins/01-input-inventory/inventory_layout.py"
    spec = importlib.util.spec_from_file_location("f03_inventory_layout", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load inventory layout: {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    try:
        spec.loader.exec_module(module)
    except Exception:
        sys.modules.pop(spec.name, None)
        raise
    return module.InventoryLayoutError, module.load_inventory


def load_matrix_validator():
    """Load the sibling matrix validator without a cached-name substitution."""
    spec = importlib.util.spec_from_file_location("f03_matrix_contract", HERE / "matrix_contract.py")
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load matrix contract")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    try:
        spec.loader.exec_module(module)
    except Exception:
        sys.modules.pop(spec.name, None)
        raise
    return module.validate_matrix


InventoryLayoutError, load_inventory = load_inventory_api()
PROFILE_FIELDS = frozenset(("id", "api", "version", "scope", "compatibility", "extensions", "status", "blocker"))
ROOT_FIELDS = frozenset(("schema", "inventory_sha256", "profiles", "row_fields", "statuses"))
REQUIREMENT_FIELDS = frozenset(("profile", "role", "required_input_id", "related_input_ids"))
ROW_FIELDS = ("profile", "requirement_kind", "name", "mandatory", "source_id", "source_revision",
              "source_sha256", "source_locator", "condition", "owner_task", "test_source_id",
              "test_source_revision", "test_source_sha256", "test_selector", "status", "evidence", "blocker")
PROFILES = (
    ("opengl-4.6-core", "OpenGL", "4.6", "core-only", "excluded", "not-promised"),
    ("gles-3.2", "GLES", "3.2", "core-only", "desktop-compatibility-excluded", "not-promised"),
    ("vulkan-1.4-core", "Vulkan", "1.4", "core-only", "wsi-and-portability-separate", "promoted-provenance-only"),
)
STATUSES = ("supported", "emulated", "unsupported", "blocked")
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9-]*$")
EXPECTED_REQUIREMENTS = (
    ("opengl-4.6-core", "api-limit-format-spec", "opengl-46-core-spec", ("opengl-gles-registry", "glsl-460-spec")),
    ("opengl-4.6-core", "conformance-manifest", "opengl-cts-manifest", ("piglit-gl30-bindfragdata",)),
    ("gles-3.2", "api-limit-format-spec", "gles-32-spec", ("opengl-gles-registry", "essl-320-spec")),
    ("gles-3.2", "conformance-manifest", "gles-cts-manifest", ("piglit-gl30-bindfragdata",)),
    ("vulkan-1.4-core", "api-limit-format-spec", "vulkan-14-spec", ("vulkan-registry", "spirv-core-grammar")),
    ("vulkan-1.4-core", "conformance-manifest", "vulkan-cts-mustpass", ("vk-gl-cts-api-version",)),
)


class ScopeError(ValueError):
    """A profile scope or source-requirement contract is unsafe."""


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


def inventory(path: Path) -> tuple[str, dict[str, dict[str, object]]]:
    try:
        loaded = load_inventory(path)
    except InventoryLayoutError as error:
        reject(f"inventory cannot be loaded: {error}")
    return loaded.revision, {str(entry["id"]): entry for entry in loaded.inputs}


def validate_scope(value: dict[str, object], revision: str) -> dict[str, dict[str, object]]:
    if set(value) != ROOT_FIELDS or value.get("schema") != 1:
        reject("profile scope does not match schema version 1")
    if value.get("inventory_sha256") != revision:
        reject("profile scope has a stale inventory_sha256")
    if value.get("row_fields") != list(ROW_FIELDS) or value.get("statuses") != list(STATUSES):
        reject("profile scope has an unexpected row schema or status catalog")
    records = value.get("profiles")
    if not isinstance(records, list):
        reject("profile scope has malformed profiles")
    actual = []
    for profile in records:
        if not isinstance(profile, dict) or set(profile) != PROFILE_FIELDS:
            reject("profile scope has malformed profile fields")
        if profile["status"] != "blocked" or not isinstance(profile["blocker"], str):
            reject("profile scope must retain a blocked matrix state")
        actual.append(tuple(profile[field] for field in ("id", "api", "version", "scope", "compatibility", "extensions")))
    if tuple(actual) != PROFILES:
        reject("profile scope does not match the reviewed final targets")
    return {str(profile["id"]): profile for profile in records}


def source_gaps(value: dict[str, object], revision: str, identifiers: set[str], profiles: set[str]) -> tuple[str, ...]:
    if set(value) != {"schema", "inventory_sha256", "requirements"} or value.get("schema") != 1:
        reject("source requirements do not match schema version 1")
    if value.get("inventory_sha256") != revision:
        reject("source requirements have a stale inventory_sha256")
    requirements = value.get("requirements")
    if not isinstance(requirements, list) or not requirements:
        reject("source requirements are empty")
    seen, actual, missing = set(), [], []
    for requirement in requirements:
        if not isinstance(requirement, dict) or set(requirement) != REQUIREMENT_FIELDS:
            reject("source requirement has malformed fields")
        profile, role, required = requirement["profile"], requirement["role"], requirement["required_input_id"]
        related = requirement["related_input_ids"]
        if (not isinstance(profile, str) or profile not in profiles
                or not all(isinstance(item, str) and IDENTIFIER.fullmatch(item) for item in (role, required))):
            reject("source requirement has an invalid profile, role, or input id")
        if (not isinstance(related, list) or not related
                or any(not isinstance(item, str) or not IDENTIFIER.fullmatch(item) for item in related)
                or len(set(related)) != len(related) or not set(related) <= identifiers):
            reject("source requirement has unknown related input ids")
        if (profile, role) in seen:
            reject("source requirements duplicate a profile role")
        seen.add((profile, role))
        actual.append((profile, role, required, tuple(related)))
        if required not in identifiers:
            missing.append(required)
    if tuple(actual) != EXPECTED_REQUIREMENTS:
        reject("source requirements do not match the reviewed profile catalog")
    return tuple(missing)


def profile_blocker(missing: tuple[str, ...]) -> str:
    return "inventory-sources-incomplete" if missing else "matrix-incomplete"


def validate(
    scope_path: Path = SCOPE, requirements_path: Path = REQUIREMENTS_PATH,
    manifest: Path = MANIFEST, matrix_path: Path | None = None,
) -> tuple[str, ...]:
    revision, inputs = inventory(manifest)
    profiles = validate_scope(document(scope_path), revision)
    missing = source_gaps(document(requirements_path), revision, set(inputs), set(profiles))
    blocker = profile_blocker(missing)
    if any(profile["blocker"] != blocker for profile in profiles.values()):
        reject(f"profile scope must use blocker {blocker}")
    if matrix_path is not None:
        load_matrix_validator()(matrix_path, revision, inputs, set(profiles))
    return missing
