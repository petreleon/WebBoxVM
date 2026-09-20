#!/usr/bin/env python3
"""Expose only the complete sealed F02 source contract to a future F05 consumer."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE_DIR = HERE.parent / "01-role-aware-source-contract"
SOURCE_CONTRACT = SOURCE_DIR / "source_contract.json"
SOURCE_LOCK = SOURCE_DIR / "source_contract.lock"
LOCK_API = SOURCE_DIR / "role_aware_source_lock.py"
PAIRS = tuple((profile, role) for profile in ("opengl-4.6-core", "gles-3.2", "vulkan-1.4-core")
              for role in ("normative-root", "full-suite-root"))
CLAIMS = frozenset(("khronos_selector", "api_support", "conformance", "certification", "profile_support",
                    "performance"))
_BARE_MODULES = ("role_aware_source_lock", "role_aware_source_contract", "role_aware_source_evidence",
                 "full_suite_roots", "normative_roots", "vulkan_definition_contract", "source_role_contract",
                 "source_role_records", "source_role_artifacts", "source_cache", "source_model",
                 "normative_notices", "webboxvm_source_builder", "inventory_layout")


class F05SourceAdapterError(ValueError):
    """F05 cannot consume an absent, partial, stale, or qualifying source contract."""


def reject(message: str) -> None:
    raise F05SourceAdapterError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def _load_lock_api():
    previous_modules = {name: sys.modules.get(name) for name in _BARE_MODULES}
    private_name, previous_private = "f05_sealed_source_lock", sys.modules.get("f05_sealed_source_lock")
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
    except F05SourceAdapterError:
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


def _claim_free(value: object) -> bool:
    return isinstance(value, dict) and set(value) == CLAIMS and all(item is False for item in value.values())


def _checked_contract(value: object) -> dict[str, object]:
    fields = {"schema", "kind", "authority", "producer", "claims", "cts_executions", "record_count",
              "records", "inventory_lock_sha256", "bindings", "closures", "auxiliary", "states",
              "source_contract_sha256"}
    if not isinstance(value, dict) or set(value) != fields or value.get("schema") != 1:
        reject("source contract has an unexpected schema")
    body = {key: item for key, item in value.items() if key != "source_contract_sha256"}
    if value["source_contract_sha256"] != hashlib.sha256(canonical(body)).hexdigest():
        reject("source contract has a stale self hash")
    states = value["states"]
    required_states = {"mandatory_role_inventory_complete", "profile_status", "blocker", "api_support",
                       "conformance", "certification", "profile_support", "performance"}
    if (not _claim_free(value["claims"]) or value["cts_executions"] != 0 or not isinstance(states, dict)
            or set(states) != required_states or states["mandatory_role_inventory_complete"] is not True
            or states["profile_status"] != "blocked" or states["blocker"] != "matrix-incomplete"
            or any(states[name] is not False for name in required_states - {"mandatory_role_inventory_complete",
                                                                            "profile_status", "blocker"})):
        reject("source contract attempts to promote qualification or profile status")
    records = value["records"]
    if not isinstance(records, list) or value["record_count"] != 8 or len(records) != 8:
        reject("source contract lacks the complete eight-record catalog")
    record_map = {item.get("id"): item for item in records if isinstance(item, dict)}
    if len(record_map) != 8 or any(not isinstance(identifier, str) for identifier in record_map):
        reject("source contract has duplicate or malformed records")
    bindings = value["bindings"]
    if (not isinstance(bindings, list) or [(item.get("profile"), item.get("role")) for item in bindings
                                           if isinstance(item, dict)] != list(PAIRS)):
        reject("source contract has missing, duplicate, or reordered mandatory roles")
    for binding in bindings:
        if not isinstance(binding, dict) or binding.get("record_id") not in record_map:
            reject("source contract binding does not resolve a record")
        record = record_map[binding["record_id"]]
        fields = ("record_id", "record_kind", "scope", "revision", "sha256", "bytes")
        source = ("id", "kind", "scope", "revision", "sha256", "bytes")
        if any(binding.get(field) != record.get(name) for field, name in zip(fields, source)):
            reject("source contract binding has a stale record identity")
        if binding["role"] == "normative-root":
            if binding["record_kind"] != "upstream-source" or binding["scope"] != "normative-source":
                reject("source contract has a non-normative mandatory root")
        elif (binding["record_kind"] != "full-suite-root" or binding["scope"] != "full-conformance-suite"
              or record.get("profile") != binding["profile"] or binding.get("unfiltered") is not True
              or binding.get("suite_id") != binding["record_id"]):
            reject("source contract has a filtered, cross-profile, or noncanonical suite root")
    roots = [item["record_id"] for item in bindings if item["role"] == "full-suite-root"]
    closures = value["closures"]
    if (not isinstance(closures, list) or [(item.get("profile"), item.get("root_id")) for item in closures
                                           if isinstance(item, dict)] != [(pair[0], root) for pair, root in zip(PAIRS[1::2], roots)]
            or any(item.get("unfiltered") is not True for item in closures if isinstance(item, dict))):
        reject("source contract lacks an exact unfiltered full-suite closure")
    auxiliary = value["auxiliary"]
    if (not isinstance(auxiliary, list) or len(auxiliary) != 4
            or any(not isinstance(item, dict) or not _claim_free(item.get("claims"))
                   or item.get("cts_executions") != 0 or item.get("discharges_required_role") is not False
                   for item in auxiliary)):
        reject("source contract has malformed auxiliary no-claim evidence")
    bound = {item["record_id"] for item in bindings}
    if {item["id"] for item in auxiliary if item.get("id") in record_map} != set(record_map) - bound:
        reject("source contract permits an auxiliary-only role substitute")
    return copy.deepcopy(value)


def _descriptor(value: object) -> dict[str, object]:
    contract = _checked_contract(value)
    records = [{key: item.get(key) for key in ("id", "kind", "revision", "sha256", "bytes")}
               for item in contract["records"]]
    return {"schema": 1, "kind": "webboxvm-f05-admitted-source-contract", "authority": "WebBoxVM",
            "producer": "WebBoxVM", "source_contract_sha256": contract["source_contract_sha256"],
            "inventory_lock_sha256": contract["inventory_lock_sha256"], "record_count": 8, "records": records,
            "bindings": contract["bindings"], "closures": contract["closures"], "auxiliary": contract["auxiliary"],
            "claims": contract["claims"], "cts_executions": 0,
            "states": {"source_contract_complete": True, "f05_profile_registration": False,
                       "profile_status": "blocked", "blocker": "matrix-incomplete", "api_support": False,
                       "conformance": False, "certification": False, "profile_support": False, "performance": False}}


def _load_from_paths(contract_path: Path, lock_path: Path) -> dict[str, object]:
    try:
        return _descriptor(_load_lock_api().load_locked_contract(contract_path, lock_path))
    except F05SourceAdapterError:
        raise
    except Exception as error:
        reject(str(error))


def admitted_source_contract() -> dict[str, object]:
    """Return a new profile-neutral descriptor; callers cannot select or register a profile."""
    return _load_from_paths(SOURCE_CONTRACT, SOURCE_LOCK)


def main() -> None:
    if sys.argv[1:]:
        raise SystemExit("usage: f05_source_adapter.py")
    try:
        print(json.dumps(admitted_source_contract(), sort_keys=True))
    except F05SourceAdapterError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
