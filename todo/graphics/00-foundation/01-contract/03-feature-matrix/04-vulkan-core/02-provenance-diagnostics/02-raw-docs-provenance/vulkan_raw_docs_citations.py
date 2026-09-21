#!/usr/bin/env python3
"""Bind reviewed raw Vulkan-Docs fragments as citation candidates only."""

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
    path, name = HERE / "vulkan_raw_docs_citation_sources.py", "f03422_fixed_map_sources"
    prior = sys.modules.get(name)
    try:
        if path.is_symlink() or not path.is_file():
            raise RuntimeError("fixed citation map sources must be a regular file")
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            raise RuntimeError("cannot load fixed citation map sources")
        module = importlib.util.module_from_spec(spec); sys.modules[name] = module; spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            raise RuntimeError("citation map sources resolved from an unexpected path")
        return module
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


SOURCE, CACHE = fixed_source(), None
CACHE = SOURCE.cache_api()
CitationError, reject, canonical, document = SOURCE.CitationSourceError, SOURCE.reject, SOURCE.canonical, SOURCE.document
MAP = HERE / "vulkan_raw_docs_citations.json"
PROFILE, MAX_BYTES = "vulkan-1.4-core", 8 * 1024 * 1024
CLAIMS = {name: False for name in ("khronos_selector", "api_support", "conformance", "certification", "profile_support", "performance")}
STATES = {"mandatory_role_inventory_complete": True, "profile_status": "blocked", "blocker": "matrix-incomplete",
          "api_support": False, "conformance": False, "certification": False, "profile_support": False, "performance": False}
EXPECTED_BINDING = {"profile": PROFILE, "role": "normative-root", "record_id": "vulkan-14-spec",
                    "record_kind": "upstream-source", "scope": "normative-source", "revision": CACHE.REVISION,
                    "sha256": CACHE.ROOT["sha256"], "bytes": CACHE.ROOT["bytes"]}
BOUNDARY_SHA256 = "dd07b39283a3e41edf217e20dbb1eec90e8e6e73b6aae748d46bb57588983683"


def source_inputs(cache_root: Path):
    api = SOURCE.bindings_api()
    try:
        contract, binding, boundary = api.load_locked_source_contract(), None, SOURCE.channel_api().validate()
        binding = api.binding_for(contract, PROFILE, "normative-root")
    except Exception as error:
        reject(f"cannot load the fixed F03 normative role: {error}")
    normative, registry = (boundary.get("normative_root"), boundary.get("registry")) if isinstance(boundary, dict) else (None, None)
    blocked_registry = {"status": "blocked", "implementation_owner": None, "independent_test_plan": None}
    if (binding != EXPECTED_BINDING or not isinstance(normative, dict) or normative.get("binding") != binding
            or normative.get("identity") != CACHE.ROOT or normative.get("permitted_use") != "requires-separately-pinned-citation-map"
            or boundary.get("boundary_sha256") != BOUNDARY_SHA256
            or boundary.get("docs_include_closure") != "unadmitted-requires-separately-pinned-citation-map"
            or boundary.get("matrix_row_count") != 0 or boundary.get("claims") != CLAIMS or boundary.get("cts_executions") != 0
            or boundary.get("states") != STATES or not isinstance(registry, dict) or registry.get("row_policy") != blocked_registry
            or registry.get("scaffold_row_count") != 0):
        reject("F03 boundary does not expose the exact self-hashed citation-only channel")
    try:
        cache = CACHE.cache(cache_root)
    except Exception as error:
        reject(f"reviewed raw Docs cache cannot be verified: {error}")
    return contract, binding, boundary, cache


def candidates() -> list[dict[str, object]]:
    return [{"id": item["id"], "normative_root": copy.deepcopy(CACHE.ROOT),
             "raw_fragment": copy.deepcopy(item), "relation": "same-revision-raw-fragment; inclusion-unproven",
             "semantic_status": "citation-candidate-only", "core_coverage": "unassigned"}
            for item in CACHE.FRAGMENTS]


def build(cache_root: Path) -> dict[str, object]:
    contract, binding, boundary, cache = source_inputs(cache_root)
    values = candidates()
    body = {"schema": 1, "kind": "webboxvm-vulkan-raw-docs-citation-map", "authority": "WebBoxVM", "producer": "WebBoxVM",
            "profile": PROFILE, "source_contract_sha256": contract["source_contract_sha256"],
            "inventory_lock_sha256": contract["inventory_lock_sha256"], "source_channel_boundary_sha256": boundary["boundary_sha256"],
            "normative_binding": binding, "citation_candidates": values, "citation_candidate_count": len(values), "cache": cache,
            "scope": {"map_role": "citation-only", "include_closure": "unadmitted", "rendered_docs": "forbidden",
                      "inferred_symbol_name_locators": "forbidden", "core_coverage": "not-established", "matrix_row_count": 0,
                      "registry_structural_fact": {"status": "blocked", "source_locator": None,
                                                    "reason": "no reviewed raw Docs semantic anchor"}},
            "matrix_rows": [], "claims": CLAIMS, "cts_executions": 0, "states": STATES}
    return {**body, "map_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate(path: Path = MAP, cache_root: Path | None = None) -> dict[str, object]:
    if cache_root is None:
        reject("an external reviewed raw Docs cache is required")
    expected, value = build(cache_root), document(path, "raw Docs citation map")
    body = {key: item for key, item in value.items() if key != "map_sha256"}
    if value.get("map_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        reject("raw Docs citation map has a stale self hash")
    if len(canonical(value)) > MAX_BYTES:
        reject("raw Docs citation map exceeds the 8 MiB artifact limit")
    if value != expected:
        reject("raw Docs citation map differs from the exact citation-only observation")
    return copy.deepcopy(value)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--map", type=Path, default=MAP)
    parser.add_argument("--refresh", action="store_true")
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        if args.refresh:
            CACHE.refresh(args.cache_root, args.timeout)
        value = validate(args.map, args.cache_root)
        print(json.dumps(value, sort_keys=True) if args.emit_json else "PASS: 2 raw Docs citation candidates; matrix-incomplete")
    except (CitationError, CACHE.CitationSourceError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
