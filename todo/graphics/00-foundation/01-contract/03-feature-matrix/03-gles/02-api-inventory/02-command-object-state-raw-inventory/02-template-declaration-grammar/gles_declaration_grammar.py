#!/usr/bin/env python3
"""Validate the finite, source-only F03.3.2.2.2 declaration grammar."""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
RAW = HERE.parent
DOMAIN = RAW / "01-command-domain-classification/gles_command_domain_classification.py"
ARTIFACT_PATH = RAW / "01-command-domain-classification/gles_command_domain_artifact.py"
CACHE_PATH = RAW.parent / "01-normative-pdf-cache/gles_normative_pdf_cache.py"
RULES_PATH, CATALOG = HERE / "gles_declaration_grammar_rules.py", HERE / "gles_declaration_grammar.json"


class GrammarValidationError(ValueError):
    """The grammar is stale, incorrectly sourced, or promotes source metadata."""


def reject(message: str) -> None:
    raise GrammarValidationError(message)


def private(path: Path, name: str):
    if path.is_symlink() or not path.is_file():
        reject("fixed private dependency must be a regular file")
    before = sys.modules.get(name)
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
    except GrammarValidationError:
        raise
    except Exception as error:
        reject(f"cannot load fixed private dependency: {error}")
    finally:
        if before is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = before


RULES = private(RULES_PATH, "f033222_gles_grammar_rules")
ARTIFACT = private(ARTIFACT_PATH, "f033222_gles_domain_artifact")
CACHE = private(CACHE_PATH, "f033222_gles_pdf_cache")
DOMAIN_API = private(DOMAIN, "f033222_gles_domain")
canonical, exact = ARTIFACT.canonical, ARTIFACT.exact


def artifact(callable, *args):
    try:
        return callable(*args)
    except ARTIFACT.ArtifactError as error:
        reject(str(error))


def source_input(cache_root: Path):
    locator = "gles32-pdf-v1:page=27;section=2.2"
    try:
        authority = CACHE.SOURCE_API.authority().validate()
        decision = CACHE.SOURCE_API.authority().consume("command-state", locator)["decision"]
        receipt, manifest = CACHE.inspect(cache_root, locator, "command-state"), CACHE.manifest()
        source = receipt["source"]
        raw = CACHE.pdf_bytes(CACHE.external_root(cache_root), source)
        domain = DOMAIN_API.validate(cache_root)
    except Exception as error:
        reject(str(error))
    if (not exact(authority.get("normative_root"), source) or not exact(receipt.get("source"), source)
            or not exact(manifest.get("source"), source) or decision.get("id") != "command-state"
            or receipt.get("physical_pdf_pages") != RULES.PAGES or not exact(domain.get("source"), source)
            or domain.get("source_class") != "command-state"):
        reject("authority, cache, or command-domain identity is not the exact GLES root")
    return source, authority, decision, manifest, domain, raw


def rendered(cache_root: Path) -> dict[str, object]:
    source, authority, decision, manifest, domain, raw = source_input(cache_root)
    try:
        anchors, templates = RULES.check_anchors(raw, RULES.PAGES)
    except RULES.GrammarError as error:
        reject(str(error))
    grammar = {"c_binding": {"document_command_prefix": "", "c_command_prefix": RULES.PREFIX,
                              "input": "unprefixed-command-name"},
               "literal_pattern": "[A-Z][A-Za-z0-9]*",
               "template_choice_groups": [{"notation": notation, "choices": list(choices)} for notation, choices in RULES.GROUPS],
               "formal_templates": templates,
               "normalization": {"preserves_input_order": True, "disallows_duplicate_outputs": True,
                                 "blocks_unexpanded_or_guessed_forms": True}}
    body = {"schema": 1, "kind": "webboxvm-gles32-declaration-grammar", "profile": RULES.PROFILE,
            "source": source, "source_class": "command-state", "source_decision": decision,
            "source_authority_sha256": authority["boundary_sha256"], "source_contract_sha256": authority["source_contract_sha256"],
            "inventory_lock_sha256": authority["inventory_lock_sha256"], "cache_boundary_sha256": manifest["cache_boundary_sha256"],
            "physical_pdf_pages": RULES.PAGES, "domain_classification_sha256": domain["classification_sha256"],
            "anchors": anchors, "grammar": grammar, "raw_only": True, "promotion_allowed": False,
            "scope_fence": "declaration-spelling-only"}
    return {**body, "grammar_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate(cache_root: Path, catalog_path: Path = CATALOG) -> dict[str, object]:
    expected = rendered(cache_root)
    actual = artifact(ARTIFACT.document, catalog_path)
    artifact(ARTIFACT.self_hashed, actual, "grammar_sha256")
    artifact(ARTIFACT.forbidden, actual)
    if not exact(actual, expected):
        reject("declaration grammar is stale, mixed, incomplete, or promoted")
    return copy.deepcopy(actual)


def normalize(forms: object) -> list[str]:
    try:
        return RULES.normalize(forms)
    except RULES.GrammarError as error:
        reject(str(error))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--catalog", type=Path, default=CATALOG)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        value = rendered(args.cache_root) if args.emit_json else validate(args.cache_root, args.catalog)
        print(json.dumps(value, indent=2, sort_keys=True) if args.emit_json else "PASS: 5 base anchors and 27 formal GLES templates; source-only")
    except GrammarValidationError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
