#!/usr/bin/env python3
"""Validate F03.2.2.5.1's closed, raw-only OpenGL command-domain map."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE_PATH = HERE.parents[1] / "01-normative-pdf-cache/opengl_normative_pdf_cache.py"
BASELINE_PATH = HERE.parents[1] / "02-command-object-raw-inventory/opengl_command_object_raw_inventory.py"
RULES_PATH = HERE / "opengl_command_domain_rules.py"
ARTIFACTS_PATH = HERE / "opengl_command_domain_artifacts.py"
INDEX_PATH = HERE / "opengl_command_domain_index.py"
CROSSCHECKS_PATH = HERE / "opengl_command_domain_crosschecks.py"
CLAIMS = {key: False for key in ("khronos_selector", "api_support", "conformance", "certification",
                                 "profile_support", "performance")}
FORBIDDEN = frozenset(("requirement_kind", "status", "implementation_owner", "independent_test_plan",
                       "owner_task", "test_source_role", "evidence", "coverage", "api_support"))
class ClassificationError(ValueError):
    """The source-family map is stale, promoted, ambiguous, or incomplete."""
def reject(message: str) -> None:
    raise ClassificationError(message)


def private(path: Path, name: str):
    if path.is_symlink() or not path.is_file():
        reject("fixed private dependency must be a regular file")
    prior = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            reject("cannot load fixed private dependency")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            reject("fixed private dependency resolved from an unexpected path")
        return module
    except ClassificationError:
        raise
    except Exception as error:
        reject(f"cannot load fixed private dependency: {error}")
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior
CACHE = private(CACHE_PATH, "f032251_pdf_cache")
BASELINE = private(BASELINE_PATH, "f032251_direct_baseline")
RULES = private(RULES_PATH, "f032251_command_domain_rules")
ARTIFACTS = private(ARTIFACTS_PATH, "f032251_command_domain_artifacts")
INDEX = private(INDEX_PATH, "f032251_command_domain_index")
CROSSCHECKS = private(CROSSCHECKS_PATH, "f032251_command_domain_crosschecks")
CATALOG = HERE / ARTIFACTS.ROOT_NAME
canonical = ARTIFACTS.canonical


def source_input(cache_root: Path) -> tuple[dict[str, object], dict[str, object], dict[str, object], bytes]:
    locator = "opengl46-core-pdf-v1:page=32;section=2.2"
    try:
        authority = CACHE.SOURCE_API.authority()
        boundary, consumed = authority.validate(), authority.consume("command-object-state", locator)
        receipt, manifest = CACHE.inspect(cache_root, locator), CACHE.manifest()
        source = receipt["source"]
        raw = CACHE.pdf_bytes(CACHE.external_root(cache_root), source)
    except Exception as error:
        reject(str(error))
    if (not isinstance(boundary, dict) or not isinstance(consumed, dict) or not isinstance(source, dict)
            or boundary.get("normative_root") != source or consumed.get("source") != source
            or consumed.get("decision", {}).get("id") != "command-object-state"
            or manifest.get("source") != source or receipt.get("physical_pdf_pages") != RULES.PAGES):
        reject("F03.2.1 and F03.2.2.1 do not expose the exact admitted PDF")
    return source, manifest, boundary, raw


def baseline_input(cache_root: Path, source: dict[str, object]) -> dict[str, object]:
    try:
        value = BASELINE.validate(cache_root)
        families = value["coverage_manifest"]["families"]
        command_ids = [item["command_fact_id"] for item in families]
        fact_by_id = {item["fact_id"]: item for item in value["facts"]}
    except Exception as error:
        reject(str(error))
    if (value.get("source") != source or len(value.get("facts", [])) != 24 or len(command_ids) != 12
            or len(set(command_ids)) != 12 or value.get("matrix_row_count") != 0):
        reject("F03.2.2.2 direct-creation baseline is not exact")
    anchors = [{"fact_id": item, "source_locator": fact_by_id[item]["source_locator"],
                "physical_page": fact_by_id[item]["physical_page"]} for item in command_ids]
    return {"inventory_sha256": value["inventory_sha256"], "facts_sha256": value["facts_sha256"],
            "coverage_manifest_sha256": hashlib.sha256(canonical(value["coverage_manifest"])).hexdigest(),
            "direct_creation_command_count": len(command_ids), "direct_creation_command_ids": command_ids,
            "direct_creation_anchors": anchors}


def rendered_bundle(cache_root: Path) -> dict[str, dict[str, object]]:
    source, manifest, boundary, raw = source_input(cache_root)
    baseline = baseline_input(cache_root, source)
    try:
        assignments = CROSSCHECKS.baseline_assignments(baseline, RULES, reject)
        commands = RULES.command_rows(raw, RULES.PAGES, baseline, assignments)
        non_commands = RULES.non_command_rows()
    except RULES.RuleError as error:
        reject(str(error))
    routes = [{"id": key, "target": target, "kind": kind} for key, (target, kind) in RULES.ROUTES.items()]
    if {row["route"] for row in commands + non_commands} != set(RULES.ROUTES):
        reject("closed route catalog has an unused or implicit route")
    core = {"schema": 1, "kind": "webboxvm-opengl46-command-domain-classification", "profile": RULES.PROFILE,
            "source": source, "source_class": "command-object-state", "authority_boundary_sha256": boundary["boundary_sha256"],
            "source_contract_sha256": boundary["source_contract_sha256"], "inventory_lock_sha256": boundary["inventory_lock_sha256"],
            "cache_boundary_sha256": manifest["cache_boundary_sha256"], "cache_layout": manifest["cache_layout"],
            "physical_pdf_pages": RULES.PAGES, "index_omission_crosscheck": INDEX.verify(raw, RULES, CROSSCHECKS, reject),
            "serialized_size_limit": ARTIFACTS.MAX_SERIALIZED, "claims": CLAIMS, "cts_executions": 0,
            "matrix_row_count": 0, "states": CACHE.STATES}
    try:
        return ARTIFACTS.bundle(core, routes, baseline, commands, non_commands)
    except ValueError as error:
        reject(str(error))


def fences(value: dict[str, object]) -> None:
    rows = value.get("command_families", []) + value.get("non_command_families", [])
    if not isinstance(value.get("command_families"), list) or not isinstance(value.get("non_command_families"), list):
        reject("classification catalog has no family lists")
    if any(not isinstance(row, dict) or FORBIDDEN & set(row) or "catchall" in str(row.get("route", "")) for row in rows):
        reject("classification family has a forbidden promotion or catch-all route")
    baseline = value.get("baseline", {})
    baseline_ids = baseline.get("direct_creation_command_ids", []) if isinstance(baseline, dict) else []
    excluded = [item for row in value["command_families"] for item in row.get("excluded_baseline_command_ids", [])]
    if len(baseline_ids) != 12 or len(excluded) != 12 or set(excluded) != set(baseline_ids):
        reject("baseline command anchors are not excluded exactly once from remaining source families")
    crosscheck = value.get("index_omission_crosscheck", {})
    if (crosscheck.get("role") != "omission-crosscheck-only" or crosscheck.get("physical_page_range") != [801, 851]
            or crosscheck.get("checked_page_count") != 51):
        reject("index omission crosscheck was promoted, shortened, or omitted")
    if value.get("claims") != CLAIMS or value.get("cts_executions") != 0 or value.get("matrix_row_count") != 0:
        reject("classification catalog promotes claims, CTS, or Matrix rows")


def validate(cache_root: Path, catalog_path: Path = CATALOG) -> dict[str, object]:
    value = ARTIFACTS.validate(catalog_path, rendered_bundle(cache_root), reject)
    fences(value)
    return value


def reject_matrix_row(row: object, cache_root: Path, catalog_path: Path = CATALOG) -> None:
    validate(cache_root, catalog_path)
    reject("source-family classification cannot create a Matrix v2 row")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--catalog", type=Path, default=CATALOG)
    emitted = parser.add_mutually_exclusive_group()
    emitted.add_argument("--emit-json", action="store_true")
    emitted.add_argument("--emit-artifact", choices=ARTIFACTS.ALL_NAMES)
    args = parser.parse_args()
    try:
        bundle = rendered_bundle(args.cache_root)
        if args.emit_json or args.emit_artifact:
            name = args.emit_artifact or ARTIFACTS.ROOT_NAME
            print(ARTIFACTS.serialized(bundle[name]), end="")
        else:
            value = ARTIFACTS.validate(args.catalog, bundle, reject)
            fences(value)
            print(f"PASS: {len(value['command_families'])} closed command families; matrix-incomplete")
    except ClassificationError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
