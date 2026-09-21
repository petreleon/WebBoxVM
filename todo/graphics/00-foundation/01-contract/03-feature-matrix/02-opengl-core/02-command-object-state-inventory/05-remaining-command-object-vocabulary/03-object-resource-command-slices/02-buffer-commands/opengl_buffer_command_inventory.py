#!/usr/bin/env python3
"""Validate the bounded, source-only F03.2.2.5.3.2 buffer declaration slice."""

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
STATE_PATH = HERE.parents[2] / "03-state-and-lifecycle-raw-inventory/01-buffer-binding-lifecycle-raw-slice/opengl_state_raw_inventory.py"
RULES_PATH, ARTIFACTS_PATH = HERE / "opengl_buffer_command_rules.py", HERE / "opengl_buffer_command_artifacts.py"
CLAIMS = {key: False for key in ("khronos_selector", "api_support", "conformance", "certification", "profile_support", "performance")}
FORBIDDEN = frozenset(("status", "support", "supported", "implementation_owner", "owner_task", "test", "tests", "cts", "evidence", "conformance", "performance", "behavior"))
RAW_KEYS = frozenset(("fact_id", "document_name", "declaration", "physical_page", "source_page_span", "numeric_section", "source_locator", "source_sequence_position", "c_name", "fragment", "source_order"))


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


CACHE = private(CACHE_PATH, "f0322532_cache")
CLASSIFIER = private(CLASSIFIER_PATH, "f0322532_classifier")
GRAMMAR = private(GRAMMAR_PATH, "f0322532_grammar")
BASELINE = private(BASELINE_PATH, "f0322532_baseline")
STATE = private(STATE_PATH, "f0322532_state")
RULES = private(RULES_PATH, "f0322532_rules")
ARTIFACTS = private(ARTIFACTS_PATH, "f0322532_artifacts")
INVENTORY = HERE / ARTIFACTS.ROOT_NAME


def inputs(cache_root: Path):
    locator = "opengl46-core-pdf-v1:page=82;section=6.1"
    try:
        authority = CACHE.SOURCE_API.authority(); boundary, consumed = authority.validate(), authority.consume("command-object-state", locator)
        receipt, manifest = CACHE.inspect(cache_root, locator), CACHE.manifest(); source = receipt["source"]
        raw = CACHE.pdf_bytes(CACHE.external_root(cache_root), source)
        classifier, grammar, baseline, state = CLASSIFIER.validate(cache_root), GRAMMAR.validate(cache_root), BASELINE.validate(cache_root), STATE.validate(cache_root)
    except Exception as error:
        reject(str(error))
    if (boundary.get("normative_root") != source or consumed.get("source") != source or consumed.get("decision", {}).get("id") != "command-object-state"
            or manifest.get("source") != source or receipt.get("physical_pdf_pages") != RULES.PAGES or classifier.get("source") != source
            or grammar.get("source") != source or baseline.get("source") != source or state.get("source") != source
            or any(value.get("profile") != RULES.PROFILE or value.get("source_class") != "command-object-state" for value in (classifier, grammar, baseline, state))):
        reject("authority, cache, classifier, grammar, baseline, or state source differs")
    return source, manifest, boundary, raw, classifier, grammar, baseline, state


def classifier_binding(value: dict[str, object]) -> dict[str, object]:
    expected = {"id": "buffer", "route": "buffer-commands", "source_scope": "6.1-6.7", "source_order": 5,
                "anchor": {"kind": "formal-declaration", "physical_page": 82, "section": "6.1", "source_locator": "opengl46-core-pdf-v1:page=82;section=6.1", "declaration": "void DeleteBuffers( sizei n, const uint *buffers );"},
                "excluded_baseline_command_ids": ["command:glCreateBuffers"],
                "excluded_baseline_command_anchors": [{"fact_id": "command:glCreateBuffers", "physical_page": 81, "section": "6.1", "source_locator": "opengl46-core-pdf-v1:page=81;section=6.1"}]}
    rows = [item for item in value.get("command_families", []) if isinstance(item, dict) and item.get("id") == "buffer"]
    if rows != [expected] or not isinstance(value.get("classification_sha256"), str): reject("classifier does not bind the exact buffer command family")
    return {"classification_sha256": value["classification_sha256"], "family_id": "buffer", "route": "buffer-commands", "source_scope": "6.1-6.7"}


def grammar_binding(value: dict[str, object]) -> dict[str, object]:
    expected = {"document_command_prefix": "", "c_command_prefix": "gl", "applies_only_to": "bare-source-anchored-command-names"}
    if value.get("rules", {}).get("c_binding_prefix") != expected or not isinstance(value.get("normalization_sha256"), str):
        reject("grammar does not bind the sealed bare OpenGL C-name prefix")
    return {"normalization_sha256": value["normalization_sha256"], "c_command_prefix": "gl"}


def baseline_binding(value: dict[str, object]) -> dict[str, object]:
    rows = [item for item in value.get("facts", []) if isinstance(item, dict) and item.get("fact_id") == "command:glCreateBuffers"]
    if len(rows) != 1 or len(value.get("facts", [])) != 24 or value.get("matrix_row_count") != 0: reject("direct-creation baseline differs")
    row, condition = rows[0], rows[0].get("condition", {})
    if (row.get("physical_page"), row.get("source_locator"), condition.get("declaration")) != (81, "opengl46-core-pdf-v1:page=81;section=6.1", "void CreateBuffers( sizei n, uint *buffers );"):
        reject("baseline exclusion is not the exact CreateBuffers command")
    return {"inventory_sha256": value["inventory_sha256"], "facts_sha256": value["facts_sha256"], "excluded_command_fact_ids": ["command:glCreateBuffers"]}


def state_binding(value: dict[str, object]) -> dict[str, object]:
    ids = [item.get("id") for item in value.get("facts", []) if isinstance(item, dict)]
    if (ids != ["buffer-existing-rebind", "buffer-delete-current-context", "array-buffer-binding-state"] or value.get("complete") is not False
            or value.get("matrix_rows") != 0 or value.get("cts_executions") != 0 or value.get("claims") != CLAIMS):
        reject("existing buffer state artifact differs from its raw no-claim boundary")
    return {"inventory_sha256": value["inventory_sha256"], "facts_sha256": value["facts_sha256"], "fact_ids": ids}


def bound_locators(source: dict[str, object], declarations, exclusions) -> None:
    try: consumed = [CACHE.SOURCE_API.authority().consume("command-object-state", row["source_locator"]) for row in declarations + exclusions]
    except Exception as error: reject(f"raw declaration locator is outside the admitted source authority: {error}")
    if any(item.get("source") != source or item.get("decision", {}).get("id") != "command-object-state" for item in consumed):
        reject("raw declaration locator is not bound to the admitted OpenGL core source")


def rendered(cache_root: Path) -> dict[str, dict[str, object]]:
    source, manifest, boundary, raw, classifier, grammar, baseline, state = inputs(cache_root)
    try: declarations, exclusions, windows = RULES.rows(raw, GRAMMAR.RULES)
    except RULES.RuleError as error: reject(str(error))
    bound_locators(source, declarations, exclusions)
    core = {"schema": 1, "kind": "webboxvm-opengl46-buffer-command-raw-inventory", "profile": RULES.PROFILE,
            "source_class": "command-object-state", "source": source, "authority_boundary_sha256": boundary["boundary_sha256"],
            "source_contract_sha256": boundary["source_contract_sha256"], "inventory_lock_sha256": boundary["inventory_lock_sha256"],
            "cache_boundary_sha256": manifest["cache_boundary_sha256"], "cache_layout": manifest["cache_layout"], "physical_pdf_pages": RULES.PAGES,
            "classifier_binding": classifier_binding(classifier), "grammar_binding": grammar_binding(grammar), "baseline_binding": baseline_binding(baseline),
            "buffer_state_binding": state_binding(state), "source_windows": windows, "formal_source_declaration_count": 39, "baseline_exclusions": exclusions,
            "deferred_non_declaration_routes": {"buffer_binding_and_deletion_transitions": "F03.2.2.3", "numeric_properties_and_formats": "F03.2.3"},
            "rejections": ["baseline-direct-creation-duplicate", "unanchored-template-or-ambiguous-declaration", "extension-compatibility-registry-lower-profile-source", "state-lifecycle-and-numeric-properties-deferred"],
            "raw_only": True, "semantic_fact_count": 0, "matrix_row_count": 0, "cts_executions": 0, "claims": CLAIMS, "states": CACHE.STATES}
    try: return ARTIFACTS.bundle(core, declarations)
    except ValueError as error: reject(str(error))


def fences(value: dict[str, object]) -> None:
    rows, exclusions = value.get("declarations"), value.get("baseline_exclusions")
    if (not isinstance(rows, list) or not isinstance(exclusions, list) or len(rows) != 38 or len(exclusions) != 1
            or any(not isinstance(row, dict) or set(row) != RAW_KEYS or FORBIDDEN & set(row) for row in rows)):
        reject("raw declaration fields are incomplete, promoted, or outside the closed slice")
    positions = [row["source_sequence_position"] for row in rows] + [item.get("source_sequence_position") for item in exclusions]
    if ([row["source_order"] for row in rows] != list(range(1, 39)) or sorted(positions) != list(range(1, 40))
            or len({row["fact_id"] for row in rows}) != 38 or exclusions[0].get("fact_id") != "command:glCreateBuffers"):
        reject("raw declaration ordering, identities, or baseline exclusion changed")
    if any(row["c_name"] != "gl" + row["document_name"] or row["source_locator"] != RULES.locator(tuple(row["source_page_span"]), row["numeric_section"]) for row in rows):
        reject("raw declaration has a malformed C name, page span, or source locator")
    if (value.get("formal_source_declaration_count") != 39 or value.get("deferred_non_declaration_routes") != {"buffer_binding_and_deletion_transitions": "F03.2.2.3", "numeric_properties_and_formats": "F03.2.3"}
            or value.get("rejections") != ["baseline-direct-creation-duplicate", "unanchored-template-or-ambiguous-declaration", "extension-compatibility-registry-lower-profile-source", "state-lifecycle-and-numeric-properties-deferred"]
            or value.get("raw_only") is not True or value.get("semantic_fact_count") != 0 or value.get("claims") != CLAIMS
            or value.get("matrix_row_count") != 0 or value.get("cts_executions") != 0):
        reject("raw slice loses source coverage, route deferrals, or no-claim bounds")


def validate(cache_root: Path, inventory_path: Path = INVENTORY) -> dict[str, object]:
    value = ARTIFACTS.validate(inventory_path, rendered(cache_root), reject); fences(value); return value


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--inventory", type=Path, default=INVENTORY); parser.add_argument("--emit-json", action="store_true")
    parser.add_argument("--emit-artifact", choices=(ARTIFACTS.ROOT_NAME,) + tuple(name for _, name in ARTIFACTS.FRAGMENTS)); args = parser.parse_args()
    try:
        bundle = rendered(args.cache_root)
        if args.emit_json or args.emit_artifact: print(ARTIFACTS.serialized(bundle[args.emit_artifact or ARTIFACTS.ROOT_NAME]), end="")
        else: print(f"PASS: {len(validate(args.cache_root, args.inventory)['declarations'])} buffer command raw declarations; source-only")
    except InventoryError as error:
        print(f"FAIL: {error}", file=sys.stderr); raise SystemExit(2)


if __name__ == "__main__": main()
