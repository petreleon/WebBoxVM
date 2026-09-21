#!/usr/bin/env python3
"""Bind the complete unfiltered VCTS ledger as a root-wide diagnostic only."""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def fixed_source():
    path, name = HERE / "vulkan_full_suite_diagnostic_sources.py", "f03423_fixed_diagnostic_sources"
    if path.is_symlink() or not path.is_file():
        raise RuntimeError("fixed diagnostic source API must be a regular file")
    prior = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            raise RuntimeError("cannot load fixed diagnostic source API")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            raise RuntimeError("diagnostic source API resolved from an unexpected path")
        return module
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


source = fixed_source()
DIAGNOSTIC = HERE / "vulkan_full_suite_diagnostics.json"
PROFILE, MAX_BYTES = "vulkan-1.4-core", 8 * 1024 * 1024
CLAIMS = {name: False for name in ("khronos_selector", "api_support", "conformance", "certification",
                                   "profile_support", "performance")}
STATES = {"mandatory_role_inventory_complete": True, "profile_status": "blocked", "blocker": "matrix-incomplete",
          "api_support": False, "conformance": False, "certification": False, "profile_support": False, "performance": False}
EXPECTED_BINDING = {"profile": PROFILE, "role": "full-suite-root", "record_id": "vulkan-cts-default",
                    "record_kind": "full-suite-root", "scope": "full-conformance-suite",
                    "revision": "f6a29701220f34dd1407513bfe80d74ca7b392ce",
                    "sha256": "b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4",
                    "bytes": 3347, "suite_id": "vulkan-cts-default",
                    "selector_path": "external/vulkancts/mustpass/main/vk-default.txt", "unfiltered": True,
                    "selector_scope": "Khronos full root; not core-only"}
CATEGORIES = {"core": 0, "wsi": 1, "video": 1, "extension": 4, "unknown": 92}
ROOT_KEYS = ("id", "kind", "scope", "revision", "sha256", "bytes", "license", "attribution", "immutable_url", "artifact")

DiagnosticError, reject, canonical, document = (source.DiagnosticSourceError, source.reject, source.canonical, source.document)


def external_root(path: Path) -> Path:
    if not isinstance(path, Path) or not path.is_absolute() or path.is_symlink():
        reject("selector cache root must be an absolute nonsymlink path")
    root = path.resolve()
    if root == source.REPO or source.REPO in root.parents:
        reject("selector cache root must remain external to the repository")
    return root


def cache_fact(cache_root: Path, contract: dict[str, object]) -> dict[str, object]:
    root, api = external_root(cache_root), source.cache_api()
    try:
        verified = api.verify_selector_cache(api.source_lock.load_locked_contract(), root)
    except Exception as error:
        reject(f"sealed external selector cache cannot be verified: {error}")
    expected = {"source_contract_sha256": contract["source_contract_sha256"],
                "selector_record_ids": ["opengl-46-core-spec", "opengl-cts-gl46-main", "gles-32-spec",
                                        "gles-cts-main", "vulkan-14-spec", "vulkan-cts-default", "vulkan-registry"],
                "release_proof_ids": ["opengl-cts-4681-license", "vulkan-cts-1462-license"]}
    if verified != expected:
        reject("selector cache has incomplete, mixed, filtered, or stale identities")
    return {"mode": "sealed-external-selector-cache", "file_count": 9, **verified}


def source_inputs(cache_root: Path):
    api = source.bindings_api()
    try:
        contract = api.load_locked_source_contract()
        binding = api.binding_for(contract, PROFILE, "full-suite-root")
    except Exception as error:
        reject(f"cannot load the fixed F03 full-suite role: {error}")
    if (binding != EXPECTED_BINDING or contract.get("claims") != CLAIMS or contract.get("cts_executions") != 0
            or contract.get("states") != STATES):
        reject("F03 does not expose the exact no-claim unfiltered Vulkan full-suite role")
    try:
        channel, observed = source.channel_api().validate(), source.ledger_api().build()
    except Exception as error:
        reject(f"Vulkan source-channel or ledger evidence is invalid: {error}")
    full = channel.get("full_suite_root") if isinstance(channel, dict) else None
    root = observed.get("source_root") if isinstance(observed, dict) else None
    ledger = observed.get("ledger") if isinstance(observed, dict) else None
    taxonomy = observed.get("taxonomy") if isinstance(observed, dict) else None
    if (not isinstance(full, dict) or full.get("binding") != binding or not isinstance(root, dict)
            or {key: root.get(key) for key in ROOT_KEYS} != full.get("identity")
            or not isinstance(ledger, dict) or not isinstance(taxonomy, dict)
            or observed.get("record_sha256") != "cea45295dab76b0adeed650f884cb14a12ff9e169bade31200c10649e50ed6f3"
            or (ledger.get("identity_sha256"), ledger.get("ledger_sha256"), ledger.get("member_count"), ledger.get("member_total_bytes"))
               != ("30b272f8c563e0dabf307795c01496eb70f744514b4790439ebbfc69c7bd5218",
                   "608d520463bfd4724e79b52ee639a12d446e14c5e8d7b3687872eaf3c4e917f0", 98, 434669348)
            or taxonomy.get("category_counts") != CATEGORIES or root.get("unfiltered") is not True):
        reject("F02 full-suite ledger is not the exact unfiltered root-wide observation")
    return contract, binding, channel, root, ledger, taxonomy, cache_fact(cache_root, contract)


def rows(members: object) -> list[dict[str, object]]:
    if not isinstance(members, list) or len(members) != 98:
        reject("VCTS diagnostic lacks the complete ordered member set")
    result = []
    for ordinal, item in enumerate(members, 1):
        if not isinstance(item, dict) or item.get("category") not in CATEGORIES:
            reject("VCTS diagnostic has an unclassified or malformed member")
        result.append({"ordinal": ordinal, "path": item["path"], "parent_path": item["parent_path"], "revision": item["revision"],
                       "blob_sha1": item["blob_sha1"], "sha256": item["sha256"], "bytes": item["bytes"],
                       "category": item["category"], "rule_id": item["rule_id"], "source_locator": item["source_locator"],
                       "relation": "root-wide-diagnostic-only", "implementation_owner": None,
                       "independent_test_plan": None, "coverage": "unassigned"})
    if {name: sum(item["category"] == name for item in result) for name in CATEGORIES} != CATEGORIES:
        reject("VCTS diagnostic reclassifies the observed root-wide taxonomy")
    return result


def build(cache_root: Path) -> dict[str, object]:
    contract, binding, channel, root, ledger, taxonomy, cache = source_inputs(cache_root)
    observed = rows(ledger["members"])
    body = {"schema": 1, "kind": "webboxvm-vulkan-full-suite-diagnostic", "profile": PROFILE,
            "source_contract_sha256": contract["source_contract_sha256"], "inventory_lock_sha256": contract["inventory_lock_sha256"],
            "source_channel_boundary_sha256": channel["boundary_sha256"],
            "full_suite_root": {"binding": binding, "identity": {key: root[key] for key in ROOT_KEYS},
                                "permitted_use": "root-wide-diagnostic-only"}, "selector_cache": cache,
            "ledger": {"identity_sha256": ledger["identity_sha256"], "ledger_sha256": ledger["ledger_sha256"],
                       "member_count": ledger["member_count"], "member_total_bytes": ledger["member_total_bytes"],
                       "rows_sha256": hashlib.sha256(canonical(observed)).hexdigest()},
            "taxonomy": {"taxonomy_sha256": taxonomy["taxonomy_sha256"], "category_counts": CATEGORIES}, "rows": observed,
            "diagnostic_scope": {"local_filtering": "forbidden", "case_selection": "none", "coverage_inference": "forbidden",
                                  "cts_execution": "forbidden", "matrix_row_count": 0}, "claims": CLAIMS, "cts_executions": 0,
            "states": STATES}
    return {**body, "diagnostic_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate(path: Path = DIAGNOSTIC, cache_root: Path | None = None) -> dict[str, object]:
    if cache_root is None:
        reject("an external selector cache is required")
    expected, value = build(cache_root), document(path)
    body = {key: item for key, item in value.items() if key != "diagnostic_sha256"}
    if value.get("diagnostic_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        reject("diagnostic has a stale self hash")
    if len(canonical(value)) > MAX_BYTES:
        reject("diagnostic exceeds the 8 MiB artifact limit")
    if value != expected:
        reject("diagnostic differs from the exact root-wide VCTS observation")
    return copy.deepcopy(value)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--selector-cache-root", type=Path, required=True)
    parser.add_argument("--diagnostic", type=Path, default=DIAGNOSTIC)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        value = validate(args.diagnostic, args.selector_cache_root)
        print(json.dumps(value, sort_keys=True) if args.emit_json else "PASS: 98 root-wide VCTS diagnostic rows; matrix-incomplete")
    except DiagnosticError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
