#!/usr/bin/env python3
"""Validate the raw-only F03.3.2.2.3.1 GLES declaration slice."""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
RAW = HERE.parents[1]
DOMAIN = RAW / "01-command-domain-classification/gles_command_domain_classification.py"
ARTIFACT_PATH = RAW / "01-command-domain-classification/gles_command_domain_artifact.py"
GRAMMAR_PATH = RAW / "02-template-declaration-grammar/gles_declaration_grammar.py"
CACHE_PATH = RAW.parent / "01-normative-pdf-cache/gles_normative_pdf_cache.py"
CATALOG_PATH = HERE / "gles_generic_sync_query_raw_catalog.py"
INVENTORY = HERE / "gles_generic_sync_query_raw_inventory.json"
FACT_KEYS = frozenset(("raw_id", "family_id", "source_family_order", "unprefixed_name", "c_name", "declaration",
                       "physical_page", "section", "source_locator", "source_order", "derivation_class"))


class InventoryError(ValueError):
    """The generic/sync/query raw declaration slice is not safely reproducible."""


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


CATALOG = private(CATALOG_PATH, "f0332231_gles_generic_catalog")
ARTIFACT = private(ARTIFACT_PATH, "f0332231_gles_artifact")
DOMAIN_API = private(DOMAIN, "f0332231_gles_domain")
GRAMMAR = private(GRAMMAR_PATH, "f0332231_gles_grammar")
CACHE = private(CACHE_PATH, "f0332231_gles_cache")
canonical, exact = ARTIFACT.canonical, ARTIFACT.exact


def artifact(callable, *args):
    try:
        return callable(*args)
    except ARTIFACT.ArtifactError as error:
        reject(str(error))


def source_input(cache_root: Path):
    locator = "gles32-pdf-v1:page=31;section=2.3"
    try:
        authority = CACHE.SOURCE_API.authority().validate()
        decision = CACHE.SOURCE_API.authority().consume("command-state", locator)["decision"]
        receipt, manifest = CACHE.inspect(cache_root, locator, "command-state"), CACHE.manifest()
        source = receipt["source"]
        raw = CACHE.pdf_bytes(CACHE.external_root(cache_root), source)
        domain, grammar = DOMAIN_API.validate(cache_root), GRAMMAR.validate(cache_root)
        families = CATALOG.bound_families(DOMAIN_API.CHUNKS, ARTIFACT.document)
    except Exception as error:
        reject(str(error))
    if (not exact(authority.get("normative_root"), source) or not exact(receipt.get("source"), source)
            or not exact(manifest.get("source"), source) or decision.get("id") != "command-state"
            or receipt.get("physical_pdf_pages") != CATALOG.PAGES or not exact(domain.get("source"), source)
            or domain.get("source_class") != "command-state" or not exact(grammar.get("source"), source)
            or grammar.get("domain_classification_sha256") != domain.get("classification_sha256")):
        reject("authority, cache, domain, or grammar identity is not the exact GLES source root")
    return source, authority, decision, manifest, domain, grammar, families, raw


def rendered(cache_root: Path) -> dict[str, object]:
    source, authority, decision, manifest, domain, grammar, families, raw = source_input(cache_root)
    try:
        facts = CATALOG.facts(raw, CATALOG.PAGES, families, GRAMMAR.normalize)
    except CATALOG.CatalogError as error:
        reject(str(error))
    family_rows = [{"id": identifier, "source_order": order} for identifier, order in sorted(families.items(), key=lambda row: row[1])]
    body = {"schema": 1, "kind": "webboxvm-gles32-generic-sync-query-raw-inventory", "profile": CATALOG.PROFILE,
            "source": source, "source_class": "command-state", "source_decision": decision,
            "source_authority_sha256": authority["boundary_sha256"], "source_contract_sha256": authority["source_contract_sha256"],
            "inventory_lock_sha256": authority["inventory_lock_sha256"], "cache_boundary_sha256": manifest["cache_boundary_sha256"],
            "physical_pdf_pages": CATALOG.PAGES, "domain_classification_sha256": domain["classification_sha256"],
            "grammar_sha256": grammar["grammar_sha256"], "domain_families": family_rows, "raw_entries": facts,
            "raw_entry_count": len(facts), "raw_entries_sha256": hashlib.sha256(canonical(facts)).hexdigest(),
            "raw_only": True, "promotion_allowed": False, "scope_fence": "formal-declaration-only"}
    return {**body, "inventory_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate(cache_root: Path, inventory_path: Path = INVENTORY) -> dict[str, object]:
    expected, actual = rendered(cache_root), artifact(ARTIFACT.document, inventory_path)
    artifact(ARTIFACT.self_hashed, actual, "inventory_sha256")
    artifact(ARTIFACT.forbidden, actual)
    if not exact(actual, expected):
        reject("generic/sync/query inventory is stale, incomplete, rerouted, or promoted")
    if not all(set(row) == FACT_KEYS for row in actual["raw_entries"]):
        reject("raw declaration fields are incomplete or promoted")
    return copy.deepcopy(actual)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--inventory", type=Path, default=INVENTORY)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        value = rendered(args.cache_root) if args.emit_json else validate(args.cache_root, args.inventory)
        print(json.dumps(value, indent=2, sort_keys=True) if args.emit_json else "PASS: 19 bounded GLES generic/sync/query declarations; source-only")
    except InventoryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
