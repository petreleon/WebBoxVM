#!/usr/bin/env python3
"""Validate F03.2.2.5.4.2's bounded source-only submission declarations."""

from __future__ import annotations

import argparse
import copy
import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE_PATH = HERE.parents[2] / "01-normative-pdf-cache/opengl_normative_pdf_cache.py"
CLASSIFIER_PATH = HERE.parents[1] / "01-command-domain-classification/opengl_command_domain_classification.py"
GRAMMAR_PATH = HERE.parents[1] / "02-template-declaration-grammar/opengl_declaration_grammar.py"
RULES_PATH, ARTIFACTS_PATH = HERE / "opengl_draw_compute_submission_rules.py", HERE / "opengl_draw_compute_submission_artifacts.py"
CLAIMS = {key: False for key in ("khronos_selector", "api_support", "conformance", "certification", "profile_support", "performance")}
FORBIDDEN = frozenset(("requirement_kind", "status", "implementation_owner", "independent_test_plan", "owner_task", "test_source_role", "evidence", "coverage", "semantics", "behavior", "support", "supported"))


class InventoryError(ValueError):
    """A declaration escaped its source, route, or no-claim boundary."""


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


CACHE = private(CACHE_PATH, "f0322542_cache")
CLASSIFIER = private(CLASSIFIER_PATH, "f0322542_classifier")
GRAMMAR = private(GRAMMAR_PATH, "f0322542_grammar")
RULES = private(RULES_PATH, "f0322542_rules")
ARTIFACTS = private(ARTIFACTS_PATH, "f0322542_artifacts")
INVENTORY = HERE / ARTIFACTS.ROOT_NAME
TOP_LEVEL = frozenset(("schema", "kind", "profile", "source_class", "source", "authority_boundary_sha256", "source_contract_sha256", "inventory_lock_sha256", "cache_boundary_sha256", "cache_layout", "physical_pdf_pages", "serialized_size_limit", "classification_binding", "declaration_grammar_binding", "source_windows", "formal_source_declaration_count", "raw_submission_declaration_count", "formal_source_exclusions", "raw_only", "semantic_fact_count", "rejections", "claims", "cts_executions", "matrix_row_count", "states", "artifact_receipts", "raw_inventory_sha256", "declarations"))


def classifier_binding(value: dict[str, object], source: dict[str, object]) -> dict[str, object]:
    expected = (("draw-submission", "10.4", 13, 388, "10.4", "void DrawArrays( enum mode, int first, sizei count );"),
                ("conditional-rendering", "10.9", 14, 403, "10.9", "void BeginConditionalRender( uint id, enum mode );"),
                ("transform-feedback-draw", "13.3.3", 19, 472, "13.3.3", "void DrawTransformFeedback( enum mode, uint id );"),
                ("compute-submission", "19", 25, 568, "19", "void DispatchCompute( uint num groups x, uint num groups y, uint num groups z );"))
    rows = value.get("command_families") if isinstance(value, dict) else None
    selected = [row for row in rows if isinstance(row, dict) and row.get("route") == "draw-compute-submission"] if isinstance(rows, list) else []
    compact = [(row.get("id"), row.get("source_scope"), row.get("source_order"), row["anchor"].get("physical_page"), row["anchor"].get("section"), row["anchor"].get("declaration")) for row in selected if isinstance(row.get("anchor"), dict)]
    if value.get("source") != source or value.get("source_class") != "command-object-state" or compact != list(expected) or not isinstance(value.get("classification_sha256"), str):
        reject("F03.2.2.5.1 does not bind exact included and excluded submission families")
    return {"classification_sha256": value["classification_sha256"], "route": "draw-compute-submission", "included_family_ids": [item[0] for item in (expected[0], expected[2], expected[3])], "out_of_scope_same_route_family_ids": [expected[1][0]], "included_source_orders": [item[2] for item in (expected[0], expected[2], expected[3])]}


def grammar_binding(value: dict[str, object], source: dict[str, object]) -> dict[str, object]:
    rules = value.get("rules") if isinstance(value, dict) else None
    prefix = rules.get("c_binding_prefix") if isinstance(rules, dict) else None
    expected = {"document_command_prefix": "", "c_command_prefix": RULES.PREFIX, "applies_only_to": "bare-source-anchored-command-names"}
    if value.get("source") != source or prefix != expected or not isinstance(value.get("normalization_sha256"), str):
        reject("F03.2.2.5.2 does not bind sealed OpenGL C-name normalization")
    return {"normalization_sha256": value["normalization_sha256"], "document_command_prefix": "", "c_command_prefix": RULES.PREFIX}


def source_input(cache_root: Path) -> tuple[dict[str, object], dict[str, object], dict[str, object], bytes, dict[str, object], dict[str, object]]:
    locator = "opengl46-core-pdf-v1:page=388;section=10.4"
    try:
        authority = CACHE.SOURCE_API.authority()
        boundary, consumed = authority.validate(), authority.consume("command-object-state", locator)
        receipt, manifest = CACHE.inspect(cache_root, locator), CACHE.manifest()
        source = receipt["source"]
        raw = CACHE.pdf_bytes(CACHE.external_root(cache_root), source)
        classification, grammar = CLASSIFIER.validate(cache_root), GRAMMAR.validate(cache_root)
    except Exception as error:
        reject(str(error))
    if (not isinstance(boundary, dict) or not isinstance(consumed, dict) or not isinstance(source, dict) or boundary.get("normative_root") != source or consumed.get("source") != source or consumed.get("decision", {}).get("id") != "command-object-state" or manifest.get("source") != source or receipt.get("physical_pdf_pages") != RULES.PAGES):
        reject("F03.2.1 and F03.2.2.1 do not expose the exact admitted PDF")
    return source, manifest, boundary, raw, classification, grammar


def rendered(cache_root: Path) -> dict[str, dict[str, object]]:
    source, manifest, boundary, raw, classification, grammar = source_input(cache_root)
    try:
        declarations, exclusions = RULES.rows(raw, GRAMMAR.RULES)
    except RULES.RuleError as error:
        reject(str(error))
    core = {"schema": 1, "kind": "webboxvm-opengl46-draw-compute-submission-inventory", "profile": RULES.PROFILE, "source_class": "command-object-state", "source": source, "authority_boundary_sha256": boundary["boundary_sha256"], "source_contract_sha256": boundary["source_contract_sha256"], "inventory_lock_sha256": boundary["inventory_lock_sha256"], "cache_boundary_sha256": manifest["cache_boundary_sha256"], "cache_layout": manifest["cache_layout"], "physical_pdf_pages": RULES.PAGES, "serialized_size_limit": ARTIFACTS.MAX_SERIALIZED, "classification_binding": classifier_binding(classification, source), "declaration_grammar_binding": grammar_binding(grammar, source), "source_windows": [{"numeric_section": section, "physical_pages": list(pages), "formal_declaration_count": formal, "raw_submission_declaration_count": actual} for section, pages, formal, actual, _ in RULES.WINDOWS], "formal_source_declaration_count": 28, "raw_submission_declaration_count": 26, "raw_only": True, "semantic_fact_count": 0, "rejections": list(RULES.REJECTIONS), "claims": CLAIMS, "cts_executions": 0, "matrix_row_count": 0, "states": CACHE.STATES}
    try:
        return ARTIFACTS.bundle(core, declarations, exclusions)
    except ValueError as error:
        reject(str(error))


def fences(value: dict[str, object]) -> None:
    rows, exclusions = value.get("declarations"), value.get("formal_source_exclusions")
    row_shape = {"fact_id", "document_name", "c_name", "declaration", "physical_page", "numeric_section", "source_locator", "source_sequence_position", "source_order", "fragment"}
    exclusion_shape = {"document_name", "declaration", "physical_page", "numeric_section", "source_locator", "source_sequence_position", "exclusion"}
    if set(value) != TOP_LEVEL or not isinstance(rows, list) or not isinstance(exclusions, list) or FORBIDDEN & set(value) or any(not isinstance(row, dict) or set(row) != row_shape or FORBIDDEN & set(row) for row in rows) or any(not isinstance(row, dict) or set(row) != exclusion_shape or FORBIDDEN & set(row) for row in exclusions):
        reject("raw inventory has a forbidden promotion or unclosed shape")
    if ([row["source_order"] for row in rows] != list(range(1, 27)) or [row["source_sequence_position"] for row in rows] != list(range(2, 9)) + list(range(10, 29)) or [row["document_name"] for row in exclusions] != ["DrawArraysOneInstance", "DrawElementsOneInstance"] or [row["source_sequence_position"] for row in exclusions] != [1, 9] or any(row["c_name"] != "gl" + row["document_name"] or row["source_locator"] != f"opengl46-core-pdf-v1:page={row['physical_page']};section={row['numeric_section']}" for row in rows)):
        reject("raw declaration order, source closure, or pseudo-command exclusions changed")
    if (len(rows) != 26 or len({row["fact_id"] for row in rows}) != 26 or value.get("formal_source_declaration_count") != 28 or value.get("raw_submission_declaration_count") != 26 or value.get("raw_only") is not True or value.get("semantic_fact_count") != 0 or value.get("rejections") != list(RULES.REJECTIONS) or value.get("claims") != CLAIMS or value.get("cts_executions") != 0 or value.get("matrix_row_count") != 0):
        reject("raw inventory loses exact counts or no-claim bounds")


def validate(cache_root: Path, inventory_path: Path = INVENTORY) -> dict[str, object]:
    value = ARTIFACTS.validate(inventory_path, rendered(cache_root), reject)
    fences(value)
    return copy.deepcopy(value)


def reject_matrix_row(row: object, cache_root: Path, inventory_path: Path = INVENTORY) -> None:
    validate(cache_root, inventory_path)
    reject("source declaration inventory cannot create a Matrix v2 row")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--inventory", type=Path, default=INVENTORY)
    parser.add_argument("--emit-artifact", choices=(ARTIFACTS.ROOT_NAME,) + tuple(name for _, name in ARTIFACTS.FRAGMENTS))
    args = parser.parse_args()
    try:
        bundle = rendered(args.cache_root)
        if args.emit_artifact:
            print(ARTIFACTS.serialized(bundle[args.emit_artifact]), end="")
        else:
            print(f"PASS: {len(validate(args.cache_root, args.inventory)['declarations'])} source-only draw/compute submission declarations; matrix-incomplete")
    except InventoryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
