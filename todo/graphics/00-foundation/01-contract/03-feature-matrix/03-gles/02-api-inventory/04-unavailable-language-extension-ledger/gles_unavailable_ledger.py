#!/usr/bin/env python3
"""Seal GLES classes that need a separately admitted language source."""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
LEDGER = HERE / "gles_unavailable_ledger.json"
AUTHORITY = HERE.parents[1] / "01-source-authority/gles_source_authority.py"
PROFILE = "gles-3.2"
UNAVAILABLE = ("shader", "precision", "extension")
SUBSTITUTES = {
    "essl-320-spec": "ESSL alias",
    "gl.xml": "registry XML",
    "desktop-glsl": "desktop GLSL",
    "gles-3.1": "lower GLES version",
    "essl-3.10": "lower ESSL version",
}
CLAIMS = {name: False for name in ("khronos_selector", "api_support", "conformance", "certification",
                                   "profile_support", "performance")}
STATES = {"mandatory_role_inventory_complete": True, "profile_status": "blocked",
          "blocker": "matrix-incomplete", "api_support": False, "conformance": False,
          "certification": False, "profile_support": False, "performance": False}
class LedgerError(ValueError):
    """A GLES source-decision ledger is not safe to consume."""
def reject(message: str) -> None:
    raise LedgerError(message)
def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")
def unique(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            reject("ledger has duplicate JSON fields")
        value[key] = item
    return value
def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"ledger cannot be read: {error}")
    if not isinstance(value, dict):
        reject("ledger is not a JSON object")
    return value
def authority_api():
    """Load F03.3.1 from its one fixed path under a private module name."""
    name, prior = "f03324_gles_source_authority", sys.modules.get("f03324_gles_source_authority")
    if AUTHORITY.is_symlink() or not AUTHORITY.is_file():
        reject("fixed F03.3.1 source authority is unavailable")
    try:
        spec = importlib.util.spec_from_file_location(name, AUTHORITY)
        if spec is None or spec.loader is None:
            reject("cannot load fixed F03.3.1 source authority")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != AUTHORITY.resolve():
            reject("F03.3.1 source authority resolved from an unexpected path")
        return module
    except LedgerError:
        raise
    except Exception as error:
        reject(f"cannot load fixed F03.3.1 source authority: {error}")
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior
def boundary(source_boundary: Path | None = None, source_contract: Path | None = None,
             source_lock: Path | None = None):
    api = authority_api()
    try:
        value = api.validate(api.BOUNDARY if source_boundary is None else source_boundary,
                             source_contract, source_lock)
    except Exception as error:
        reject(str(error))
    if (value.get("profile") != PROFILE or value.get("claims") != CLAIMS or value.get("cts_executions") != 0
            or value.get("states") != STATES):
        reject("F03.3.1 does not retain the exact no-claim GLES state")
    return value
def decisions(value: dict[str, object]) -> list[dict[str, object]]:
    choices = value.get("locator_classes")
    if not isinstance(choices, list):
        reject("F03.3.1 has no locator decisions")
    selected = [item for item in choices if isinstance(item, dict) and item.get("id") in UNAVAILABLE]
    expected = [{"id": identifier, "decision": "requires-separate-admission", "availability": "unavailable",
                 "source_role": None, "source_locator_syntax": None, "permitted_extraction_scope": "",
                 "blocker": "unadmitted-distinct-source"} for identifier in UNAVAILABLE]
    if selected != expected:
        reject("F03.3.1 unavailable classes are stale, mixed, reordered, or promoted")
    return copy.deepcopy(selected)
def rendered(source: dict[str, object]) -> dict[str, object]:
    body = {
        "schema": 1,
        "kind": "webboxvm-gles-unavailable-language-extension-ledger",
        "profile": PROFILE,
        "source_authority_sha256": source["boundary_sha256"],
        "source_contract_sha256": source["source_contract_sha256"],
        "inventory_lock_sha256": source["inventory_lock_sha256"],
        "normative_root": copy.deepcopy(source["normative_root"]),
        "full_suite_root": copy.deepcopy(source["full_suite_root"]),
        "unavailable_classes": decisions(source),
        "rejected_substitutes": list(SUBSTITUTES),
        "matrix_row_count": 0,
        "claims": CLAIMS,
        "cts_executions": 0,
        "states": STATES,
    }
    return {**body, "ledger_sha256": hashlib.sha256(canonical(body)).hexdigest()}
def validate(ledger_path: Path = LEDGER, source_boundary: Path | None = None,
             source_contract: Path | None = None, source_lock: Path | None = None) -> dict[str, object]:
    expected = rendered(boundary(source_boundary, source_contract, source_lock))
    value = document(ledger_path)
    body = {key: item for key, item in value.items() if key != "ledger_sha256"}
    if value.get("ledger_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        reject("ledger has a stale self hash")
    if value != expected:
        reject("ledger differs from the exact F03.3.1 source decision")
    return copy.deepcopy(value)
def unavailable_class(identifier: str, ledger_path: Path = LEDGER) -> dict[str, object]:
    value = validate(ledger_path)
    matches = [item for item in value["unavailable_classes"] if item["id"] == identifier]
    if len(matches) != 1:
        reject("unknown or admitted class cannot use the unavailable-source ledger")
    return copy.deepcopy(matches[0])
def reject_substitute(identifier: str, candidate: str, ledger_path: Path = LEDGER) -> None:
    unavailable_class(identifier, ledger_path)
    label = SUBSTITUTES.get(candidate, "unadmitted source") if isinstance(candidate, str) else "invalid source"
    reject(f"{label} cannot replace unavailable {identifier}; separate F02 admission is required")
def reject_matrix_row(row: object, ledger_path: Path = LEDGER) -> None:
    if not isinstance(row, dict) or not isinstance(row.get("requirement_kind"), str):
        reject("Matrix v2 row does not identify an unavailable GLES class")
    unavailable_class(row["requirement_kind"], ledger_path)
    if row.get("status") != "blocked":
        reject("unavailable source class cannot be promoted")
    reject("unavailable source class cannot create a Matrix v2 row")


def reject_matrix_promotion(identifier: str, status: str, ledger_path: Path = LEDGER) -> None:
    reject_matrix_row({"requirement_kind": identifier, "status": status}, ledger_path)
def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ledger", type=Path, default=LEDGER)
    parser.add_argument("--emit-json", action="store_true")
    parser.add_argument("--reject-class")
    parser.add_argument("--candidate")
    args = parser.parse_args()
    try:
        if args.emit_json:
            print(json.dumps(rendered(boundary()), indent=2, sort_keys=True))
        elif args.reject_class is not None:
            reject_substitute(args.reject_class, args.candidate, args.ledger)
        else:
            validate(args.ledger)
            print("PASS: 3 unavailable GLES classes; matrix-incomplete")
    except LedgerError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
if __name__ == "__main__":
    main()
