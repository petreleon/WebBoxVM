#!/usr/bin/env python3
"""Verify only the sealed GLES 3.2 normative PDF in an external cache."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
HELPERS = HERE / "gles_normative_pdf_cache_source.py"
MANIFEST = HERE / "gles_normative_pdf_cache.json"


def helpers():
    name, prior = "f03321_gles_pdf_helpers", sys.modules.get("f03321_gles_pdf_helpers")
    if HELPERS.is_symlink() or not HELPERS.is_file():
        raise RuntimeError("fixed cache helpers must be a regular file")
    spec = importlib.util.spec_from_file_location(name, HELPERS)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load fixed cache helpers")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    try:
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != HELPERS.resolve():
            raise RuntimeError("cache helpers resolved from an unexpected path")
        return module
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


SOURCE_API = helpers()
CacheError, reject, canonical, pairs = (SOURCE_API.CacheError, SOURCE_API.reject, SOURCE_API.canonical, SOURCE_API.pairs)
admitted_source, external_root = SOURCE_API.admitted_source, SOURCE_API.external_root
cache_file, pdf_bytes, physical_pages, page_number = (SOURCE_API.cache_file, SOURCE_API.pdf_bytes,
                                                       SOURCE_API.physical_pages, SOURCE_API.page_number)
PROFILE, CLASSES, PAGES, LAYOUT, SOURCE = (SOURCE_API.PROFILE, SOURCE_API.CLASSES, SOURCE_API.PAGES,
                                            SOURCE_API.LAYOUT, SOURCE_API.SOURCE)
NO_CLAIMS, STATES, AUTHORITY, REPO = SOURCE_API.NO_CLAIMS, SOURCE_API.STATES, SOURCE_API.AUTHORITY, SOURCE_API.REPO


def expected() -> dict[str, object]:
    sources = {kind: admitted_source(kind, f"gles32-pdf-v1:page=1;section=1") for kind in CLASSES}
    if any(source != SOURCE for source in sources.values()):
        reject("F03.3.1 did not preserve the two admitted GLES source classes")
    body = {"schema": 1, "kind": "webboxvm-gles-normative-pdf-cache", "profile": PROFILE,
            "source_classes": list(CLASSES), "source": SOURCE, "cache_layout": LAYOUT,
            "physical_pdf_pages": PAGES, "claims": NO_CLAIMS, "cts_executions": 0,
            "matrix_rows": 0, "states": STATES}
    return {**body, "cache_boundary_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def manifest(path: Path = MANIFEST) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"cache manifest cannot be read: {error}")
    body = dict(value) if isinstance(value, dict) else {}
    digest = body.pop("cache_boundary_sha256", None)
    if digest != hashlib.sha256(canonical(body)).hexdigest() or value != expected():
        reject("cache manifest is stale or differs from the sealed no-claim boundary")
    return value


def inspect(cache_root: Path, locator: str, locator_class: str = CLASSES[0],
            manifest_path: Path = MANIFEST) -> dict[str, object]:
    value, source = manifest(manifest_path), admitted_source(locator_class, locator)
    pages = physical_pages(pdf_bytes(external_root(cache_root), source))
    if page_number(locator) > pages:
        reject("locator physical page exceeds the sealed PDF")
    return {"source": source, "locator": locator, "physical_pdf_pages": pages,
            "claims": value["claims"], "cts_executions": 0, "matrix_rows": 0, "states": value["states"]}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path)
    parser.add_argument("--locator", default="gles32-pdf-v1:page=1;section=1")
    parser.add_argument("--source-class", default=CLASSES[0])
    parser.add_argument("--manifest", type=Path, default=MANIFEST)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        if args.emit_json:
            print(json.dumps(expected(), indent=2, sort_keys=True))
        elif args.cache_root is None:
            reject("--cache-root is required unless --emit-json is used")
        else:
            result = inspect(args.cache_root, args.locator, args.source_class, args.manifest)
            print(f"PASS: GLES normative PDF {result['physical_pdf_pages']} physical pages; matrix-incomplete")
    except CacheError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
