#!/usr/bin/env python3
"""Seal F03.4.2.1 Vulkan source channels without admitting a matrix row."""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BOUNDARY = HERE / "vulkan_source_channels.json"
PROFILE = "vulkan-1.4-core"
CLAIMS = {name: False for name in ("khronos_selector", "api_support", "conformance", "certification",
                                   "profile_support", "performance")}
STATES = {"mandatory_role_inventory_complete": True, "profile_status": "blocked",
          "blocker": "matrix-incomplete", "api_support": False, "conformance": False,
          "certification": False, "profile_support": False, "performance": False}
IDENTITY = ("id", "kind", "scope", "revision", "sha256", "bytes", "license", "attribution",
            "immutable_url", "artifact")


def sources():
    path, name = HERE / "vulkan_source_channel_sources.py", "f03421_channel_sources"
    prior = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, path)
        if path.is_symlink() or spec is None or spec.loader is None:
            raise RuntimeError("cannot load fixed channel sources")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            raise RuntimeError("channel sources resolved from an unexpected path")
        return module
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


SOURCE = sources()
BoundaryError, reject, canonical = SOURCE.BoundaryError, SOURCE.reject, SOURCE.canonical
BINDINGS = SOURCE.BINDINGS


def locked(api, source_contract: Path | None, source_lock: Path | None) -> dict[str, object]:
    if (source_contract is None) != (source_lock is None):
        reject("source contract and source lock must be supplied together")
    try:
        return api.load_locked_source_contract() if source_contract is None else api.load_locked_source_contract(source_contract, source_lock)
    except api.BindingError as error:
        reject(str(error))


def root(contract: object, api, role: str, identifier: str, channel: str, permitted: str) -> dict[str, object]:
    try:
        binding = api.binding_for(contract, PROFILE, role)
    except api.BindingError as error:
        reject(str(error))
    records = {item.get("id"): item for item in contract.get("records", []) if isinstance(item, dict)} if isinstance(contract, dict) else {}
    record = records.get(identifier)
    if binding.get("record_id") != identifier or not isinstance(record, dict):
        reject("contract has an unexpected Vulkan root")
    if any(binding.get(left) != record.get(right) for left, right in
           (("record_kind", "kind"), ("scope", "scope"), ("revision", "revision"), ("sha256", "sha256"), ("bytes", "bytes"))):
        reject("Vulkan root binding has a stale identity")
    return {"channel": channel, "role": role, "binding": binding,
            "identity": {key: record[key] for key in IDENTITY}, "permitted_use": permitted}


def registry(contract: object) -> dict[str, object]:
    module = SOURCE.inventory_api()
    try:
        identity = module.validate_scaffold()
    except Exception as error:
        reject(f"F03.4.1 registry scaffold is invalid: {error}")
    records = {item.get("id"): item for item in contract.get("records", []) if isinstance(item, dict)} if isinstance(contract, dict) else {}
    record = records.get("vulkan-registry")
    if not isinstance(record, dict) or any(identity.get(key) != record.get(key) for key in
                                           ("revision", "sha256", "bytes", "license", "attribution")):
        reject("F03.4.1 registry identity differs from the role-aware contract")
    if identity.get("scope") != "registry-metadata" or identity.get("discharges_required_role") is not False:
        reject("registry attempts to discharge a mandatory source role")
    return {"channel": "auxiliary-registry-structural-only", "identity": copy.deepcopy(identity),
            "permitted_use": "raw-structural-inventory-only", "matrix_ingress": False,
            "row_policy": {"status": "blocked", "implementation_owner": None, "independent_test_plan": None},
            "scaffold_row_count": 0}


def structural_rows(rows: object) -> int:
    fields = SOURCE.inventory_api().ROW_FIELDS
    if not isinstance(rows, list):
        reject("registry rows are not a list")
    for row in rows:
        if not isinstance(row, dict) or set(row) != fields:
            reject("registry row has an unsafe schema")
        if row.get("status") != "blocked" or row.get("implementation_owner") is not None or row.get("independent_test_plan") is not None:
            reject("registry structural row has an owner, test plan, or promoted status")
    return len(rows)


def rendered(contract: object, api) -> dict[str, object]:
    if not isinstance(contract, dict) or contract.get("claims") != CLAIMS or contract.get("cts_executions") != 0 or contract.get("states") != STATES:
        reject("contract does not retain the no-claim matrix-incomplete state")
    body = {"schema": 1, "kind": "webboxvm-vulkan-source-channel-boundary", "profile": PROFILE,
            "source_contract_sha256": contract["source_contract_sha256"], "inventory_lock_sha256": contract["inventory_lock_sha256"],
            "normative_root": root(contract, api, "normative-root", "vulkan-14-spec", "raw-docs-citation-root",
                                   "requires-separately-pinned-citation-map"),
            "full_suite_root": root(contract, api, "full-suite-root", "vulkan-cts-default", "full-suite-diagnostic-root",
                                    "root-wide-diagnostic-only"), "registry": registry(contract),
            "docs_include_closure": "unadmitted-requires-separately-pinned-citation-map", "matrix_row_count": 0,
            "claims": CLAIMS, "cts_executions": 0, "states": STATES}
    return {**body, "boundary_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate(boundary_path: Path = BOUNDARY, source_contract: Path | None = None,
             source_lock: Path | None = None) -> dict[str, object]:
    api = SOURCE.bindings_api()
    expected = rendered(locked(api, source_contract, source_lock), api)
    value = SOURCE.document(boundary_path)
    body = {key: item for key, item in value.items() if key != "boundary_sha256"}
    if value.get("boundary_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        reject("boundary has a stale self hash")
    if value != expected:
        reject("boundary differs from the exact role-aware source decision")
    return copy.deepcopy(value)


def matrix_ingress(row: object, boundary_path: Path = BOUNDARY) -> None:
    validate(boundary_path)
    locator = row.get("source_locator") if isinstance(row, dict) else None
    if not isinstance(locator, str):
        reject("Vulkan matrix row has no source locator")
    if "vk.xml" in locator or locator.startswith("xml/"):
        reject("registry structural locator cannot enter the normative matrix")
    reject("Vulkan matrix row requires an admitted citation map")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--boundary", type=Path, default=BOUNDARY)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        value = validate(args.boundary)
        print(json.dumps(value, sort_keys=True) if args.emit_json else "PASS: 3 sealed Vulkan channels; matrix-incomplete")
    except BoundaryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
