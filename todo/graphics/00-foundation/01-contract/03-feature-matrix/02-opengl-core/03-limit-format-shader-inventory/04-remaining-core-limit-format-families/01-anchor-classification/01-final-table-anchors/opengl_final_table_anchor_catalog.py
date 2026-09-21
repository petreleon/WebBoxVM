#!/usr/bin/env python3
"""Build and validate the no-claim F03.2.3.4.1.1 final-table catalog."""

from __future__ import annotations

import argparse
import importlib.util
import re
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE_PATH = HERE / "opengl_final_table_anchor_source.py"


def source_api():
    if SOURCE_PATH.is_symlink() or not SOURCE_PATH.is_file():
        raise RuntimeError("fixed catalog source helper is unavailable")
    name, prior = "f032341_source", sys.modules.get("f032341_source")
    try:
        spec = importlib.util.spec_from_file_location(name, SOURCE_PATH)
        if spec is None or spec.loader is None:
            raise RuntimeError("cannot load fixed catalog source helper")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module; spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != SOURCE_PATH.resolve():
            raise RuntimeError("catalog source helper resolved from an unexpected path")
        return module
    finally:
        if prior is None: sys.modules.pop(name, None)
        else: sys.modules[name] = prior


SOURCE = source_api()
POLICY_API = SOURCE.private(HERE / "opengl_final_table_anchor_policy.py", "f032341_policy")
POLICY = POLICY_API.policy
EXPECTED_POLICY = POLICY_API.POLICY
PDF = SOURCE.private(HERE / "opengl_final_table_anchor_pdf.py", "f032341_pdf")
RECEIPT = HERE / "opengl_final_table_anchor_catalog.json"
FRAGMENTS = ((HERE / "opengl_final_table_anchor_cells_23_56_23_66.py", "f032341_first"),
             (HERE / "opengl_final_table_anchor_cells_23_67_23_74.py", "f032341_last"))
CLAIMS = {key: False for key in ("khronos_selector", "api_support", "conformance", "certification",
                                 "profile_support", "performance")}
ROUTES = ("covered", "eligible-unreviewed", "route-to-state", "shader-unadmitted", "extension-unadmitted",
          "out-of-domain")
ROUTE_DESTINATIONS = {
    "covered": frozenset(), "out-of-domain": frozenset(), "route-to-state": frozenset(("F03.2.2.3.2",)),
    "shader-unadmitted": frozenset(("F03.2.3.2",)), "extension-unadmitted": frozenset(("F03.2.3.2",)),
    "eligible-unreviewed": frozenset(("F03.2.3.4.2", "F03.2.3.4.3", "F03.2.3.4.4.1", "F03.2.3.4.5.1",
                                        "F03.2.3.4.5.2", "F03.2.3.4.5.3", "F03.2.3.4.5.4")),
}


def reject(message: str) -> None:
    SOURCE.reject(message)


def modules():
    values = []
    for path, name in FRAGMENTS:
        module = SOURCE.private(path, name)
        if not isinstance(getattr(module, "FRAGMENT_ID", None), str) or not isinstance(getattr(module, "ROWS", None), tuple):
            reject("catalog fragment has an invalid shape")
        values.append(module)
    return values


def records() -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    rows, parts, table_rows, order = [], [], Counter(), 0
    prior_page, prior_table = 0, ""
    for module in modules():
        cells = module.ROWS
        for item in cells:
            if (not isinstance(item, tuple) or len(item) != 6 or type(item[1]) is not int
                    or any(type(value) is not str or not value for index, value in enumerate(item) if index != 1)):
                reject("catalog cell has an invalid typed shape")
            table, page, column, cell, command, section = item
            if not re.fullmatch(r"23\.(5[6-9]|6[0-9]|70|72|73|74)", table) or column not in ("Get value", "Description"):
                reject("catalog cell escaped its assigned table or column")
            if page < prior_page or (page == prior_page and table < prior_table):
                reject("catalog cells are reordered")
            prior_page, prior_table, order = page, table, order + 1
            table_rows[table] += 1
            expected = EXPECTED_POLICY.get((table, cell)) if isinstance(EXPECTED_POLICY, dict) else None
            if expected is None:
                reject("catalog source cell has no explicit finite policy mapping")
            try: route, reason, destination = POLICY(table, cell)
            except ValueError as error: reject(str(error))
            if (not all(type(value) is str and value for value in (route, reason, destination))
                    or route not in ROUTES or destination not in ROUTE_DESTINATIONS[route]):
                reject("catalog policy returned an invalid route, reason, or destination")
            if (route, reason, destination) != expected:
                reject("catalog policy outcome differs from its explicit finite mapping")
            rows.append({"candidate_id": f"opengl46-table-{table.replace('.', '-')}-row-{table_rows[table]:02}",
                         "physical_page": page, "numeric_section": section, "source_locator": f"opengl46-core-pdf-v1:page={page};section={section}",
                         "table": table, "table_row": table_rows[table], "table_column": column, "cell_text": cell,
                         "get_command": command, "source_order": order, "route": route, "route_reason": reason,
                         "destination": destination})
        parts.append({"fragment_id": module.FRAGMENT_ID, "candidate_count": len(cells), "cells_sha256": SOURCE.sha256(list(cells)),
                      "first_table": cells[0][0], "last_table": cells[-1][0]})
    if (len(rows) != 159 or [row["source_order"] for row in rows] != list(range(1, 160))
            or set(EXPECTED_POLICY) != {(row["table"], row["cell_text"]) for row in rows}):
        reject("catalog has missing, duplicate, or reordered source candidates")
    return rows, parts


def anchored(rows: list[dict[str, object]], raw: bytes, pages: int) -> None:
    PDF.anchored(rows, raw, pages, SOURCE, reject)


def built(cache_root: Path) -> tuple[dict[str, object], list[dict[str, object]]]:
    input_data, rows, fragments = SOURCE.inputs(cache_root), *records()
    anchored(rows, input_data["raw"], input_data["pages"])
    counts = {route: sum(row["route"] == route for row in rows) for route in ROUTES}
    body = {"schema": 1, "kind": "webboxvm-opengl46-final-table-anchor-classification", "profile": SOURCE.PROFILE,
            "source_authority_boundary_sha256": input_data["authority"]["boundary_sha256"],
            "source_contract_sha256": input_data["authority"]["source_contract_sha256"], "inventory_lock_sha256": input_data["authority"]["inventory_lock_sha256"],
            "cache_boundary_sha256": input_data["cache"]["cache_boundary_sha256"], "physical_pdf_pages": input_data["pages"],
            "normative_record_id": input_data["source"]["record_id"], "normative_source_sha256": input_data["source"]["sha256"],
            "reviewed_inventory_sha256": input_data["reviewed"]["inventory_sha256"], "reviewed_facts_sha256": input_data["reviewed"]["facts_sha256"],
            "unadmitted_ledger_sha256": input_data["ledger"]["ledger_sha256"], "fragments": fragments,
            "excluded_table": "23.71-reviewed-by-F03.2.3.1", "candidate_count": len(rows), "candidate_rows_sha256": SOURCE.sha256(rows),
            "route_counts": counts, "classification_only": True, "semantic_fact_count": 0, "matrix_row_count": 0,
            "cts_executions": 0, "claims": CLAIMS}
    return ({**body, "catalog_sha256": SOURCE.sha256(body)}, rows)


def rendered(cache_root: Path) -> dict[str, object]:
    return built(cache_root)[0]


def validate(cache_root: Path, receipt_path: Path = RECEIPT) -> dict[str, object]:
    value = SOURCE.document(receipt_path)
    body = {key: item for key, item in value.items() if key != "catalog_sha256"}
    try: digest = SOURCE.sha256(body)
    except (TypeError, ValueError) as error: reject(f"catalog receipt cannot be hashed: {error}")
    if value.get("catalog_sha256") != digest:
        reject("catalog receipt has a stale self hash")
    if value != rendered(cache_root):
        reject("catalog receipt is stale, partial, reordered, cross-profile, or promoted")
    return value


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, default=RECEIPT)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        value, rows = built(args.cache_root) if args.emit_json else (validate(args.cache_root, args.receipt), [])
        print(SOURCE.json.dumps({"receipt": value, "candidates": rows}, indent=2, sort_keys=True) if args.emit_json
              else f"PASS: {value['candidate_count']} final-table candidates; classification-only")
    except SOURCE.CatalogError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
