#!/usr/bin/env python3
"""Build and validate the source-only F03.2.3.4.1.3 closed classification."""

from __future__ import annotations

import argparse
import importlib.util
import json
import re
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent

def fixed(path: Path, name: str):
    if path.is_symlink() or not path.is_file(): raise RuntimeError("fixed aggregate helper must be a regular file")
    prior = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None: raise RuntimeError("cannot load fixed aggregate helpers")
        module = importlib.util.module_from_spec(spec); sys.modules[name] = module; spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve(): raise RuntimeError("fixed aggregate helper resolved unexpectedly")
        return module
    finally:
        if prior is None: sys.modules.pop(name, None)
        else: sys.modules[name] = prior

SOURCE = fixed(HERE / "opengl_closed_anchor_source.py", "f0323413_source")
POLICY = SOURCE.private(HERE / "opengl_closed_anchor_policy.py", "f0323413_policy")
CATALOG = HERE.parent / "01-final-table-anchors" / "opengl_final_table_anchor_catalog.py"
MANIFEST = HERE.parent / "02-chapter-local-anchor-manifest" / "opengl_chapter_local_anchor_manifest.py"
RECEIPT, BUNDLE = HERE / "opengl_closed_anchor_classification.json", "opengl_closed_anchor_child_receipts.json"
LAYOUT = (("final-table-01", "opengl_closed_anchor_candidates_01.json", 0, 53),
          ("final-table-02", "opengl_closed_anchor_candidates_02.json", 53, 106),
          ("final-table-03", "opengl_closed_anchor_candidates_03.json", 106, 159),
          ("chapter-local-01", "opengl_closed_anchor_candidates_04.json", 159, 197))
PROFILE = "opengl-4.6-core"
IDENTITY_FIELDS = ("cache_boundary_sha256", "source_authority_boundary_sha256", "source_contract_sha256",
                   "inventory_lock_sha256", "normative_record_id", "normative_source_sha256",
                   "physical_pdf_pages", "reviewed_inventory_sha256", "reviewed_facts_sha256",
                   "unadmitted_ledger_sha256")
CLAIMS = {key: False for key in ("api_support", "certification", "conformance", "guest_browser_behavior",
                                 "implementation_behavior", "performance", "profile_support")}
ROW_FIELDS = frozenset(("aggregate_order", "candidate_id", "child", "child_source_order", "source_locator",
                        "route", "route_reason", "destination"))
LOCATOR = re.compile(r"^opengl46-core-pdf-v1:page=[1-9][0-9]*;section=[1-9][0-9]*(?:\.[1-9][0-9]*)*$")

def reject(message: str) -> None:
    SOURCE.reject(message)

def child_data(cache_root: Path):
    catalog, manifest = SOURCE.private(CATALOG, "f0323413_catalog"), SOURCE.private(MANIFEST, "f0323413_manifest")
    try:
        catalog_value = catalog.validate(cache_root); catalog_expected, catalog_rows = catalog.built(cache_root)
        manifest_value = manifest.validate(cache_root); manifest_expected, _ = manifest.rendered(cache_root)
        local_rows, _ = manifest.records()
    except Exception as error:
        reject(f"child receipt cannot be validated: {error}")
    if (catalog_value != catalog_expected or manifest_value != manifest_expected
            or catalog_value.get("candidate_count") != len(catalog_rows)
            or manifest_value.get("candidate_count") != len(local_rows)):
        reject("child receipt is stale, partial, reordered, or promoted")
    if (catalog_value.get("profile") != PROFILE or manifest_value.get("profile") != PROFILE
            or any(catalog_value.get(key) != manifest_value.get(key) for key in IDENTITY_FIELDS)):
        reject("child receipts do not share exact profile, cache, authority, reviewed, and ledger identities")
    return catalog_value, manifest_value, catalog_rows, local_rows

def route(route: object, reason: object, destination: object) -> None:
    bad = ("*", "wildcard", "catch-all", "catch all", "residual", "other")
    if (not all(isinstance(value, str) and value for value in (route, reason)) or not isinstance(destination, str)
            or route not in POLICY.ROUTES or destination not in POLICY.DESTINATIONS[route]
            or any(word in reason.casefold() for word in bad)):
        reject("candidate lacks one allowed fixed route, reason, or destination")

def rows(catalog_rows: list[dict[str, object]], local_rows: list[dict[str, object]]) -> list[dict[str, object]]:
    catalog_fields = frozenset("candidate_id physical_page numeric_section source_locator table table_row table_column cell_text get_command source_order route route_reason destination".split())
    local_fields = frozenset("candidate_id physical_page numeric_section source_locator anchor_kind table row_scope column_scope source_order candidate_reason classification".split())
    if (len(catalog_rows) != 159 or len(local_rows) != 38
            or set(POLICY.LOCAL_POLICY) != {item.get("candidate_id") for item in local_rows}):
        reject("child candidate universe is missing, duplicate, or reordered")
    result = []
    for child, incoming, fields in (("final-table-catalog", catalog_rows, catalog_fields),
                                    ("chapter-local-manifest", local_rows, local_fields)):
        for item in incoming:
            if set(item) != fields or (child == "chapter-local-manifest" and item.get("classification") != "unclassified"):
                reject("child candidate has an incomplete or promoted shape")
            outcome = (item["route"], item["route_reason"], item["destination"]) if child == "final-table-catalog" else POLICY.LOCAL_POLICY[item["candidate_id"]]
            route(*outcome)
            result.append({"aggregate_order": len(result) + 1, "candidate_id": item["candidate_id"], "child": child,
                           "child_source_order": item["source_order"], "source_locator": item["source_locator"],
                           "route": outcome[0], "route_reason": outcome[1], "destination": outcome[2]})
    check_rows(result)
    return result

def check_rows(value: list[dict[str, object]]) -> None:
    expected_children = ["final-table-catalog"] * 159 + ["chapter-local-manifest"] * 38
    if len(value) != 197 or [item.get("child") for item in value] != expected_children:
        reject("aggregate has a missing, duplicate, or reordered child candidate")
    if [item.get("child_source_order") for item in value[:159]] != list(range(1, 160)) or [item.get("child_source_order") for item in value[159:]] != list(range(1, 39)):
        reject("aggregate lost a child source order")
    identifiers = set()
    for order, item in enumerate(value, 1):
        if (set(item) != ROW_FIELDS or item.get("aggregate_order") != order or type(item.get("child_source_order")) is not int
                or not isinstance(item.get("candidate_id"), str) or item["candidate_id"] in identifiers
                or not isinstance(item.get("source_locator"), str) or not LOCATOR.fullmatch(item["source_locator"])):
            reject("aggregate candidate is duplicate, promoted, or lacks its immutable source locator")
        identifiers.add(item["candidate_id"]); route(item["route"], item["route_reason"], item["destination"])

def sealed(body: dict[str, object], key: str) -> dict[str, object]:
    return {**body, key: SOURCE.sha256(body)}

def bundle(catalog: dict[str, object], manifest: dict[str, object]) -> dict[str, object]:
    body = {"schema": 1, "kind": "webboxvm-opengl46-closed-anchor-child-receipts", "profile": PROFILE,
            "final_table_catalog": catalog, "chapter_local_manifest": manifest}
    return sealed(body, "child_receipts_sha256")

def fragments(value: list[dict[str, object]]):
    result = []
    for fragment_id, filename, start, stop in LAYOUT:
        candidates = value[start:stop]
        if len(candidates) != stop - start or len({item["child"] for item in candidates}) != 1:
            reject("aggregate fragment lost its fixed child boundary")
        body = {"schema": 1, "kind": "webboxvm-opengl46-closed-anchor-candidate-fragment", "profile": PROFILE,
                "fragment_id": fragment_id, "child": candidates[0]["child"], "candidates": candidates,
                "candidate_count": len(candidates), "candidates_sha256": SOURCE.sha256(candidates)}
        result.append((filename, sealed(body, "fragment_sha256")))
    return result

def rendered(cache_root: Path):
    catalog, manifest, catalog_rows, local_rows = child_data(cache_root); value = rows(catalog_rows, local_rows)
    child_bundle, parts = bundle(catalog, manifest), fragments(value)
    registry = [{"filename": filename, "fragment_id": part["fragment_id"], "child": part["child"],
                 "candidate_count": part["candidate_count"], "candidates_sha256": part["candidates_sha256"],
                 "fragment_sha256": part["fragment_sha256"]} for filename, part in parts]
    source_identities = {key: catalog[key] for key in IDENTITY_FIELDS}
    route_counts = {name: sum(item["route"] == name for item in value) for name in POLICY.ROUTES}
    destination_counts = dict(sorted(Counter(item["destination"] for item in value).items()))
    body = {"schema": 1, "kind": "webboxvm-opengl46-closed-anchor-classification", "profile": PROFILE,
            "source_identities": source_identities, "child_receipt_bundle": {"filename": BUNDLE,
            "child_count": 2, "child_receipts_sha256": child_bundle["child_receipts_sha256"]},
            "fragment_registry": registry, "candidate_count": len(value), "candidate_sequence_sha256": SOURCE.sha256(value),
            "route_counts": route_counts, "destination_counts": destination_counts, "classification_only": True,
            "semantic_fact_count": 0, "matrix_row_count": 0, "cts_executions": 0, "claims": CLAIMS}
    return sealed(body, "aggregate_sha256"), child_bundle, parts

def write_artifacts(cache_root: Path, artifact_dir: Path = HERE) -> None:
    if artifact_dir.is_symlink() or not artifact_dir.is_dir(): reject("aggregate artifact directory is invalid")
    receipt, child_bundle, parts = rendered(cache_root)
    SOURCE.write(artifact_dir / BUNDLE, json.dumps(child_bundle, indent=2, sort_keys=True) + "\n")
    for filename, part in parts: SOURCE.write(artifact_dir / filename, SOURCE.candidate_fragment_text(part))
    SOURCE.write(artifact_dir / RECEIPT.name, json.dumps(receipt, indent=2, sort_keys=True) + "\n")

def validate(cache_root: Path, receipt_path: Path = RECEIPT, artifact_dir: Path = HERE) -> dict[str, object]:
    value = SOURCE.document(receipt_path); body = {key: item for key, item in value.items() if key != "aggregate_sha256"}
    if value.get("aggregate_sha256") != SOURCE.sha256(body): reject("aggregate receipt has a stale self hash")
    expected, child_bundle, parts = rendered(cache_root)
    if value != expected or SOURCE.document(artifact_dir / BUNDLE) != child_bundle:
        reject("aggregate receipt or exact child receipt bundle is stale, partial, reordered, or promoted")
    for filename, part in parts:
        if SOURCE.document(artifact_dir / filename) != part: reject("aggregate candidate fragment is stale, partial, reordered, or promoted")
    return value

def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--write", action="store_true"); args = parser.parse_args()
    try:
        if args.write: write_artifacts(args.cache_root)
        value = validate(args.cache_root)
        print(f"PASS: {value['candidate_count']} closed source-only candidates; {value['route_counts']['eligible-unreviewed']} eligible-unreviewed")
    except SOURCE.AggregateError as error:
        print(f"FAIL: {error}", file=sys.stderr); raise SystemExit(2)


if __name__ == "__main__": main()
