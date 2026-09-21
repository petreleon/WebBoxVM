#!/usr/bin/env python3
"""Validate the bounded, source-only F03.2.2.5.3.1 declaration slice."""

from __future__ import annotations

import argparse
import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE_PATH = HERE.parents[2] / "01-normative-pdf-cache/opengl_normative_pdf_cache.py"
CLASSIFIER_PATH = HERE.parents[1] / "01-command-domain-classification/opengl_command_domain_classification.py"
GRAMMAR_PATH = HERE.parents[1] / "02-template-declaration-grammar/opengl_declaration_grammar.py"
BASELINE_PATH = HERE.parents[2] / "02-command-object-raw-inventory/opengl_command_object_raw_inventory.py"
RULES_PATH, ARTIFACTS_PATH = HERE / "opengl_generic_object_sync_raw_rules.py", HERE / "opengl_generic_object_sync_raw_artifacts.py"
CLAIMS = {key: False for key in ("khronos_selector", "api_support", "conformance", "certification", "profile_support", "performance")}
FORBIDDEN = frozenset(("status", "support", "supported", "implementation_owner", "owner_task", "test", "tests", "cts", "evidence", "conformance", "performance"))


class InventoryError(ValueError):
    """A raw declaration escaped its exact source and no-claim boundary."""


def reject(message: str) -> None:
    raise InventoryError(message)


def private(path: Path, name: str):
    if path.is_symlink() or not path.is_file(): reject("fixed private dependency must be a regular file")
    prior = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None: reject("cannot load fixed private dependency")
        module = importlib.util.module_from_spec(spec); sys.modules[name] = module; spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve(): reject("fixed private dependency resolved from an unexpected path")
        return module
    except InventoryError: raise
    except Exception as error: reject(f"cannot load fixed private dependency: {error}")
    finally:
        if prior is None: sys.modules.pop(name, None)
        else: sys.modules[name] = prior


CACHE = private(CACHE_PATH, "f0322531_cache")
CLASSIFIER = private(CLASSIFIER_PATH, "f0322531_classifier")
GRAMMAR = private(GRAMMAR_PATH, "f0322531_grammar")
BASELINE = private(BASELINE_PATH, "f0322531_baseline")
RULES = private(RULES_PATH, "f0322531_rules")
ARTIFACTS = private(ARTIFACTS_PATH, "f0322531_artifacts")
INVENTORY = HERE / ARTIFACTS.ROOT_NAME


def family(value: dict[str, object], identifier: str) -> dict[str, object]:
    matches = [item for item in value.get("command_families", []) if isinstance(item, dict) and item.get("id") == identifier]
    if len(matches) != 1: reject("classifier has a missing or duplicate required family")
    return matches[0]


def inputs(cache_root: Path) -> dict[str, object]:
    locator = "opengl46-core-pdf-v1:page=58;section=4.1"
    try:
        receipt, manifest = CACHE.inspect(cache_root, locator), CACHE.manifest()
        source = receipt["source"]; raw = CACHE.pdf_bytes(CACHE.external_root(cache_root), source)
        classifier, grammar, baseline = CLASSIFIER.validate(cache_root), GRAMMAR.validate(cache_root), BASELINE.validate(cache_root)
    except Exception as error:
        reject(str(error))
    generic, direct = family(classifier, "event-query-sync"), family(classifier, "direct-creation")
    if (source != manifest.get("source") or source != classifier.get("source") or source != grammar.get("source") or source != baseline.get("source")
            or receipt.get("physical_pdf_pages") != RULES.PAGES or classifier.get("profile") != RULES.PROFILE
            or generic.get("route") != "generic-object-sync" or generic.get("source_scope") != "4.1-4.3"
            or direct.get("route") != "baseline-direct-creation" or direct.get("source_scope") != "2.6.1"
            or grammar.get("rules", {}).get("c_binding_prefix") != {"document_command_prefix": "", "c_command_prefix": "gl", "applies_only_to": "bare-source-anchored-command-names"}
            or len(baseline.get("facts", [])) != 24 or baseline.get("matrix_row_count") != 0):
        reject("cache, classifier, grammar, or direct-creation baseline identities differ")
    return {"source": source, "cache": manifest, "raw": raw, "classifier": classifier, "grammar": grammar, "baseline": baseline,
            "generic": generic, "direct": direct}


def bound_baseline(value: dict[str, object], exclusions: list[dict[str, object]]) -> None:
    by_id = {item.get("fact_id"): item for item in value.get("facts", []) if isinstance(item, dict)}
    ids = [item["fact_id"] for item in exclusions]
    if ids != ["command:glFenceSync", "command:glCreateQueries"] or len(by_id) != 24:
        reject("baseline exclusions are not the exact direct-creation commands")
    for item in exclusions:
        actual = by_id.get(item["fact_id"]); condition = actual.get("condition", {}) if isinstance(actual, dict) else {}
        if (not isinstance(actual, dict) or actual.get("physical_page") != item["physical_page"] or actual.get("source_locator") != item["source_locator"]
                or condition.get("declaration") != item["declaration"]):
            reject("baseline exclusion does not bind its exact direct-creation fact")


def rendered(cache_root: Path) -> dict[str, dict[str, object]]:
    data = inputs(cache_root)
    try:
        declarations, exclusions, deferred = RULES.rows(data["raw"], GRAMMAR.RULES)
    except RULES.RuleError as error:
        reject(str(error))
    bound_baseline(data["baseline"], exclusions)
    core = {"schema": 1, "kind": "webboxvm-opengl46-generic-object-sync-raw-inventory", "profile": RULES.PROFILE,
            "source_class": "command-object-state", "source": data["source"], "cache_boundary_sha256": data["cache"]["cache_boundary_sha256"],
            "cache_layout": data["cache"]["cache_layout"], "physical_pdf_pages": RULES.PAGES,
            "classifier_binding": {"classification_sha256": data["classifier"]["classification_sha256"], "family_id": data["generic"]["id"],
                                    "route": data["generic"]["route"], "source_scope": data["generic"]["source_scope"],
                                    "direct_creation_family_id": data["direct"]["id"], "direct_creation_source_scope": data["direct"]["source_scope"]},
            "grammar_binding": {"normalization_sha256": data["grammar"]["normalization_sha256"], "c_command_prefix": "gl"},
            "baseline_binding": {"inventory_sha256": data["baseline"]["inventory_sha256"], "facts_sha256": data["baseline"]["facts_sha256"],
                                 "excluded_command_fact_ids": [item["fact_id"] for item in exclusions]},
            "source_windows": [{"numeric_section": section, "physical_pages": list(pages), "formal_declaration_count": count} for section, pages, count, _ in RULES.WINDOWS],
            "formal_source_declaration_count": 25, "baseline_exclusions": exclusions,
            "returned_state_semantics_deferred": {"target": "F03.2.2.3", "declaration_ids": deferred, "semantic_fact_count": 0},
            "raw_only": True, "semantic_fact_count": 0, "matrix_row_count": 0, "cts_executions": 0, "claims": CLAIMS, "states": CACHE.STATES}
    try: return ARTIFACTS.bundle(core, declarations)
    except ValueError as error: reject(str(error))


def fences(value: dict[str, object]) -> None:
    rows, exclusions = value.get("declarations"), value.get("baseline_exclusions")
    if not isinstance(rows, list) or not isinstance(exclusions, list) or len(rows) != 23 or len(exclusions) != 2:
        reject("raw declaration or baseline exclusion counts changed")
    required = {"fact_id", "document_name", "declaration", "physical_page", "numeric_section", "source_locator", "source_sequence_position", "c_name", "fragment", "source_order"}
    if (any(not isinstance(row, dict) or set(row) != required or FORBIDDEN & set(row) for row in rows)
            or [row["source_order"] for row in rows] != list(range(1, 24)) or len({row["fact_id"] for row in rows}) != 23):
        reject("raw declaration facts are duplicate, reordered, incomplete, or promoted")
    if any(row["c_name"] != "gl" + row["document_name"] or row["source_locator"] != f"opengl46-core-pdf-v1:page={row['physical_page']};section={row['numeric_section']}" for row in rows):
        reject("raw declaration has a cross-profile or malformed source locator")
    positions = [row["source_sequence_position"] for row in rows] + [row["source_sequence_position"] for row in exclusions]
    deferred = value.get("returned_state_semantics_deferred", {})
    if (sorted(positions) != list(range(1, 26)) or deferred.get("target") != "F03.2.2.3"
            or set(deferred.get("declaration_ids", [])) - {row["fact_id"] for row in rows}
            or len(deferred.get("declaration_ids", [])) != 13 or len(set(deferred.get("declaration_ids", []))) != 13
            or [item.get("fact_id") for item in exclusions] != ["command:glFenceSync", "command:glCreateQueries"]
            or value.get("claims") != CLAIMS or value.get("cts_executions") != 0 or value.get("matrix_row_count") != 0
            or value.get("raw_only") is not True or value.get("semantic_fact_count") != 0):
        reject("raw slice loses source coverage, state deferral, or no-claim bounds")


def validate(cache_root: Path, inventory_path: Path = INVENTORY) -> dict[str, object]:
    value = ARTIFACTS.validate(inventory_path, rendered(cache_root), reject); fences(value); return value


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--inventory", type=Path, default=INVENTORY); parser.add_argument("--emit-json", action="store_true")
    parser.add_argument("--emit-artifact", choices=(ARTIFACTS.ROOT_NAME,) + tuple(name for _, name in ARTIFACTS.FRAGMENTS)); args = parser.parse_args()
    try:
        bundle = rendered(args.cache_root)
        if args.emit_json or args.emit_artifact: print(ARTIFACTS.serialized(bundle[args.emit_artifact or ARTIFACTS.ROOT_NAME]), end="")
        else: print(f"PASS: {len(validate(args.cache_root, args.inventory)['declarations'])} generic object/sync raw declarations; source-only")
    except InventoryError as error:
        print(f"FAIL: {error}", file=sys.stderr); raise SystemExit(2)


if __name__ == "__main__": main()
