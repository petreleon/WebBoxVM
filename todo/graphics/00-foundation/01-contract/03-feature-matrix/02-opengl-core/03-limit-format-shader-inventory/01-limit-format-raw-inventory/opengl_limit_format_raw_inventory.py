#!/usr/bin/env python3
"""Validate the bounded F03.2.3.1 raw OpenGL limit/format inventory."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CATALOG_PATH = HERE / "opengl_limit_format_raw_catalog.py"
INVENTORY = HERE / "opengl_limit_format_raw_inventory.json"
MAX_SERIALIZED = 1024 * 1024
FACT_KEYS = frozenset(("raw_id", "name", "kind", "physical_page", "numeric_section", "source_locator",
                       "table", "table_row", "source_order", "derivation_class"))
FORBIDDEN = frozenset(("api_support", "profile_support", "support", "supported", "status", "claims",
                       "implementation_owner", "owner", "owner_task", "independent_test_plan", "test",
                       "tests", "reference_test", "cts", "cts_executions", "matrix_row_count", "matrix_rows",
                       "matrix_v2", "requirement_kind", "evidence", "conformance", "certification", "performance"))


class InventoryError(ValueError):
    """The raw data is stale, outside its boundary, or promoted."""


def reject(message: str) -> None:
    raise InventoryError(message)


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
    except InventoryError:
        raise
    except Exception as error:
        reject(f"cannot load fixed private dependency: {error}")
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


CATALOG = private(CATALOG_PATH, "f03231_limit_format_catalog")


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def pairs(items):
    value = {}
    for key, item in items:
        if key in value:
            reject("inventory has duplicate JSON fields")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    try:
        raw = path.read_bytes()
        if len(raw) > MAX_SERIALIZED:
            reject("inventory exceeds its serialized-size cap")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"inventory cannot be read: {error}")
    if not isinstance(value, dict):
        reject("inventory is not a JSON object")
    return value


def rendered(cache_root: Path) -> dict[str, object]:
    try:
        input_data = CATALOG.source_input(cache_root)
        facts = CATALOG.facts(input_data["raw"], input_data["physical_pdf_pages"])
        coverage = CATALOG.coverage()
    except (CATALOG.InventoryError, KeyError, TypeError) as error:
        reject(str(error))
    if coverage.get("complete") is not False or not facts:
        reject("bounded raw catalog cannot represent complete or empty coverage")
    provenance = {key: input_data[key] for key in ("cache_boundary_sha256", "cache_layout", "physical_pdf_pages")}
    body = {"schema": 1, "kind": "webboxvm-opengl46-bounded-limit-format-raw-inventory",
            "profile": CATALOG.PROFILE, "source_class": CATALOG.LIMIT_CLASS, "source": input_data["source"],
            "source_decision": input_data["decision"], "source_authority_boundary_sha256": input_data["source_authority_boundary_sha256"],
            "source_contract_sha256": input_data["source_contract_sha256"], "inventory_lock_sha256": input_data["inventory_lock_sha256"],
            "cache_provenance": provenance, "serialized_size_limit": MAX_SERIALIZED, "coverage_manifest": coverage,
            "facts": facts, "raw_fact_count": len(facts), "facts_sha256": hashlib.sha256(canonical(facts)).hexdigest(),
            "raw_only": True, "promotion_allowed": False}
    value = {**body, "inventory_sha256": hashlib.sha256(canonical(body)).hexdigest()}
    if len(canonical(value)) > MAX_SERIALIZED:
        reject("inventory exceeds its serialized-size cap")
    return value


def forbidden(value: object) -> None:
    if isinstance(value, dict):
        for key, item in value.items():
            if key in FORBIDDEN:
                reject("raw inventory contains a forbidden promotion field")
            forbidden(item)
    elif isinstance(value, list):
        for item in value:
            forbidden(item)


def fences(value: dict[str, object]) -> None:
    forbidden(value)
    facts, coverage = value.get("facts"), value.get("coverage_manifest")
    if (value.get("profile") != CATALOG.PROFILE or value.get("source_class") != CATALOG.LIMIT_CLASS
            or value.get("raw_only") is not True or value.get("promotion_allowed") is not False
            or not isinstance(coverage, dict) or coverage.get("complete") is not False):
        reject("inventory escaped its bounded raw-only profile")
    if not isinstance(facts, list) or value.get("raw_fact_count") != len(facts) or not facts:
        reject("inventory has incomplete raw facts")
    if any(not isinstance(row, dict) or set(row) != FACT_KEYS or row.get("derivation_class") != CATALOG.LIMIT_CLASS
           for row in facts):
        reject("raw fact has an invalid or promoted shape")
    ids, orders = [row["raw_id"] for row in facts], [row["source_order"] for row in facts]
    if len(set(ids)) != len(ids) or orders != list(range(1, len(facts) + 1)):
        reject("raw facts are duplicate, incomplete, or reordered")
    if value.get("facts_sha256") != hashlib.sha256(canonical(facts)).hexdigest():
        reject("raw facts have a stale self hash")


def validate(cache_root: Path, inventory_path: Path = INVENTORY) -> dict[str, object]:
    value = document(inventory_path)
    body = {key: item for key, item in value.items() if key != "inventory_sha256"}
    if value.get("inventory_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        reject("inventory has a stale self hash")
    fences(value)
    if value != rendered(cache_root):
        reject("inventory is stale, mixed, partial, reordered, cross-profile, or incorrectly anchored")
    return value


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--inventory", type=Path, default=INVENTORY)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        value = rendered(args.cache_root) if args.emit_json else validate(args.cache_root, args.inventory)
        print(json.dumps(value, sort_keys=True, separators=(",", ":")) if args.emit_json
              else f"PASS: {len(value['facts'])} bounded OpenGL raw facts; raw-only")
    except InventoryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
