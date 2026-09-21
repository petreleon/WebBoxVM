#!/usr/bin/env python3
"""Verify a bounded buffer-binding state/lifecycle slice; never a Matrix input."""

import argparse
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
MANIFEST = HERE / "opengl_state_raw_inventory.json"
CACHE_PATH = HERE.parents[1] / "01-normative-pdf-cache/opengl_normative_pdf_cache.py"
ANCHOR_PATH = HERE / "opengl_state_raw_anchors.py"
LIMIT = 8 * 1024 * 1024


class InventoryError(ValueError):
    """The bounded raw inventory is stale, mixed or promoted."""


def private(path, name):
    if path.is_symlink() or not path.is_file():
        raise InventoryError("fixed dependency must be a regular nonsymlink file")
    prior = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            raise InventoryError("cannot load fixed dependency")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(module.__file__).resolve() != path.resolve():
            raise InventoryError("fixed dependency resolved to unexpected path")
        return module
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


CACHE = private(CACHE_PATH, "f03223_sealed_pdf_cache")
ANCHORS = private(ANCHOR_PATH, "f03223_reviewed_state_anchors")


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def seal(body):
    return {**body, "inventory_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def rendered(cache_root):
    try:
        receipt = CACHE.inspect(cache_root, ANCHORS.PREFIX + "84;section=6.1")
        boundary = CACHE.manifest()
        raw = CACHE.pdf_bytes(CACHE.external_root(cache_root), receipt["source"])
        facts = ANCHORS.facts_from_pages(ANCHORS.extract(raw))
        for fact in facts:
            if CACHE.admitted_source("command-object-state", fact["source_locator"]) != receipt["source"]:
                raise InventoryError("mixed state source authority")
    except (CACHE.CacheError, ANCHORS.AnchorError) as error:
        raise InventoryError(str(error)) from error
    body = {"schema": 1, "kind": "webboxvm-opengl-buffer-binding-state-lifecycle-raw-slice",
            "profile": ANCHORS.PROFILE, "source_class": "command-object-state",
            "source": receipt["source"], "cache_boundary_sha256": boundary["cache_boundary_sha256"],
            "physical_pdf_pages": receipt["physical_pdf_pages"], "complete": False,
            "coverage_manifest": ANCHORS.coverage(), "facts": facts,
            "facts_sha256": hashlib.sha256(canonical(facts)).hexdigest(),
            "claims": CACHE.NO_CLAIMS, "matrix_rows": 0, "cts_executions": 0,
            "states": CACHE.STATES}
    return seal(body)


def pairs(items):
    result = {}
    for key, value in items:
        if key in result:
            raise InventoryError("duplicate JSON key")
        result[key] = value
    return result


def validate(cache_root, manifest_path=MANIFEST):
    try:
        raw = manifest_path.read_bytes()
        if len(raw) > LIMIT:
            raise InventoryError("manifest exceeds 8 MiB")
        value = json.loads(raw.decode(), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise InventoryError(f"manifest cannot be read: {error}") from error
    if not isinstance(value, dict):
        raise InventoryError("manifest must be a JSON object")
    body = {key: item for key, item in value.items() if key != "inventory_sha256"}
    if value != seal(body):
        raise InventoryError("manifest has a stale self hash")
    if (value.get("claims") != CACHE.NO_CLAIMS or value.get("matrix_rows") != 0
            or value.get("cts_executions") != 0 or value.get("complete") is not False):
        raise InventoryError("raw inventory promotes support, completion, Matrix or CTS")
    if value != rendered(cache_root):
        raise InventoryError("raw inventory is missing, mixed, reordered or differs from reviewed anchors")
    return value


def reject_matrix_row(row, cache_root, manifest_path=MANIFEST):
    validate(cache_root, manifest_path)
    raise InventoryError("raw state/lifecycle slice cannot create a Matrix v2 row")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, default=MANIFEST)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        value = rendered(args.cache_root) if args.emit_json else validate(args.cache_root, args.manifest)
        print(json.dumps(value, indent=2, sort_keys=True) if args.emit_json else
              f"PASS: {len(value['facts'])} bounded buffer-binding state/lifecycle facts; matrix-incomplete")
    except InventoryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
