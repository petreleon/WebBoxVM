#!/usr/bin/env python3
"""Validate F03.2.2.5.2's source-only OpenGL declaration grammar."""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE_PATH = HERE.parents[1] / "01-normative-pdf-cache/opengl_normative_pdf_cache.py"
RULES_PATH, ARTIFACT, MAX_SERIALIZED = HERE / "opengl_declaration_grammar_rules.py", HERE / "opengl_declaration_grammar.json", 8 * 1024 * 1024
CLAIMS = {key: False for key in ("khronos_selector", "api_support", "conformance", "certification", "profile_support", "performance")}
FORBIDDEN = frozenset(("requirement_kind", "status", "implementation_owner", "independent_test_plan", "owner_task", "test_source_role", "evidence", "coverage"))


class GrammarError(ValueError):
    """The rule set is stale, promoted, or not based on the sealed PDF."""


def reject(message: str) -> None:
    raise GrammarError(message)


def private(path: Path, name: str):
    if path.is_symlink() or not path.is_file():
        reject("fixed private dependency must be a regular file")
    previous = sys.modules.get(name)
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
    except GrammarError:
        raise
    except Exception as error:
        reject(f"cannot load fixed private dependency: {error}")
    finally:
        if previous is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = previous


CACHE, RULES = private(CACHE_PATH, "f032252_pdf_cache"), private(RULES_PATH, "f032252_declaration_rules")


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def pairs(items):
    value = {}
    for key, item in items:
        if key in value:
            reject("normalization artifact has duplicate JSON fields")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    try:
        raw = path.read_bytes()
        if len(raw) > MAX_SERIALIZED:
            reject("normalization artifact exceeds the serialized-size cap")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"normalization artifact cannot be read: {error}")
    if not isinstance(value, dict):
        reject("normalization artifact is not a JSON object")
    return value


def source_input(cache_root: Path) -> tuple[dict[str, object], dict[str, object], dict[str, object], bytes]:
    locator = "opengl46-core-pdf-v1:page=32;section=2.2"
    try:
        authority = CACHE.SOURCE_API.authority()
        boundary, consumed = authority.validate(), authority.consume("command-object-state", locator)
        receipt, manifest = CACHE.inspect(cache_root, locator), CACHE.manifest()
        source = receipt["source"]
        raw = CACHE.pdf_bytes(CACHE.external_root(cache_root), source)
    except Exception as error:
        reject(str(error))
    if (not isinstance(boundary, dict) or not isinstance(consumed, dict) or not isinstance(source, dict)
            or boundary.get("normative_root") != source or consumed.get("source") != source
            or consumed.get("decision", {}).get("id") != "command-object-state" or manifest.get("source") != source
            or receipt.get("physical_pdf_pages") != RULES.PAGES):
        reject("F03.2.1 and F03.2.2.1 do not expose the exact admitted PDF")
    return source, manifest, boundary, raw


def rendered(cache_root: Path) -> dict[str, object]:
    source, manifest, boundary, raw = source_input(cache_root)
    try:
        rules = RULES.artifact_rules(raw)
    except RULES.RuleError as error:
        reject(str(error))
    body = {"schema": 1, "kind": "webboxvm-opengl46-declaration-normalization-grammar", "profile": RULES.PROFILE,
            "source_class": "command-object-state", "source": source, "authority_boundary_sha256": boundary["boundary_sha256"],
            "source_contract_sha256": boundary["source_contract_sha256"], "inventory_lock_sha256": boundary["inventory_lock_sha256"],
            "cache_boundary_sha256": manifest["cache_boundary_sha256"], "cache_layout": manifest["cache_layout"],
            "physical_pdf_pages": RULES.PAGES, "serialized_size_limit": MAX_SERIALIZED, "rules": rules,
            "claims": CLAIMS, "cts_executions": 0, "matrix_row_count": 0, "states": CACHE.STATES}
    return {**body, "normalization_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def fences(value: dict[str, object]) -> None:
    rules = value.get("rules")
    if not isinstance(rules, dict) or FORBIDDEN & set(value) or any(FORBIDDEN & set(item) for item in rules.get("literal_examples", []) if isinstance(item, dict)):
        reject("normalization artifact has a forbidden Matrix or promotion shape")
    templates = rules.get("template_examples")
    if not isinstance(templates, list) or len(templates) != 1 or templates[0].get("c_names") != ["glUniform1i", "glUniform1f", "glUniform2i", "glUniform2f", "glUniform3i", "glUniform3f", "glUniform4i", "glUniform4f"]:
        reject("normalization artifact omits or changes the formal Uniform expansion")
    if value.get("claims") != CLAIMS or value.get("cts_executions") != 0 or value.get("matrix_row_count") != 0:
        reject("normalization artifact promotes claims, CTS, or Matrix rows")


def validate(cache_root: Path, artifact_path: Path = ARTIFACT) -> dict[str, object]:
    value = document(artifact_path)
    body = {key: item for key, item in value.items() if key != "normalization_sha256"}
    if value.get("normalization_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        reject("normalization artifact has a stale self hash")
    fences(value)
    if value != rendered(cache_root):
        reject("normalization artifact is stale, mixed, partial, reordered, or incorrectly anchored")
    return copy.deepcopy(value)


def reject_matrix_row(row: object, cache_root: Path, artifact_path: Path = ARTIFACT) -> None:
    validate(cache_root, artifact_path)
    reject("declaration normalization cannot create a Matrix v2 row")


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
            print("PASS: 2 literal examples and 8 formal template expansions; matrix-incomplete")
    except GrammarError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
