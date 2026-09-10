#!/usr/bin/env python3
"""Fail closed while authoritative metadata for derived Docs members is absent."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ANCHOR_DIR = HERE.parent / "01-successor-closure-anchor"
sys.path.insert(0, str(ANCHOR_DIR))
import successor_closure_anchor as closure_anchor

RECORD = HERE / "derived_member_authority_gate.json"
TARGET = ("vulkan-1.4-core", "api-limit-format-spec", "vulkan-14-spec")
TARGET_FIELDS = ("profile", "role", "required_input_id")
IDENTITY = ("selector", "sha256", "bytes", "generation_id")
AUTHORITY = ("license_expression", "attribution", "source_role", "provenance", "producer_authority")
PROHIBITIONS = ("defaults", "raw-to-derived-inheritance", "output-to-source-inheritance",
                "uniform-invented-labels", "aliases")
FACTS = frozenset(("authority_manifest_present", "all_derived_fields_authoritative", "source_role_proved"))
EFFECTS = frozenset(("inventory_changed", "cache_freshness_proved", "f03_changed", "supported",
                     "conformant", "certified", "near_native", "admission_eligible", "admitted",
                     "cutover_ready", "satisfies_vulkan_14_core_manifest"))


class AuthorityError(ValueError):
    """The authority gate cannot safely describe the current blocker."""


class AuthorityBlocked(AuthorityError):
    """No independently authoritative manifest is currently available."""


def reject(message: str) -> None:
    raise AuthorityError(message)


def exact(value: object, fields: frozenset[str], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != fields:
        reject(f"{label} has an invalid schema")
    return value


def list_value(value: object, expected: tuple[str, ...], label: str) -> None:
    if type(value) is not list or value != list(expected):
        reject(f"{label} is incomplete or reordered")


def digest(value: object, label: str) -> str:
    try:
        return closure_anchor.digest(value, label)
    except closure_anchor.AnchorError as error:
        reject(str(error))


def canonical(value: dict[str, object]) -> bytes:
    return json.dumps({key: item for key, item in value.items() if key != "gate_sha256"},
                      sort_keys=True, separators=(",", ":")).encode()


def document(path: Path, label: str = "authority gate") -> dict[str, object]:
    try:
        return closure_anchor.document(path, label)
    except closure_anchor.AnchorError as error:
        reject(str(error))


def anchor_digest() -> str:
    try:
        return closure_anchor.anchor()
    except closure_anchor.AnchorError as error:
        reject(f"successor closure anchor is invalid: {error}")


def all_false(value: object, fields: frozenset[str], label: str) -> None:
    rows = exact(value, fields, label)
    if any(item is not False for item in rows.values()):
        reject(f"{label} attempts to claim authority, admission, or release effects")


def gate_value(value: object) -> str:
    fields = frozenset(("schema", "contract", "status", *TARGET_FIELDS, "anchor_sha256",
                        "expected_derived_count", "identity_fields", "authority_fields", "prohibitions",
                        "external_authority_manifest", "facts", "effects", "gate_sha256"))
    record = exact(value, fields, "derived member authority gate")
    state = tuple(record.get(key) for key in ("schema", "contract", "status", *TARGET_FIELDS))
    if type(record.get("schema")) is not int or state != (
            1, "vulkan-docs-derived-member-authority-gate-v1", "external-authority-unavailable", *TARGET):
        reject("authority gate has an unexpected contract or target")
    if digest(record["anchor_sha256"], "anchor_sha256") != anchor_digest():
        reject("authority gate does not bind the historical successor anchor")
    if type(record.get("expected_derived_count")) is not int or record["expected_derived_count"] != 1462:
        reject("authority gate does not retain the exact derived-member count")
    list_value(record["identity_fields"], IDENTITY, "derived identity fields")
    list_value(record["authority_fields"], AUTHORITY, "derived authority fields")
    list_value(record["prohibitions"], PROHIBITIONS, "authority prohibitions")
    if record["external_authority_manifest"] is not None:
        reject("local or unanchored authority manifest is forbidden")
    all_false(record["facts"], FACTS, "authority facts")
    all_false(record["effects"], EFFECTS, "authority effects")
    actual = hashlib.sha256(canonical(record)).hexdigest()
    if digest(record["gate_sha256"], "gate_sha256") != actual:
        reject("authority gate sha256 does not bind its contents")
    return actual


def gate(path: Path = RECORD) -> str:
    return gate_value(document(path))


def require_authority_manifest(value: object) -> None:
    if value is not None:
        reject("local or unanchored authority manifest is forbidden")
    raise AuthorityBlocked("per-derived external authority is unavailable")


def main() -> None:
    try:
        if len(sys.argv) not in (1, 2):
            reject("usage: derived_member_authority_gate.py [GATE.json]")
        record = document(Path(sys.argv[1]) if len(sys.argv) == 2 else RECORD)
        gate_value(record)
        require_authority_manifest(record["external_authority_manifest"])
    except AuthorityBlocked as error:
        print("AUTHORITY-GATE: 1462 derived; external authority unavailable")
        print(f"BLOCKED: {error}")
        raise SystemExit(2)
    except AuthorityError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
