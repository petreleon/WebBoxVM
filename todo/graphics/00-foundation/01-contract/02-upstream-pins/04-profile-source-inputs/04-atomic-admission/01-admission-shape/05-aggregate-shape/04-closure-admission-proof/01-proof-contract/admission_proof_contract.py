#!/usr/bin/env python3
"""Freeze the six-role, non-admitting grammar for aggregate closure proof."""

from __future__ import annotations

import hashlib
import json
import sys
from dataclasses import dataclass
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONTRACT = HERE / "admission_proof_contract.json"
REQUIREMENTS = HERE.parents[6] / "03-feature-matrix/01-profile-scope/source_requirements.json"
MAX_BYTES = 64 * 1024
ROOT_FIELDS = frozenset(("schema", "kind", "source_requirements_sha256", "roles", "states", "contract_sha256"))
ROLE_FIELDS = frozenset(("profile", "role", "required_input_id", "envelope"))
REQUIREMENT_FIELDS = frozenset(("profile", "role", "required_input_id", "related_input_ids"))
STATE_FIELDS = ("admission_eligible", "inventory_ready", "fresh_cache_ready", "cutover_ready", "f03_ready")
CANONICAL_REQUIREMENTS = (
    ("opengl-4.6-core", "api-limit-format-spec", "opengl-46-core-spec", ("opengl-gles-registry", "glsl-460-spec")),
    ("opengl-4.6-core", "conformance-manifest", "opengl-cts-manifest", ("piglit-gl30-bindfragdata",)),
    ("gles-3.2", "api-limit-format-spec", "gles-32-spec", ("opengl-gles-registry", "essl-320-spec")),
    ("gles-3.2", "conformance-manifest", "gles-cts-manifest", ("piglit-gl30-bindfragdata",)),
    ("vulkan-1.4-core", "api-limit-format-spec", "vulkan-14-spec", ("vulkan-registry", "spirv-core-grammar")),
    ("vulkan-1.4-core", "conformance-manifest", "vulkan-cts-mustpass", ("vk-gl-cts-api-version",)),
)
CANONICAL = tuple(row[:3] for row in CANONICAL_REQUIREMENTS)
ENVELOPES = ("v1-regular-source", "v1-regular-source", "v1-regular-source",
             "v1-regular-compound", "v1-docs-unresolved-boundary",
             "v1-vcts-boundary-plus-v2-diagnostic")


class ContractError(ValueError):
    """A proof grammar input can falsely promote an incomplete source closure."""


@dataclass(frozen=True)
class FrozenContract:
    roles: tuple[tuple[str, str, str, str], ...]
    source_requirements_sha256: str
    states: tuple[tuple[str, bool], ...]


def reject(message: str) -> None:
    raise ContractError(message)


def pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in rows:
        if key in value:
            reject("proof contract has a duplicate JSON key")
        value[key] = item
    return value


def load(path: Path, label: str) -> tuple[dict[str, object], bytes]:
    if not isinstance(path, Path):
        reject(f"{label} path is invalid")
    try:
        raw = path.read_bytes()
        if len(raw) > MAX_BYTES:
            reject(f"{label} exceeds its JSON byte limit")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except ContractError:
        raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"{label} cannot be read: {error}")
    if not isinstance(value, dict):
        reject(f"{label} is not a JSON object")
    return value, raw


def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "contract_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def sha256(raw: bytes, label: str) -> str:
    value = hashlib.sha256(raw).hexdigest()
    if not value or len(value) != 64:
        reject(f"{label} has no immutable digest")
    return value


def locked_digest(value: object, label: str) -> str:
    if (not isinstance(value, str) or len(value) != 64
            or any(character not in "0123456789abcdef" for character in value)):
        reject(f"{label} has an invalid immutable digest")
    return value


def requirement_rows(path: Path) -> tuple[tuple[tuple[str, str, str], ...], str]:
    value, raw = load(path, "source requirements")
    if (set(value) != {"schema", "inventory_sha256", "requirements"}
            or type(value.get("schema")) is not int or value.get("schema") != 1):
        reject("source requirements have an unexpected schema")
    locked_digest(value.get("inventory_sha256"), "source requirements inventory lock")
    rows = value.get("requirements")
    if not isinstance(rows, list):
        reject("source requirements have no ordered rows")
    actual = []
    for row in rows:
        if not isinstance(row, dict) or set(row) != REQUIREMENT_FIELDS:
            reject("source requirements have a malformed row")
        values = tuple(row.get(key) for key in ("profile", "role", "required_input_id"))
        related = row.get("related_input_ids")
        if (any(not isinstance(item, str) or not item for item in values) or not isinstance(related, list)
                or not related or any(not isinstance(item, str) or not item for item in related)
                or len(set(related)) != len(related)):
            reject("source requirements have an incomplete row")
        actual.append((*values, tuple(related)))
    if tuple(actual) != CANONICAL_REQUIREMENTS:
        reject("source requirements do not retain the canonical six-role order")
    return tuple(row[:3] for row in actual), sha256(raw, "source requirements")


def validate(path: Path = CONTRACT, requirements_path: Path = REQUIREMENTS) -> FrozenContract:
    required, requirements_digest = requirement_rows(requirements_path)
    value, _ = load(path, "proof contract")
    if (set(value) != ROOT_FIELDS or type(value.get("schema")) is not int or value.get("schema") != 1
            or value.get("kind") != "aggregate-admission-proof-contract"):
        reject("proof contract has an unexpected schema")
    if locked_digest(value.get("source_requirements_sha256"), "proof contract source-requirements lock") != requirements_digest:
        reject("proof contract has a stale source-requirements lock")
    roles, states = value.get("roles"), value.get("states")
    if not isinstance(roles, list) or not isinstance(states, dict) or set(states) != set(STATE_FIELDS):
        reject("proof contract has an incomplete role or readiness grammar")
    actual = []
    for row in roles:
        if not isinstance(row, dict) or set(row) != ROLE_FIELDS:
            reject("proof contract has a malformed role envelope")
        values = tuple(row.get(key) for key in ("profile", "role", "required_input_id", "envelope"))
        if any(not isinstance(item, str) or not item for item in values):
            reject("proof contract has an incomplete role envelope")
        actual.append(values)
    if tuple(item[:3] for item in actual) != required or tuple(item[3] for item in actual) != ENVELOPES:
        reject("proof contract permits a substituted, reordered, or mixed-policy role")
    if any(type(states[key]) is not bool or states[key] for key in STATE_FIELDS):
        reject("proof contract has a falsely ready state")
    if locked_digest(value.get("contract_sha256"), "proof contract self-hash") != digest(value):
        reject("proof contract self-hash is invalid")
    return FrozenContract(tuple(actual), requirements_digest, tuple((key, states[key]) for key in STATE_FIELDS))


def main() -> None:
    try:
        result = validate() if len(sys.argv) == 1 else None
        if result is None:
            raise SystemExit("usage: admission_proof_contract.py")
    except ContractError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"CONTRACT: {len(result.roles)} roles {result.source_requirements_sha256} non-admitting")


if __name__ == "__main__":
    main()
