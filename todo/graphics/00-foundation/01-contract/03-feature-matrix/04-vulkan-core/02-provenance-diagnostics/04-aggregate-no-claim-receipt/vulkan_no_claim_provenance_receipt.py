#!/usr/bin/env python3
"""Join Vulkan raw-provenance receipts without importing a matrix claim."""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
RECEIPT = HERE / "vulkan_no_claim_provenance_receipt.json"
PROFILE = "vulkan-1.4-core"
BASE_CLAIMS = {name: False for name in ("khronos_selector", "api_support", "conformance", "certification",
                                        "profile_support", "performance")}
CLAIMS = {**BASE_CLAIMS, "guest_execution": False, "browser_execution": False, "matrix_complete": False}
STATES = {"mandatory_role_inventory_complete": True, "profile_status": "blocked", "blocker": "matrix-incomplete",
          "api_support": False, "conformance": False, "certification": False, "profile_support": False, "performance": False}
DOCS_MAP_SHA = "2d6b3270cd4af8d14d752d593cd226c54d953aa034ce3f23cb221eecc0b22e44"
DOCS_SCOPE = {"map_role": "citation-only", "include_closure": "unadmitted", "rendered_docs": "forbidden",
              "inferred_symbol_name_locators": "forbidden", "core_coverage": "not-established", "matrix_row_count": 0,
              "registry_structural_fact": {"reason": "no reviewed raw Docs semantic anchor", "source_locator": None, "status": "blocked"}}
VCTS_DIAGNOSTIC_SHA = "66d771723737b03f4a301db51b927ee51da89db9eaf5f2a10f8deeaf8e7f0b68"
VCTS_ROWS_SHA = "47fc6c455ad1bfd42c105ad8b7e4c3d4c6ab54ea962cbd28f2fc339a56f7f6af"
VCTS_SCOPE = {"local_filtering": "forbidden", "case_selection": "none", "coverage_inference": "forbidden",
              "cts_execution": "forbidden", "matrix_row_count": 0}


def fixed_source():
    path, name = HERE / "vulkan_no_claim_receipt_sources.py", "f03424_sources"
    prior = sys.modules.get(name)
    try:
        if path.is_symlink() or not path.is_file():
            raise RuntimeError("fixed aggregate sources must be a regular file")
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            raise RuntimeError("cannot load fixed aggregate sources")
        module = importlib.util.module_from_spec(spec); sys.modules[name] = module; spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            raise RuntimeError("aggregate sources resolved from an unexpected path")
        return module
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


SOURCE = fixed_source()
ReceiptError, reject, canonical, document = SOURCE.ReceiptError, SOURCE.reject, SOURCE.canonical, SOURCE.document
CHANNEL, DOCS, VCTS = SOURCE.modules()


def source_inputs(selector_cache_root: Path, docs_cache_root: Path):
    try:
        boundary = CHANNEL.validate()
        docs = DOCS.validate(cache_root=SOURCE.external_root(docs_cache_root, "Docs cache"))
        diagnostic = VCTS.validate(cache_root=SOURCE.external_root(selector_cache_root, "selector cache"))
    except Exception as error:
        reject(f"prerequisite provenance receipt is invalid: {error}")
    registry = SOURCE.registry_fact(selector_cache_root)
    headers = (boundary.get("source_contract_sha256"), boundary.get("inventory_lock_sha256"))
    if (headers[0] is None or headers[1] is None or any((item.get("source_contract_sha256"), item.get("inventory_lock_sha256")) != headers
                                                         for item in (docs, diagnostic)) or
            (registry.get("source_contract_sha256"), registry.get("inventory_lock_sha256")) != headers):
        reject("provenance receipts have mixed source-contract or lock identities")
    if (docs.get("source_channel_boundary_sha256"), diagnostic.get("source_channel_boundary_sha256")) != (boundary.get("boundary_sha256"),) * 2:
        reject("provenance receipts have mixed source-channel identities")
    return boundary, docs, diagnostic, registry


def build(selector_cache_root: Path, docs_cache_root: Path) -> dict[str, object]:
    boundary, docs, diagnostic, registry = source_inputs(selector_cache_root, docs_cache_root)
    if (boundary.get("profile"), docs.get("profile"), diagnostic.get("profile")) != (PROFILE,) * 3:
        reject("provenance receipts use a mixed profile")
    if (boundary.get("claims"), docs.get("claims"), diagnostic.get("claims")) != (BASE_CLAIMS,) * 3 or any(
            item.get("cts_executions") != 0 or item.get("states") != STATES for item in (boundary, docs, diagnostic)):
        reject("provenance receipts promote a claim or execution")
    if (docs.get("map_sha256"), docs.get("citation_candidate_count"), docs.get("scope"), docs.get("matrix_rows")) != (
            DOCS_MAP_SHA, 2, DOCS_SCOPE, []):
        reject("Docs provenance leaves the citation-only no-claim boundary")
    rows = diagnostic.get("rows")
    if (not isinstance(rows, list) or (diagnostic.get("diagnostic_sha256"), diagnostic.get("ledger", {}).get("member_count"),
                                       diagnostic.get("ledger", {}).get("rows_sha256"), diagnostic.get("full_suite_root", {}).get("permitted_use"),
                                       diagnostic.get("diagnostic_scope"), len(rows)) !=
            (VCTS_DIAGNOSTIC_SHA, 98, VCTS_ROWS_SHA, "root-wide-diagnostic-only", VCTS_SCOPE, 98)):
        reject("VCTS provenance leaves the root-wide diagnostic no-claim boundary")
    body = {"schema": 1, "kind": "webboxvm-vulkan-no-claim-provenance-receipt", "profile": PROFILE,
            "source_contract_sha256": boundary["source_contract_sha256"], "inventory_lock_sha256": boundary["inventory_lock_sha256"],
            "source_channel_boundary_sha256": boundary["boundary_sha256"], "registry": registry,
            "docs": {"citation_map_sha256": docs["map_sha256"], "citation_candidate_count": docs["citation_candidate_count"],
                     "map_role": docs["scope"]["map_role"], "include_closure": docs["scope"]["include_closure"],
                     "matrix_row_count": docs["scope"]["matrix_row_count"]},
            "vcts": {"diagnostic_sha256": diagnostic["diagnostic_sha256"], "member_count": diagnostic["ledger"]["member_count"],
                     "rows_sha256": diagnostic["ledger"]["rows_sha256"], "permitted_use": diagnostic["full_suite_root"]["permitted_use"],
                     "matrix_row_count": diagnostic["diagnostic_scope"]["matrix_row_count"]},
            "matrix_rows": [], "matrix_row_count": 0, "claims": CLAIMS, "cts_executions": 0, "states": STATES,
            "unresolved_obligations": ["admitted-Docs-closure", "reviewed-semantic-core-matrix", "owners-and-independent-test-plans",
                                       "core-only-CTS-selection-and-execution", "guest-and-browser-validation",
                                       "conformance-and-certification", "same-GPU-performance-measurement"]}
    return {**body, "receipt_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate(path: Path = RECEIPT, selector_cache_root: Path | None = None,
             docs_cache_root: Path | None = None) -> dict[str, object]:
    if selector_cache_root is None or docs_cache_root is None:
        reject("external selector and Docs caches are required")
    expected, value = build(selector_cache_root, docs_cache_root), document(path, "aggregate receipt")
    body = {key: item for key, item in value.items() if key != "receipt_sha256"}
    if value.get("receipt_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        reject("aggregate receipt has a stale self hash")
    if value != expected:
        reject("aggregate receipt differs from the exact no-claim provenance observation")
    return copy.deepcopy(value)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--selector-cache-root", type=Path, required=True)
    parser.add_argument("--docs-cache-root", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, default=RECEIPT)
    parser.add_argument("--build", action="store_true")
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        value = build(args.selector_cache_root, args.docs_cache_root) if args.build else validate(
            args.receipt, args.selector_cache_root, args.docs_cache_root)
        print(json.dumps(value, indent=2, sort_keys=True) if args.emit_json else "PASS: raw provenance only; Vulkan matrix-incomplete")
    except ReceiptError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
