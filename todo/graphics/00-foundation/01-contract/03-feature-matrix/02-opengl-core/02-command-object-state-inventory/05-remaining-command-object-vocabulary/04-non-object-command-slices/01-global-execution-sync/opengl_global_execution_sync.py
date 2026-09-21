#!/usr/bin/env python3
"""Validate F03.2.2.5.4.1's source-only global declaration inventory."""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE_PATH = HERE.parents[2] / "01-normative-pdf-cache/opengl_normative_pdf_cache.py"
CLASSIFIER_PATH = HERE.parents[1] / "01-command-domain-classification/opengl_command_domain_classification.py"
GRAMMAR_PATH = HERE.parents[1] / "02-template-declaration-grammar/opengl_declaration_grammar.py"
RULES_PATH, ARTIFACT, MAX_SERIALIZED = HERE / "opengl_global_execution_sync_rules.py", HERE / "opengl_global_execution_sync.json", 8 * 1024 * 1024
ARTIFACTS_PATH = HERE / "opengl_global_execution_sync_artifacts.py"
CLAIMS = {key: False for key in ("khronos_selector", "api_support", "conformance", "certification", "profile_support", "performance")}
FORBIDDEN = frozenset((
    "requirement_kind", "status", "implementation_owner", "independent_test_plan", "owner_task",
    "test_source_role", "evidence", "coverage", "semantics", "behavior",
))
TOP_LEVEL = frozenset((
    "schema", "kind", "profile", "source_class", "source", "authority_boundary_sha256",
    "source_contract_sha256", "inventory_lock_sha256", "cache_boundary_sha256", "cache_layout",
    "physical_pdf_pages", "serialized_size_limit", "classification_binding", "declaration_grammar_binding",
    "source_pages", "declarations", "rejections", "claims", "cts_executions", "matrix_row_count",
    "states", "inventory_sha256",
))


class InventoryError(ValueError):
    """The inventory is stale, promoted, unbounded, or not source-only."""


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


CACHE = private(CACHE_PATH, "f0322541_pdf_cache")
CLASSIFIER = private(CLASSIFIER_PATH, "f0322541_command_domain")
GRAMMAR = private(GRAMMAR_PATH, "f0322541_declaration_grammar")
RULES = private(RULES_PATH, "f0322541_global_execution_rules")
ARTIFACTS = private(ARTIFACTS_PATH, "f0322541_global_execution_artifacts")
canonical = ARTIFACTS.canonical


def classifier_binding(value: dict[str, object], source: dict[str, object]) -> dict[str, object]:
    expected = (("global-execution", "2.3.1-2.3.3", 2, 42, "2.3.3", "void Flush( void );"),
                ("shader-memory-sync", "7.13.2", 7, 183, "7.13.2", "void MemoryBarrier( bitfield barriers );"))
    rows = value.get("command_families") if isinstance(value, dict) else None
    selected = [row for row in rows if isinstance(row, dict) and row.get("route") == "global-execution-sync"] if isinstance(rows, list) else []
    compact = [
        (row.get("id"), row.get("source_scope"), row.get("source_order"), row["anchor"].get("physical_page"),
         row["anchor"].get("section"), row["anchor"].get("declaration"))
        for row in selected if isinstance(row.get("anchor"), dict)
    ]
    if (value.get("source") != source or value.get("source_class") != "command-object-state" or compact != list(expected)
            or not isinstance(value.get("classification_sha256"), str)):
        reject("F03.2.2.5.1 does not bind the exact two global source families")
    return {"classification_sha256": value["classification_sha256"], "family_ids": [item[0] for item in expected],
            "route": "global-execution-sync", "source_orders": [item[2] for item in expected]}


def grammar_binding(value: dict[str, object], source: dict[str, object]) -> dict[str, object]:
    rules = value.get("rules") if isinstance(value, dict) else None
    prefix = rules.get("c_binding_prefix") if isinstance(rules, dict) else None
    expected = {"document_command_prefix": "", "c_command_prefix": RULES.PREFIX,
                "applies_only_to": "bare-source-anchored-command-names"}
    if value.get("source") != source or prefix != expected or not isinstance(value.get("normalization_sha256"), str):
        reject("F03.2.2.5.2 does not bind the sealed OpenGL C-name normalization")
    return {"normalization_sha256": value["normalization_sha256"], "document_command_prefix": "", "c_command_prefix": RULES.PREFIX}


def source_input(cache_root: Path) -> tuple[dict[str, object], dict[str, object], dict[str, object],
                                            bytes, dict[str, object], dict[str, object]]:
    locator = "opengl46-core-pdf-v1:page=38;section=2.3.1"
    try:
        authority = CACHE.SOURCE_API.authority()
        boundary, consumed = authority.validate(), authority.consume("command-object-state", locator)
        receipt, manifest = CACHE.inspect(cache_root, locator), CACHE.manifest()
        source = receipt["source"]
        raw = CACHE.pdf_bytes(CACHE.external_root(cache_root), source)
        classification, grammar = CLASSIFIER.validate(cache_root), GRAMMAR.validate(cache_root)
    except Exception as error:
        reject(str(error))
    if (not isinstance(boundary, dict) or not isinstance(consumed, dict) or not isinstance(source, dict)
            or boundary.get("normative_root") != source or consumed.get("source") != source
            or consumed.get("decision", {}).get("id") != "command-object-state" or manifest.get("source") != source
            or receipt.get("physical_pdf_pages") != RULES.PAGES):
        reject("F03.2.1 and F03.2.2.1 do not expose the exact admitted PDF")
    return source, manifest, boundary, raw, classification, grammar


def rendered(cache_root: Path) -> dict[str, object]:
    source, manifest, boundary, raw, classification, grammar = source_input(cache_root)
    try:
        records = RULES.inventory(raw)
    except RULES.RuleError as error:
        reject(str(error))
    body = {"schema": 1, "kind": "webboxvm-opengl46-global-execution-sync-declaration-inventory", "profile": RULES.PROFILE,
            "source_class": "command-object-state", "source": source, "authority_boundary_sha256": boundary["boundary_sha256"],
            "source_contract_sha256": boundary["source_contract_sha256"], "inventory_lock_sha256": boundary["inventory_lock_sha256"],
            "cache_boundary_sha256": manifest["cache_boundary_sha256"], "cache_layout": manifest["cache_layout"],
            "physical_pdf_pages": RULES.PAGES, "serialized_size_limit": MAX_SERIALIZED,
            "classification_binding": classifier_binding(classification, source), "declaration_grammar_binding": grammar_binding(grammar, source),
            **records, "claims": CLAIMS, "cts_executions": 0, "matrix_row_count": 0, "states": CACHE.STATES}
    return {**body, "inventory_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate(cache_root: Path, artifact_path: Path = ARTIFACT) -> dict[str, object]:
    value = ARTIFACTS.document(artifact_path, MAX_SERIALIZED, reject)
    body = {key: item for key, item in value.items() if key != "inventory_sha256"}
    if value.get("inventory_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        reject("inventory artifact has a stale self hash")
    ARTIFACTS.fences(value, TOP_LEVEL, FORBIDDEN, RULES, CLAIMS, reject)
    if value != rendered(cache_root):
        reject("inventory artifact is stale, mixed, partial, reordered, or incorrectly anchored")
    return copy.deepcopy(value)


def reject_matrix_row(row: object, cache_root: Path, artifact_path: Path = ARTIFACT) -> None:
    validate(cache_root, artifact_path)
    reject("declaration inventory cannot create a Matrix v2 row")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--artifact", type=Path, default=ARTIFACT)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        value = rendered(args.cache_root) if args.emit_json else validate(args.cache_root, args.artifact)
        if args.emit_json:
            print(json.dumps(value, sort_keys=True, indent=2))
        else:
            print("PASS: 6 source-only global execution/synchronization declarations; matrix-incomplete")
    except InventoryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
