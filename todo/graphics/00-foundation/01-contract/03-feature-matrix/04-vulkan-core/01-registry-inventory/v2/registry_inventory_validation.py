#!/usr/bin/env python3
"""Fail closed for a bounded auxiliary-registry fact inventory."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from registry_inventory_contract import (
    BLOCKER, BOUNDARIES, CONTAINERS, EFFECTS, KINDS, LOCATOR, MAX_SERIALIZED, NAME,
    ROW_FIELDS, packed, rows_sha256,
)
from registry_inventory_source import RegistryError, bounded_bytes, registry_identity, reject

HERE = Path(__file__).resolve().parent
SCAFFOLD = HERE / "registry_inventory.json"
MAX_DOCUMENT_BYTES = 64 * 1024
EXPECTED_COUNT = 1458
EXPECTED_ROWS_SHA256 = "5835f804af840c6df88ee907ea98d6bec0ec2dbb9821e4e0a58a3df6864bab15"


def pairs(items: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in items:
        if key in value:
            reject(f"duplicate JSON key {key!r}")
        value[key] = item
    return value


def exact(value: object, expected: dict[str, object]) -> bool:
    return (isinstance(value, dict) and set(value) == set(expected)
            and all(type(value[key]) is type(item) and value[key] == item for key, item in expected.items()))


def valid_condition(value: object, container: object) -> bool:
    if not isinstance(value, str):
        return False
    try:
        parsed = json.loads(value, object_pairs_hook=pairs)
    except (json.JSONDecodeError, RegistryError):
        return False
    expected = {"feature"} if container == "feature" else {"feature", "section", "item"}
    return (isinstance(parsed, dict) and set(parsed) == expected and packed(parsed) == value
            and all(isinstance(part, dict) and all(isinstance(key, str) and isinstance(item, str)
                    for key, item in part.items()) for part in parsed.values()))


def validate_inventory(value: object, identity: dict[str, object]) -> None:
    fields = {"schema", "contract", "status", "source_contract_sha256", "inventory_lock_sha256", "registry", "scope",
              "boundaries", "rows", "rows_sha256", "effects"}
    headers = {key: identity[key] for key in ("source_contract_sha256", "inventory_lock_sha256")}
    if not isinstance(value, dict) or set(value) != fields or not exact(value.get("registry"), identity) or any(value.get(key) != item for key, item in headers.items()):
        reject("technical inventory has an invalid schema, source header, or identity")
    if (type(value.get("schema")) is not int or (value.get("schema"), value.get("contract"), value.get("status"))
            != (2, "vulkan-registry-technical-inventory-v2", "blocked")):
        reject("technical inventory has an unsafe status")
    if value.get("scope") != "raw-direct-cumulative-vulkan-1.0-through-1.4-structural-blocks" or value.get("boundaries") != list(BOUNDARIES):
        reject("technical inventory has an unsafe scope or boundary")
    rows = value.get("rows")
    if not exact(value.get("effects"), {effect: False for effect in EFFECTS}) or not isinstance(rows, list):
        reject("technical inventory has promoted effects or malformed rows")
    if len(rows) != EXPECTED_COUNT or rows_sha256(rows) != EXPECTED_ROWS_SHA256 or value.get("rows_sha256") != EXPECTED_ROWS_SHA256:
        reject("technical inventory is stale, partial, reordered, or has an unknown raw payload")
    seen = set()
    for order, record in enumerate(rows, 1):
        if not isinstance(record, dict) or set(record) != ROW_FIELDS:
            reject("technical inventory has a malformed row")
        kind, container, name, locator = (record[field] for field in ("requirement_kind", "container_kind", "name", "source_locator"))
        if (kind not in KINDS or container not in CONTAINERS | {"feature"} or not isinstance(name, str)
                or not NAME.fullmatch(name) or not isinstance(locator, str) or not LOCATOR.fullmatch(locator)
                or type(record["source_order"]) is not int or record["source_order"] != order
                or record["status"] != "blocked" or record["implementation_owner"] is not None
                or record["independent_test_plan"] is not None or record["blocker"] != BLOCKER
                or (kind == "version-marker") != (container == "feature") or not valid_condition(record["condition"], container)):
            reject("technical inventory has an assigned owner, test plan, or promoted row")
        if locator in seen:
            reject("technical inventory has duplicate source locators")
        seen.add(locator)
    if len(packed(value).encode()) > MAX_SERIALIZED:
        reject("technical inventory exceeds the 8 MiB serialized member cap")


def canonical(value: dict[str, object]) -> bytes:
    return packed({key: item for key, item in value.items() if key != "scaffold_sha256"}).encode()


def validate_scaffold(path: Path = SCAFFOLD) -> dict[str, object]:
    identity, raw = registry_identity(), bounded_bytes(path, MAX_DOCUMENT_BYTES, "inventory scaffold")
    try:
        value = json.loads(raw, object_pairs_hook=pairs)
    except (UnicodeDecodeError, json.JSONDecodeError, RegistryError) as error:
        reject(f"inventory scaffold is invalid JSON: {error}")
    fields = {"schema", "contract", "status", "source_contract_sha256", "inventory_lock_sha256", "registry", "boundaries",
              "rows", "effects", "scaffold_sha256"}
    headers = {key: identity[key] for key in ("source_contract_sha256", "inventory_lock_sha256")}
    if not isinstance(value, dict) or set(value) != fields or not exact(value.get("registry"), identity) or any(value.get(key) != item for key, item in headers.items()):
        reject("inventory scaffold has an invalid schema, source header, or identity")
    effects = {effect: False for effect in EFFECTS}
    if (type(value.get("schema")) is not int or (value.get("schema"), value.get("contract"), value.get("status"))
            != (2, "vulkan-registry-inventory-scaffold-v2", "blocked")):
        reject("inventory scaffold has an unsafe status")
    if value.get("boundaries") != list(BOUNDARIES) or value.get("rows") != [] or not exact(value.get("effects"), effects):
        reject("inventory scaffold has rows, missing boundaries, or promoted effects")
    if value.get("scaffold_sha256") != hashlib.sha256(canonical(value)).hexdigest():
        reject("inventory scaffold has a stale self-hash")
    return identity
