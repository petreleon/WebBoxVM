#!/usr/bin/env python3
"""Load and project F02.5.4.1 roles without accepting an ambient module."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONTRACT = HERE.parents[1]
SOURCE_DIR = CONTRACT / "02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract"
SOURCE_CONTRACT = SOURCE_DIR / "source_contract.json"
SOURCE_LOCK = SOURCE_DIR / "source_contract.lock"
LOCK_API = SOURCE_DIR / "role_aware_source_lock.py"
PAIRS = tuple((profile, role) for profile in ("opengl-4.6-core", "gles-3.2", "vulkan-1.4-core")
              for role in ("normative-root", "full-suite-root"))
ROW_FIELDS = ("profile", "requirement_kind", "name", "mandatory", "source_role", "source_locator",
              "condition", "owner_task", "test_source_role", "status", "evidence", "blocker")
STATUSES = ("supported", "emulated", "unsupported", "blocked")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
_BARE_MODULES = ("role_aware_source_lock", "role_aware_source_contract", "role_aware_source_evidence",
                 "full_suite_roots", "normative_roots", "vulkan_definition_contract", "source_role_contract",
                 "source_role_records", "source_role_artifacts", "source_cache", "source_model",
                 "normative_notices")


class BindingError(ValueError):
    """The role-aware source boundary is not exact enough for F03."""


def reject(message: str) -> None:
    raise BindingError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def load_lock_api():
    """Load F02's lock by its path after isolating susceptible bare imports."""
    previous_modules = {name: sys.modules.get(name) for name in _BARE_MODULES}
    private_name = "f03_role_aware_source_lock"
    previous_private = sys.modules.get(private_name)
    previous_path = list(sys.path)
    try:
        for name in _BARE_MODULES:
            sys.modules.pop(name, None)
        sys.path.insert(0, str(SOURCE_DIR))
        spec = importlib.util.spec_from_file_location(private_name, LOCK_API)
        if spec is None or spec.loader is None:
            reject("cannot load the sealed F02 source lock")
        module = importlib.util.module_from_spec(spec)
        sys.modules[private_name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != LOCK_API:
            reject("sealed F02 source lock resolved from an unexpected path")
        return module
    except BindingError:
        raise
    except Exception as error:
        reject(f"cannot load the sealed F02 source lock: {error}")
    finally:
        sys.path[:] = previous_path
        for name, module in previous_modules.items():
            if module is None:
                sys.modules.pop(name, None)
            else:
                sys.modules[name] = module
        if previous_private is None:
            sys.modules.pop(private_name, None)
        else:
            sys.modules[private_name] = previous_private


def checked_contract(value: object) -> dict[str, object]:
    fields = {"schema", "kind", "authority", "producer", "claims", "cts_executions", "record_count",
              "records", "inventory_lock_sha256", "bindings", "closures", "auxiliary", "states",
              "source_contract_sha256"}
    if not isinstance(value, dict) or set(value) != fields or value.get("schema") != 1:
        reject("source contract has an unexpected schema")
    body = {key: item for key, item in value.items() if key != "source_contract_sha256"}
    if value["source_contract_sha256"] != hashlib.sha256(canonical(body)).hexdigest():
        reject("source contract has a stale self hash")
    claims, states = value["claims"], value["states"]
    if (not isinstance(claims, dict) or any(item is not False for item in claims.values())
            or value["cts_executions"] != 0 or not isinstance(states, dict)
            or states.get("mandatory_role_inventory_complete") is not True
            or states.get("profile_status") != "blocked" or states.get("blocker") != "matrix-incomplete"
            or any(states.get(name) is not False for name in ("api_support", "conformance", "certification",
                                                               "profile_support", "performance"))):
        reject("source contract attempts to promote a profile or qualification claim")
    records = value["records"]
    if not isinstance(records, list) or value["record_count"] != len(records):
        reject("source contract has malformed records")
    record_map = {item.get("id"): item for item in records if isinstance(item, dict)}
    if len(record_map) != len(records) or any(not isinstance(identifier, str) for identifier in record_map):
        reject("source contract has duplicate or malformed record ids")
    bindings = value["bindings"]
    if not isinstance(bindings, list) or [(item.get("profile"), item.get("role")) for item in bindings
                                          if isinstance(item, dict)] != list(PAIRS):
        reject("source contract has missing, duplicate, or reordered mandatory roles")
    for item in bindings:
        role = item["role"]
        expected = {"profile", "role", "record_id", "record_kind", "scope", "revision", "sha256", "bytes"}
        if role == "full-suite-root":
            expected |= {"suite_id", "selector_path", "unfiltered", "selector_scope"}
        if set(item) != expected or item["record_id"] not in record_map:
            reject("source contract has a malformed role binding")
        record = record_map[item["record_id"]]
        identity = {"record_id": "id", "record_kind": "kind", "scope": "scope", "revision": "revision",
                    "sha256": "sha256", "bytes": "bytes"}
        if any(item[field] != record.get(record_field) for field, record_field in identity.items()):
            reject("source contract binding has a stale record identity")
        if role == "normative-root":
            if item["record_kind"] != "upstream-source" or item["scope"] != "normative-source":
                reject("source contract has a non-normative mandatory root")
        elif (item["record_kind"] != "full-suite-root" or item["scope"] != "full-conformance-suite"
              or record.get("profile") != item["profile"] or item["suite_id"] != item["record_id"]
              or item["selector_path"] != record.get("selector_path") or item["unfiltered"] is not True
              or item["selector_scope"] != "Khronos full root; not core-only"):
            reject("source contract has a filtered, cross-profile, or noncanonical suite root")
    closures = value["closures"]
    expected_closures = [(profile, binding_for_unchecked(bindings, profile, "full-suite-root")["record_id"])
                         for profile in ("opengl-4.6-core", "gles-3.2", "vulkan-1.4-core")]
    if (not isinstance(closures, list)
            or [(item.get("profile"), item.get("root_id")) for item in closures if isinstance(item, dict)]
               != expected_closures
            or any(not SHA256.fullmatch(str(item.get(field, ""))) for item in closures
                   for field in ("ledger_sha256", "receipt_sha256"))):
        reject("source contract lacks an exact full-suite closure receipt")
    return copy.deepcopy(value)


def load_locked_source_contract(
    contract_path: Path = SOURCE_CONTRACT, lock_path: Path = SOURCE_LOCK,
) -> dict[str, object]:
    module = load_lock_api()
    try:
        return checked_contract(module.load_locked_contract(contract_path, lock_path))
    except ValueError as error:
        reject(str(error))


def binding_for_unchecked(bindings: list[dict[str, object]], profile: str, role: str) -> dict[str, object]:
    matches = [item for item in bindings if item["profile"] == profile and item["role"] == role]
    if len(matches) != 1:
        reject("profile role does not resolve exactly once")
    return matches[0]


def role_bindings(value: object) -> list[dict[str, object]]:
    return copy.deepcopy(checked_contract(value)["bindings"])


def binding_for(value: object, profile: str, role: str) -> dict[str, object]:
    contract = checked_contract(value)
    return copy.deepcopy(binding_for_unchecked(contract["bindings"], profile, role))
