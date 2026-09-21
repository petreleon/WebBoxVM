#!/usr/bin/env python3
"""Validate the raw-only F03.3.2.2.3.3.1 GLES declaration slice."""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
RAW = HERE.parents[2]
DOMAIN = RAW / "01-command-domain-classification/gles_command_domain_classification.py"
ARTIFACT_PATH = RAW / "01-command-domain-classification/gles_command_domain_artifact.py"
GRAMMAR_PATH = RAW / "02-template-declaration-grammar/gles_declaration_grammar.py"
CACHE_PATH = RAW.parent / "01-normative-pdf-cache/gles_normative_pdf_cache.py"
LEDGER_PATH = RAW.parent / "04-unavailable-language-extension-ledger/gles_unavailable_ledger.py"
CATALOG_PATH = HERE / "gles_program_pipeline_literal_raw_catalog.py"
INVENTORY = HERE / "gles_program_pipeline_literal_raw_inventory.json"
ENTRY_FIELDS = ("raw_id", "unprefixed_name", "c_name", "declaration", "physical_page", "section", "source_order")
DEFAULT_FIELDS = frozenset(("family_id", "source_family_order", "derivation_class", "source_locator_format"))


class InventoryError(ValueError):
    """The program/pipeline literal declaration slice is not safely reproducible."""


def reject(message: str) -> None:
    raise InventoryError(message)


def private(item_file: Path, name: str):
    if item_file.is_symlink() or not item_file.is_file():
        reject("fixed private dependency must be a regular file")
    prior = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, item_file)
        if spec is None or spec.loader is None:
            reject("cannot load fixed private dependency")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != item_file.resolve():
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


CATALOG = private(CATALOG_PATH, "f03322331_gles_program_catalog")
ARTIFACT = private(ARTIFACT_PATH, "f03322331_gles_artifact")
DOMAIN_API = private(DOMAIN, "f03322331_gles_domain")
GRAMMAR = private(GRAMMAR_PATH, "f03322331_gles_grammar")
CACHE = private(CACHE_PATH, "f03322331_gles_cache")
LEDGER = private(LEDGER_PATH, "f03322331_gles_ledger")
canonical, exact = ARTIFACT.canonical, ARTIFACT.exact


def artifact(callable, *args):
    try:
        return callable(*args)
    except (ARTIFACT.ArtifactError, ValueError) as error:
        reject(str(error))


def reject_unadmitted(candidate: str) -> None:
    try:
        LEDGER.reject_substitute("shader", candidate)
    except Exception as error:
        reject(str(error))


def source_input(cache_root: Path):
    locator = "gles32-pdf-v1:page=85;section=7.1"
    try:
        authority = CACHE.SOURCE_API.authority().validate()
        decision = CACHE.SOURCE_API.authority().consume("command-state", locator)["decision"]
        receipt, manifest = CACHE.inspect(cache_root, locator, "command-state"), CACHE.manifest()
        source = receipt["source"]
        raw = CACHE.pdf_bytes(CACHE.external_root(cache_root), source)
        domain, grammar, ledger = DOMAIN_API.validate(cache_root), GRAMMAR.validate(cache_root), LEDGER.validate()
        family_order = CATALOG.bound_family(DOMAIN_API.CHUNKS, ARTIFACT.document)
    except Exception as error:
        reject(str(error))
    if (not exact(authority.get("normative_root"), source) or not exact(receipt.get("source"), source)
            or not exact(manifest.get("source"), source) or decision.get("id") != "command-state"
            or receipt.get("physical_pdf_pages") != CATALOG.PAGES or not exact(domain.get("source"), source)
            or domain.get("source_class") != "command-state" or not exact(grammar.get("source"), source)
            or grammar.get("domain_classification_sha256") != domain.get("classification_sha256")
            or not exact(ledger.get("normative_root"), source)
            or ledger.get("ledger_sha256") != domain.get("unavailable_ledger_sha256")):
        reject("authority, cache, domain, grammar, or unavailable-ledger identity is not the exact GLES source root")
    return source, authority, decision, manifest, domain, grammar, ledger, family_order, raw


def records(value: dict[str, object]) -> list[dict[str, object]]:
    if (value.get("raw_entry_fields") != list(ENTRY_FIELDS)
            or set(value.get("raw_entry_defaults", {})) != DEFAULT_FIELDS):
        reject("raw program/pipeline compact-row schema is incomplete or promoted")
    defaults = value["raw_entry_defaults"]
    rows, names = value.get("raw_entries"), set()
    if not isinstance(rows, list):
        reject("raw program/pipeline entries are not a list")
    result = []
    for row in rows:
        if not isinstance(row, list) or len(row) != len(ENTRY_FIELDS):
            reject("raw program/pipeline compact-row shape is malformed")
        item = dict(zip(ENTRY_FIELDS, row))
        if not isinstance(item["unprefixed_name"], str) or item["unprefixed_name"] in names:
            reject("raw program/pipeline name is duplicate or malformed")
        names.add(item["unprefixed_name"])
        item.update(defaults)
        item["source_locator"] = defaults["source_locator_format"].format(**item)
        result.append(item)
    if [item["source_order"] for item in result] != list(range(1, len(result) + 1)):
        reject("raw program/pipeline compact-row order is incomplete or unstable")
    return result


def rendered(cache_root: Path) -> dict[str, object]:
    source, authority, decision, manifest, domain, grammar, ledger, family_order, raw = source_input(cache_root)
    try:
        facts = CATALOG.facts(raw, CATALOG.PAGES, family_order, GRAMMAR.normalize)
    except CATALOG.CatalogError as error:
        reject(str(error))
    defaults = {"family_id": "program-shader", "source_family_order": family_order, "derivation_class": "literal",
                "source_locator_format": "gles32-pdf-v1:page={physical_page};section={section}"}
    body = {"schema": 1, "kind": "webboxvm-gles32-program-pipeline-literal-raw-inventory", "profile": CATALOG.PROFILE,
            "source": source, "source_class": "command-state", "source_decision": decision,
            "source_authority_sha256": authority["boundary_sha256"], "source_contract_sha256": authority["source_contract_sha256"],
            "inventory_lock_sha256": authority["inventory_lock_sha256"], "cache_boundary_sha256": manifest["cache_boundary_sha256"],
            "physical_pdf_pages": CATALOG.PAGES, "domain_classification_sha256": domain["classification_sha256"],
            "grammar_sha256": grammar["grammar_sha256"], "unavailable_ledger_sha256": ledger["ledger_sha256"],
            "domain_families": [{"id": "program-shader", "source_order": family_order}],
            "raw_entry_fields": list(ENTRY_FIELDS), "raw_entry_defaults": defaults, "raw_entries": facts,
            "raw_entry_count": len(facts), "raw_entries_sha256": hashlib.sha256(canonical(facts)).hexdigest(),
            "raw_only": True, "promotion_allowed": False, "scope_fence": "formal-declaration-only"}
    return {**body, "inventory_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate(cache_root: Path, inventory_file: Path = INVENTORY) -> dict[str, object]:
    expected, actual = rendered(cache_root), artifact(ARTIFACT.document, inventory_file)
    artifact(ARTIFACT.self_hashed, actual, "inventory_sha256")
    artifact(ARTIFACT.forbidden, actual)
    if not exact(actual, expected):
        reject("program/pipeline literal inventory is stale, incomplete, rerouted, or promoted")
    records(actual)
    return copy.deepcopy(actual)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--inventory", type=Path, default=INVENTORY)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        value = rendered(args.cache_root) if args.emit_json else validate(args.cache_root, args.inventory)
        print(CATALOG.pretty_json(value) if args.emit_json else "PASS: 29 bounded GLES program/pipeline literals; source-only")
    except InventoryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
