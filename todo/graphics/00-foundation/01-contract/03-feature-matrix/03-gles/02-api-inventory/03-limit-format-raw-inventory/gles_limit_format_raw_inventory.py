#!/usr/bin/env python3
"""Validate the bounded F03.3.2.3 raw GLES limit/format inventory."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CATALOG_PATH = HERE / "gles_limit_format_raw_catalog.py"
INVENTORY = HERE / "gles_limit_format_raw_inventory.json"
MAX_SERIALIZED = 1024 * 1024
FACT_KEYS = frozenset(("raw_id", "name", "kind", "physical_page", "numeric_section", "source_locator",
                       "table", "table_row", "source_order", "derivation_class"))
FORBIDDEN = frozenset(("api_support", "profile_support", "support", "supported", "status", "claims",
                       "implementation_owner", "owner", "owner_task", "independent_test_plan", "test",
                       "tests", "reference_test", "cts", "cts_executions", "matrix_row_count", "matrix_rows",
                       "matrix_v2", "requirement_kind", "evidence", "conformance", "certification", "performance"))


class InventoryError(ValueError):
    """The raw inventory is stale, outside its boundary, or promoted."""


def reject(message: str) -> None:
    raise InventoryError(message)


def private(path: Path, name: str):
    if path.is_symlink() or not path.is_file():
        reject("fixed private catalog must be a regular file")
    prior = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            reject("cannot load fixed private catalog")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            reject("fixed private catalog resolved from an unexpected path")
        return module
    except InventoryError:
        raise
    except Exception as error:
        reject(f"cannot load fixed private catalog: {error}")
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


CATALOG = private(CATALOG_PATH, "f03323_gles_limit_format_catalog")


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def exact(left: object, right: object) -> bool:
    if type(left) is not type(right): return False
    if isinstance(left, dict): return set(left) == set(right) and all(exact(left[key], right[key]) for key in left)
    if isinstance(left, list): return len(left) == len(right) and all(exact(a, b) for a, b in zip(left, right))
    return left == right


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
        facts, coverage = CATALOG.facts(input_data["raw"], input_data["physical_pdf_pages"]), CATALOG.coverage()
    except (CATALOG.InventoryError, KeyError, TypeError) as error:
        reject(str(error))
    if coverage.get("complete") is not False or len(facts) != 34:
        reject("bounded raw catalog cannot represent complete or partial source coverage")
    provenance = {key: input_data[key] for key in ("cache_boundary_sha256", "cache_layout", "physical_pdf_pages")}
    body = {"schema": 1, "kind": "webboxvm-gles32-bounded-limit-format-raw-inventory", "profile": CATALOG.PROFILE,
            "source_class": CATALOG.LIMIT_CLASS, "source": input_data["source"], "source_decision": input_data["decision"],
            "source_authority_boundary_sha256": input_data["source_authority_boundary_sha256"],
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
                reject("raw inventory contains a Matrix, test, support, or qualification claim")
            forbidden(item)
    elif isinstance(value, list):
        for item in value:
            forbidden(item)


def validate(cache_root: Path, inventory_path: Path = INVENTORY) -> dict[str, object]:
    value = document(inventory_path)
    body = {key: item for key, item in value.items() if key != "inventory_sha256"}
    if value.get("inventory_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        reject("inventory has a stale self hash")
    forbidden(value)
    expected = rendered(cache_root)
    if not exact(value, expected):
        reject("inventory differs from the exact sealed GLES table slice")
    if not all(set(row) == FACT_KEYS for row in value["facts"]):
        reject("raw facts have missing or promoted fields")
    return value


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--inventory", type=Path, default=INVENTORY)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        if args.emit_json:
            print(json.dumps(rendered(args.cache_root), indent=2, sort_keys=True))
        else:
            validate(args.cache_root, args.inventory)
            print("PASS: 34 bounded GLES raw limit/format facts; matrix-incomplete")
    except InventoryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
