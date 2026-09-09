#!/usr/bin/env python3
"""Validate a future lock-bound F03 feature-matrix artifact."""

from __future__ import annotations

import re
from pathlib import Path

from profile_contract import ROW_FIELDS, STATUSES, ScopeError, document

REPO = Path(__file__).resolve().parents[6]
MATRIX_FIELDS = frozenset(("schema", "inventory_sha256", "rows"))
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9-]*$")
TASK = re.compile(r"^[A-Z]+[0-9]+(?:\.[0-9]+)*$")
PLACEHOLDERS = frozenset(("none", "pending", "tbd", "todo", "n/a", "na"))
NORMATIVE_SOURCES = {
    "opengl-4.6-core": frozenset(("opengl-46-core-spec", "opengl-gles-registry", "glsl-460-spec")),
    "gles-3.2": frozenset(("gles-32-spec", "opengl-gles-registry", "essl-320-spec")),
    "vulkan-1.4-core": frozenset(("vulkan-14-spec", "vulkan-registry", "spirv-core-grammar")),
}
TEST_SOURCES = {
    "opengl-4.6-core": "opengl-cts-manifest", "gles-3.2": "gles-cts-manifest",
    "vulkan-1.4-core": "vulkan-cts-mustpass",
}


def reject(message: str) -> None:
    raise ScopeError(message)


def text(row: dict[str, object], field: str) -> str:
    value = row[field]
    if not isinstance(value, str) or not value.strip() or value.strip().lower() in PLACEHOLDERS:
        reject(f"matrix row has an empty {field}")
    return value


def source(row: dict[str, object], prefix: str, inputs: dict[str, dict[str, object]]) -> str:
    identifier = text(row, f"{prefix}_id")
    if not IDENTIFIER.fullmatch(identifier) or identifier not in inputs:
        reject(f"matrix row has an unknown {prefix}_id")
    entry = inputs[identifier]
    if row[f"{prefix}_revision"] != entry["revision"] or row[f"{prefix}_sha256"] != entry["sha256"]:
        reject(f"matrix row has a mismatched {prefix} identity")
    return identifier


def evidence(path: Path, row: dict[str, object]) -> str:
    value = text(row, "evidence")
    link = value.partition("#")[0]
    reference = Path(link)
    target = (path.parent / reference).resolve()
    if (reference.is_absolute() or target.name != "evidence.md" or REPO not in target.parents
            or not target.is_file()):
        reject("matrix row needs a local evidence receipt")
    return value


def validate_matrix(
    path: Path, revision: str, inputs: dict[str, dict[str, object]], profiles: set[str],
) -> int:
    value = document(path)
    if set(value) != MATRIX_FIELDS or value.get("schema") != 1:
        reject("matrix does not match schema version 1")
    if value.get("inventory_sha256") != revision:
        reject("matrix has a stale inventory_sha256")
    rows = value.get("rows")
    if not isinstance(rows, list) or not rows:
        reject("matrix has no rows")
    seen = set()
    for row in rows:
        if not isinstance(row, dict) or set(row) != set(ROW_FIELDS):
            reject("matrix row does not match the reviewed schema")
        profile = text(row, "profile")
        kind, name, condition = (text(row, field) for field in ("requirement_kind", "name", "condition"))
        if profile not in profiles or profile not in NORMATIVE_SOURCES or not IDENTIFIER.fullmatch(kind):
            reject("matrix row has an invalid profile or requirement kind")
        if type(row["mandatory"]) is not bool:
            reject("matrix row mandatory must be boolean")
        normative = source(row, "source", inputs)
        test_source = source(row, "test_source", inputs)
        if (normative == test_source or normative not in NORMATIVE_SOURCES[profile]
                or test_source != TEST_SOURCES[profile]):
            reject("matrix row has unrelated normative or independent test sources")
        locator, owner, selector = (text(row, field) for field in
                                    ("source_locator", "owner_task", "test_selector"))
        evidence(path, row)
        if not TASK.fullmatch(owner) or not locator or not selector:
            reject("matrix row has an invalid owner, locator, or test selector")
        status, blocker = row["status"], row["blocker"]
        if status not in STATUSES or not isinstance(blocker, str):
            reject("matrix row has an invalid status or blocker")
        if status in ("blocked", "unsupported"):
            text(row, "blocker")
        if status in ("supported", "emulated") and blocker:
            reject("matrix row has a stale blocker")
        if status in ("supported", "emulated") and not evidence:
            reject("supported matrix row needs implementation evidence")
        key = (profile, kind, name, condition)
        if key in seen:
            reject("matrix has duplicate requirement rows")
        seen.add(key)
    return len(rows)
