#!/usr/bin/env python3
"""Validate F03.3.2.2.1's closed, raw-only GLES source-family map."""
from __future__ import annotations
import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
RULES_PATH = HERE / "gles_command_domain_rules.py"
ARTIFACT_PATH = HERE / "gles_command_domain_artifact.py"
CROSSCHECKS_PATH = HERE / "gles_command_domain_crosschecks.py"
CACHE_PATH = ROOT / "01-normative-pdf-cache/gles_normative_pdf_cache.py"
LIMIT_PATH = ROOT / "03-limit-format-raw-inventory/gles_limit_format_raw_inventory.py"
LEDGER_PATH = ROOT / "04-unavailable-language-extension-ledger/gles_unavailable_ledger.py"
CATALOG = HERE / "gles_command_domain_classification.json"
INDEX = HERE / "gles_command_domain_index_crosscheck.json"
CHUNKS = {name: HERE / f"gles_command_domain_{name}.json" for name in
          ("declarations", "object-declarations", "execution", "state-lifecycle", "state-sources", "exclusions")}
class ClassificationError(ValueError):
    """The source-family map is stale, promoted, ambiguous, or incomplete."""
def reject(message: str) -> None:
    raise ClassificationError(message)
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
    except ClassificationError:
        raise
    except Exception as error:
        reject(f"cannot load fixed private dependency: {error}")
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior
ARTIFACT = private(ARTIFACT_PATH, "f033221_gles_domain_artifact")
RULES = private(RULES_PATH, "f033221_gles_domain_rules")
CROSSCHECKS = private(CROSSCHECKS_PATH, "f033221_gles_domain_crosschecks")
CACHE = private(CACHE_PATH, "f033221_gles_pdf_cache")
LIMIT = private(LIMIT_PATH, "f033221_gles_limit_inventory")
LEDGER = private(LEDGER_PATH, "f033221_gles_unavailable_ledger")
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
        limit, ledger = LIMIT.validate(cache_root), LEDGER.validate()
    except Exception as error:
        reject(str(error))
    if (not exact(authority.get("normative_root"), source) or not exact(receipt.get("source"), source)
            or not exact(manifest.get("source"), source) or decision.get("id") != "command-state"
            or receipt.get("physical_pdf_pages") != RULES.PAGES or not exact(limit.get("source"), source)
            or limit.get("source_class") != "limit-format" or not exact(ledger.get("normative_root"), source)):
        reject("authority, cache, inventory, or ledger identity is not the exact GLES root")
    return source, authority, decision, manifest, limit, ledger, raw
def chunk(identifier: str, families: list[dict[str, object]]) -> dict[str, object]:
    body = {"schema": 1, "kind": f"webboxvm-gles32-command-domain-{identifier}", "profile": RULES.PROFILE,
            "families": families, "family_count": len(families),
            "families_sha256": hashlib.sha256(canonical(families)).hexdigest(), "raw_only": True,
            "promotion_allowed": False}
    return {**body, "chunk_sha256": hashlib.sha256(canonical(body)).hexdigest()}
def index_artifact(crosscheck: dict[str, object]) -> dict[str, object]:
    body = {"schema": 1, "kind": "webboxvm-gles32-command-domain-index-crosscheck", "profile": RULES.PROFILE,
            "index_omission_crosscheck": crosscheck, "raw_only": True, "promotion_allowed": False}
    return {**body, "index_sha256": hashlib.sha256(canonical(body)).hexdigest()}
def rendered(cache_root: Path) -> tuple[dict[str, object], dict[str, dict[str, object]], dict[str, object]]:
    source, authority, decision, manifest, limit, ledger, raw = source_input(cache_root)
    try:
        rows = RULES.family_rows(raw, RULES.PAGES, limit, ledger, CROSSCHECKS)
        index_crosscheck = CROSSCHECKS.index_crosscheck(raw, reject)
    except RULES.RuleError as error:
        reject(str(error))
    chunks = {identifier: chunk(identifier, rows[identifier]) for identifier in CHUNKS}
    index = index_artifact(index_crosscheck)
    all_rows = sorted((row for values in rows.values() for row in values), key=lambda row: row["source_order"])
    routes = [{"id": key, "target": target, "route_kind": kind} for key, (target, kind) in RULES.ROUTES.items()]
    details = [{"id": key, "filename": CHUNKS[key].name, "family_count": chunks[key]["family_count"],
                "families_sha256": chunks[key]["families_sha256"], "chunk_sha256": chunks[key]["chunk_sha256"]}
               for key in CHUNKS]
    body = {"schema": 1, "kind": "webboxvm-gles32-command-domain-classification", "profile": RULES.PROFILE,
            "source": source, "source_class": "command-state", "source_decision": decision,
            "source_authority_sha256": authority["boundary_sha256"], "source_contract_sha256": authority["source_contract_sha256"],
            "inventory_lock_sha256": authority["inventory_lock_sha256"], "cache_boundary_sha256": manifest["cache_boundary_sha256"],
            "cache_layout": manifest["cache_layout"], "physical_pdf_pages": RULES.PAGES,
            "limit_format_inventory_sha256": limit["inventory_sha256"], "unavailable_ledger_sha256": ledger["ledger_sha256"],
            "routes": routes, "chunks": details, "index_crosscheck_receipt": {"filename": INDEX.name, "index_sha256": index["index_sha256"]}, "source_family_count": len(all_rows),
            "source_family_sha256": hashlib.sha256(canonical(all_rows)).hexdigest(), "raw_only": True,
            "promotion_allowed": False, "scope_fence": "source-routing-only"}
    return {**body, "classification_sha256": hashlib.sha256(canonical(body)).hexdigest()}, chunks, index
def validate(cache_root: Path, catalog_path: Path = CATALOG, chunk_paths: dict[str, Path] | None = None, index_path: Path = INDEX) -> dict[str, object]:
    expected, expected_chunks, expected_index = rendered(cache_root)
    actual_chunks = {key: artifact(ARTIFACT.document, (chunk_paths or CHUNKS)[key]) for key in CHUNKS}
    for key, value in actual_chunks.items():
        artifact(ARTIFACT.self_hashed, value, "chunk_sha256"); artifact(ARTIFACT.forbidden, value)
        if not exact(value, expected_chunks[key]):
            reject("source-family chunk is stale, reordered, partial, mixed, or rerouted")
    index = artifact(ARTIFACT.document, index_path)
    artifact(ARTIFACT.self_hashed, index, "index_sha256"); artifact(ARTIFACT.forbidden, index)
    if not exact(index, expected_index):
        reject("source-family index crosscheck is stale, partial, mixed, or promoted")
    catalog = artifact(ARTIFACT.document, catalog_path)
    artifact(ARTIFACT.self_hashed, catalog, "classification_sha256"); artifact(ARTIFACT.forbidden, catalog)
    if not exact(catalog, expected):
        reject("source-family catalog is stale, reordered, partial, mixed, or rerouted")
    return copy.deepcopy(catalog)
def reject_promotion(cache_root: Path, catalog_path: Path = CATALOG) -> None:
    validate(cache_root, catalog_path)
    reject("source-family routing cannot emit a qualification row")
def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--catalog", type=Path, default=CATALOG)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        catalog, chunks, index = rendered(args.cache_root) if args.emit_json else (validate(args.cache_root, args.catalog), {}, {})
        if args.emit_json:
            print(json.dumps({"catalog": catalog, "chunks": chunks, "index": index}, indent=2, sort_keys=True))
        else:
            print(f"PASS: {catalog['source_family_count']} closed GLES source families; source-routing-only")
    except ClassificationError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
if __name__ == "__main__":
    main()
