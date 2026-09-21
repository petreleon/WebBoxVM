#!/usr/bin/env python3
"""Validate the bounded F03.2.2.2 raw OpenGL command/object inventory."""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE_PATH = HERE.parent / "01-normative-pdf-cache/opengl_normative_pdf_cache.py"
RULES_PATH = HERE / "opengl_command_object_raw_rules.py"
INVENTORY = HERE / "opengl_command_object_raw_inventory.json"
MAX_SERIALIZED = 8 * 1024 * 1024
CLAIMS = {key: False for key in ("khronos_selector", "api_support", "conformance", "certification",
                                 "profile_support", "performance")}
FORBIDDEN = frozenset(("requirement_kind", "status", "implementation_owner", "independent_test_plan",
                       "owner_task", "test_source_role", "evidence", "coverage"))


class InventoryError(ValueError):
    """The bounded raw source inventory is unsafe or promoted."""


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


CACHE = private(CACHE_PATH, "f03222_pdf_cache")
RULES = private(RULES_PATH, "f03222_raw_rules")


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
            reject("inventory exceeds the 8 MiB serialized cap")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"inventory cannot be read: {error}")
    if not isinstance(value, dict):
        reject("inventory is not a JSON object")
    return value


def verified_source(cache_root: Path) -> tuple[dict[str, object], dict[str, object], bytes]:
    try:
        authority = CACHE.authority()
        if getattr(authority, "LOCATOR_SYNTAX", None) != RULES.LOCATOR_SYNTAX:
            reject("F03.2.1 locator grammar differs from the bounded raw rules")
        receipt = CACHE.inspect(cache_root, RULES.PREFIX_LOCATOR)
        source, manifest = receipt["source"], CACHE.manifest()
        if (source.get("profile") != RULES.PROFILE or source.get("record_id") != "opengl-46-core-spec"
                or receipt.get("physical_pdf_pages") != RULES.PAGES or manifest.get("source") != source):
            reject("F03.2.2.1 did not return the exact admitted OpenGL PDF identity")
        raw = CACHE.pdf_bytes(CACHE.external_root(cache_root), source)
    except (CACHE.CacheError, KeyError, TypeError) as error:
        reject(str(error))
    return source, manifest, raw


def rendered(cache_root: Path) -> dict[str, object]:
    source, manifest, raw = verified_source(cache_root)
    try:
        facts, coverage = RULES.verify_anchors(raw)
        authority = CACHE.authority()
        if any(not authority.LOCATOR.fullmatch(row["source_locator"]) for row in facts):
            reject("raw fact does not preserve the F03.2.1 locator grammar")
    except (RULES.RuleError, KeyError, TypeError) as error:
        reject(str(error))
    body = {"schema": 1, "kind": "webboxvm-opengl46-bounded-command-object-raw-inventory",
            "profile": RULES.PROFILE, "source_class": "command-object-state", "source": source,
            "cache_boundary_sha256": manifest["cache_boundary_sha256"], "cache_layout": manifest["cache_layout"],
            "physical_pdf_pages": RULES.PAGES, "serialized_size_limit": MAX_SERIALIZED,
            "coverage_manifest": coverage, "facts": facts,
            "facts_sha256": hashlib.sha256(canonical(facts)).hexdigest(), "claims": CLAIMS,
            "cts_executions": 0, "matrix_row_count": 0, "states": CACHE.STATES}
    value = {**body, "inventory_sha256": hashlib.sha256(canonical(body)).hexdigest()}
    if len(canonical(value)) > MAX_SERIALIZED:
        reject("inventory exceeds the 8 MiB serialized cap")
    return value


def fences(value: dict[str, object]) -> None:
    rows = value.get("facts")
    if not isinstance(rows, list):
        reject("inventory has no raw facts")
    if any(not isinstance(row, dict) or FORBIDDEN & set(row) for row in rows):
        reject("raw source fact has a forbidden Matrix v2 or promotion shape")
    if value.get("matrix_row_count") != 0 or value.get("cts_executions") != 0 or value.get("claims") != CLAIMS:
        reject("raw source inventory promotes claims, CTS, or Matrix rows")


def validate(cache_root: Path, inventory_path: Path = INVENTORY) -> dict[str, object]:
    value = document(inventory_path)
    body = {key: item for key, item in value.items() if key != "inventory_sha256"}
    if value.get("inventory_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        reject("inventory has a stale self hash")
    fences(value)
    if value != rendered(cache_root):
        reject("inventory is stale, mixed, partial, reordered, cross-profile, or incorrectly anchored")
    return copy.deepcopy(value)


def reject_matrix_row(row: object, cache_root: Path, inventory_path: Path = INVENTORY) -> None:
    validate(cache_root, inventory_path)
    reject("bounded raw source inventory cannot create a Matrix v2 row")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--inventory", type=Path, default=INVENTORY)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        value = rendered(args.cache_root) if args.emit_json else validate(args.cache_root, args.inventory)
        if args.emit_json:
            print(json.dumps(value, sort_keys=True, separators=(",", ":")))
        else:
            print(f"PASS: {len(value['facts'])} bounded raw command/object facts; matrix-incomplete")
    except InventoryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
