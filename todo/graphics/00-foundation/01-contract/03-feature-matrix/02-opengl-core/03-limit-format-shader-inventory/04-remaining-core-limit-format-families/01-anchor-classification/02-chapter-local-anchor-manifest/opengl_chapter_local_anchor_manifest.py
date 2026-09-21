#!/usr/bin/env python3
"""Build and validate the no-claim F03.2.3.4.1.2 local-anchor manifest."""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE_PATH, CELLS_PATH = HERE / "opengl_chapter_local_anchor_source.py", HERE / "opengl_chapter_local_anchor_cells.py"
PDF_PATH, MANIFEST = HERE / "opengl_chapter_local_anchor_pdf.py", HERE / "opengl_chapter_local_anchor_manifest.json"
FRAGMENT_FILES = {"buffer": "opengl_chapter_local_anchor_buffer.json", "pixel": "opengl_chapter_local_anchor_pixel.json",
                  "texture-image": "opengl_chapter_local_anchor_texture_image.json", "texture-state": "opengl_chapter_local_anchor_texture_state.json",
                  "texture-views": "opengl_chapter_local_anchor_texture_views.json", "texture-framebuffer": "opengl_chapter_local_anchor_texture_framebuffer.json",
                  "vertex-pixel-query": "opengl_chapter_local_anchor_vertex_pixel_query.json"}
PROFILE = "opengl-4.6-core"
CLAIMS = {key: False for key in ("khronos_selector", "api_support", "conformance", "certification", "profile_support", "performance")}
CLOSED_IDS = tuple(["table-6-5"] + [f"table-8-{number}" for number in range(2, 28)] + [f"table-9-{number}" for number in range(1, 4)] + [f"table-10-{number}" for number in range(3, 7)] + ["table-18-2", "table-18-4", "section-22-3", "table-22-2"])
CLOSED_SPECS_SHA256 = "712e5fed2dbc9cc392a63d1e8f99ddab0dade95b92061d62d13b1310ef926699"
FORBIDDEN = frozenset(("route", "destination", "owner", "support", "supported", "status", "test", "tests", "cts", "performance", "conformance", "certification"))


def source_api():
    if SOURCE_PATH.is_symlink() or not SOURCE_PATH.is_file():
        raise RuntimeError("fixed manifest source helper is unavailable")
    name, prior = "f0323412_source", sys.modules.get("f0323412_source")
    try:
        spec = importlib.util.spec_from_file_location(name, SOURCE_PATH)
        if spec is None or spec.loader is None:
            raise RuntimeError("cannot load fixed manifest source helper")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module; spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != SOURCE_PATH.resolve():
            raise RuntimeError("manifest source helper resolved from an unexpected path")
        return module
    finally:
        if prior is None: sys.modules.pop(name, None)
        else: sys.modules[name] = prior


SOURCE = source_api()
CELLS = SOURCE.private(CELLS_PATH, "f0323412_cells")
PDF = SOURCE.private(PDF_PATH, "f0323412_pdf")


def reject(message: str) -> None:
    SOURCE.reject(message)


def scope(kind: str) -> tuple[dict[str, str], list[str], str]:
    if kind == "table":
        return ({"first": "caption", "last": "caption"}, ["table-number", "caption-text"], "roadmap-enumerated-table-anchor")
    if kind == "section":
        return ({"first": "heading", "last": "heading"}, ["section-number", "heading-text"], "roadmap-enumerated-section-anchor")
    reject("candidate has an unknown source-anchor kind")


def records() -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    fragments, rows, order = [], [], 0
    value = getattr(CELLS, "FRAGMENTS", None)
    if not isinstance(value, tuple) or SOURCE.sha256(value) != CLOSED_SPECS_SHA256:
        reject("closed local candidate vector was changed")
    for fragment_id, specs in value:
        if fragment_id not in FRAGMENT_FILES or not isinstance(specs, tuple) or not specs:
            reject("candidate fragment has an invalid shape")
        local = []
        for item in specs:
            if not isinstance(item, tuple) or len(item) != 5:
                reject("candidate specification has an invalid shape")
            identifier, kind, table, page, numeric = item
            if not all(isinstance(part, str) and part for part in (identifier, kind, numeric)) or type(page) is not int:
                reject("candidate specification has invalid types")
            if kind == "table" and not isinstance(table, str): reject("table candidate lacks its exact table scope")
            if kind == "section" and table is not None: reject("section candidate cannot carry a table scope")
            row_scope, column_scope, reason = scope(kind); order += 1
            local.append({"candidate_id": f"opengl46-local-{identifier}", "physical_page": page, "numeric_section": numeric,
                          "source_locator": f"opengl46-core-pdf-v1:page={page};section={numeric}", "anchor_kind": f"{kind}-caption" if kind == "table" else "section-heading",
                          "table": table, "row_scope": row_scope, "column_scope": column_scope, "source_order": order,
                          "candidate_reason": reason, "classification": "unclassified"})
        body = {"schema": 1, "kind": "webboxvm-opengl46-chapter-local-anchor-fragment", "profile": PROFILE,
                "fragment_id": fragment_id, "candidates": local, "candidate_count": len(local), "candidates_sha256": SOURCE.sha256(local)}
        fragments.append({**body, "fragment_sha256": SOURCE.sha256(body)})
        rows.extend(local)
    if [row["candidate_id"].removeprefix("opengl46-local-") for row in rows] != list(CLOSED_IDS) or len(rows) != 38:
        reject("candidate vector is missing, duplicate, or reordered")
    return rows, fragments


def fences(rows: list[dict[str, object]]) -> None:
    expected_table = ({"first": "caption", "last": "caption"}, ["table-number", "caption-text"])
    expected_section = ({"first": "heading", "last": "heading"}, ["section-number", "heading-text"])
    prior = 0
    for row in rows:
        if set(row) != {"candidate_id", "physical_page", "numeric_section", "source_locator", "anchor_kind", "table", "row_scope", "column_scope", "source_order", "candidate_reason", "classification"}:
            reject("candidate has an incomplete or promoted shape")
        table = row["anchor_kind"] == "table-caption"
        if (row["classification"] != "unclassified" or row["source_order"] != prior + 1
                or not isinstance(row["physical_page"], int) or not isinstance(row["numeric_section"], str)
                or row["source_locator"] != f"opengl46-core-pdf-v1:page={row['physical_page']};section={row['numeric_section']}"
                or (table and not isinstance(row["table"], str)) or (not table and row["table"] is not None)
                or (row["row_scope"], row["column_scope"]) != (expected_table if table else expected_section)
                or any(bad in str(row).lower() for bad in ("*", "wildcard", "all rows", "all columns"))
                or any(key in row for key in FORBIDDEN)):
            reject("candidate is broad, reordered, or promoted")
        prior += 1


def rendered(cache_root: Path) -> tuple[dict[str, object], list[dict[str, object]]]:
    data, rows, fragments = SOURCE.inputs(cache_root), *records()
    fences(rows); PDF.anchored(rows, data["raw"], data["pages"], SOURCE, reject)
    registry = [{"fragment_id": item["fragment_id"], "filename": FRAGMENT_FILES[item["fragment_id"]], "candidate_count": item["candidate_count"],
                 "candidates_sha256": item["candidates_sha256"], "fragment_sha256": item["fragment_sha256"]} for item in fragments]
    body = {"schema": 1, "kind": "webboxvm-opengl46-chapter-local-anchor-manifest", "profile": PROFILE,
            "source_authority_boundary_sha256": data["authority"]["boundary_sha256"], "source_contract_sha256": data["authority"]["source_contract_sha256"],
            "inventory_lock_sha256": data["authority"]["inventory_lock_sha256"], "cache_boundary_sha256": data["cache"]["cache_boundary_sha256"],
            "normative_record_id": data["source"]["record_id"], "normative_source_sha256": data["source"]["sha256"], "physical_pdf_pages": data["pages"],
            "reviewed_inventory_sha256": data["reviewed"]["inventory_sha256"], "reviewed_facts_sha256": data["reviewed"]["facts_sha256"],
            "unadmitted_ledger_sha256": data["ledger"]["ledger_sha256"], "fragment_registry": registry, "candidate_count": len(rows),
            "candidate_sequence_sha256": SOURCE.sha256(rows), "classification": "unclassified", "semantic_fact_count": 0,
            "matrix_row_count": 0, "cts_executions": 0, "claims": CLAIMS}
    return ({**body, "manifest_sha256": SOURCE.sha256(body)}, fragments)


def validate(cache_root: Path, manifest_path: Path = MANIFEST, fragment_dir: Path = HERE) -> dict[str, object]:
    value = SOURCE.document(manifest_path); body = {key: item for key, item in value.items() if key != "manifest_sha256"}
    if value.get("manifest_sha256") != SOURCE.sha256(body): reject("manifest has a stale self hash")
    expected, fragments = rendered(cache_root)
    if value != expected: reject("manifest is stale, partial, reordered, cross-profile, broad, or promoted")
    for item in fragments:
        path = fragment_dir / FRAGMENT_FILES[item["fragment_id"]]; actual = SOURCE.document(path)
        digest = {key: field for key, field in actual.items() if key != "fragment_sha256"}
        if actual.get("fragment_sha256") != SOURCE.sha256(digest) or actual != item:
            reject("manifest fragment is stale, partial, reordered, or promoted")
    return value


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, default=MANIFEST); parser.add_argument("--fragment-dir", type=Path, default=HERE)
    parser.add_argument("--emit-json", action="store_true"); args = parser.parse_args()
    try:
        value, fragments = rendered(args.cache_root) if args.emit_json else (validate(args.cache_root, args.manifest, args.fragment_dir), [])
        print(json.dumps({"manifest": value, "fragments": fragments}, indent=2, sort_keys=True) if args.emit_json else f"PASS: {value['candidate_count']} chapter-local candidates; unclassified-only")
    except SOURCE.ManifestError as error:
        print(f"FAIL: {error}", file=sys.stderr); raise SystemExit(2)


if __name__ == "__main__": main()
