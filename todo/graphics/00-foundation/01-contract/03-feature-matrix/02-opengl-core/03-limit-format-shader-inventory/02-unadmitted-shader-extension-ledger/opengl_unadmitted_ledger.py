#!/usr/bin/env python3
"""Preserve F03.2.1's unadmitted shader and extension source decisions."""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
LEDGER = HERE / "opengl_unadmitted_ledger.json"
AUTHORITY = HERE.parents[1] / "01-source-authority/opengl_source_authority.py"
PROFILE = "opengl-4.6-core"
UNADMITTED = ("shader", "extension")
PINS = {
    "boundary_sha256": "71d1fdbe2bb51911d6fbf6fe4a5c5665681c5a4a8a265b3db840bc629762a641",
    "source_contract_sha256": "d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3",
    "inventory_lock_sha256": "44a0f280e0ed854091a33e458122bd8cd9a0f9fc2e4c51a7a5be92bf48d8c6f4",
}
SUBSTITUTES = ("desktop-glsl", "glsl-460-spec", "essl-320-spec", "gl.xml", "registry-metadata",
               "extension-list", "opengl-4.5-core", "opengl-4.6-compatibility")
CLAIMS = {key: False for key in ("khronos_selector", "api_support", "conformance", "certification",
                               "profile_support", "performance")}
STATES = {"mandatory_role_inventory_complete": True, "profile_status": "blocked",
          "blocker": "matrix-incomplete", "api_support": False, "conformance": False,
          "certification": False, "profile_support": False, "performance": False}


class LedgerError(ValueError):
    """The source decision cannot authorize consumption."""


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


def authority_api():
    """Execute the fixed authority file without trusting ambient module aliases."""
    name = "f03232_opengl_source_authority"
    prior = sys.modules.get(name)
    if AUTHORITY.is_symlink() or not AUTHORITY.is_file():
        reject("fixed F03.2.1 source authority is unavailable")
    try:
        spec = importlib.util.spec_from_file_location(name, AUTHORITY)
        if spec is None or spec.loader is None:
            reject("cannot load fixed F03.2.1 source authority")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != AUTHORITY.resolve():
            reject("source authority resolved from an unexpected path")
        return module
    except Exception as error:
        reject(f"cannot load fixed F03.2.1 source authority: {error}")
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


def boundary(source_boundary: Path | None = None, source_contract: Path | None = None,
             source_lock: Path | None = None) -> dict[str, object]:
    api = authority_api()
    try:
        value = api.validate(api.BOUNDARY if source_boundary is None else source_boundary,
                             source_contract, source_lock)
    except Exception as error:
        reject(str(error))
    if any(value.get(key) != digest for key, digest in PINS.items()):
        reject("F03.2.1 does not match the sealed F02 identities")
    actual = [value.get(key) for key in ("profile", "claims", "cts_executions", "states")]
    if canonical(actual) != canonical([PROFILE, CLAIMS, 0, STATES]):
        reject("F03.2.1 does not retain the exact no-claim OpenGL state")
    return value


def rendered(source: dict[str, object]) -> dict[str, object]:
    selected = [item for item in source["locator_classes"] if item["id"] in UNADMITTED]
    expected = [{"id": key, "decision": "requires-separate-admission", "availability": "unavailable",
                 "source_role": None, "source_locator_syntax": None, "permitted_extraction_scope": "",
                 "blocker": "unadmitted-distinct-source"} for key in UNADMITTED]
    if canonical(selected) != canonical(expected):
        reject("F03.2.1 unadmitted decisions are stale, mixed, reordered, or promoted")
    body = {"schema": 1, "kind": "webboxvm-opengl-unadmitted-shader-extension-ledger", "profile": PROFILE,
            "source_authority_sha256": source["boundary_sha256"],
            "source_contract_sha256": source["source_contract_sha256"],
            "inventory_lock_sha256": source["inventory_lock_sha256"],
            "normative_root": copy.deepcopy(source["normative_root"]),
            "full_suite_root": copy.deepcopy(source["full_suite_root"]),
            "unadmitted_classes": copy.deepcopy(selected), "rejected_substitutes": list(SUBSTITUTES),
            "matrix_row_count": 0, "semantic_fact_count": 0, "claims": CLAIMS,
            "cts_executions": 0, "states": STATES}
    return {**body, "ledger_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate(ledger_path: Path = LEDGER, source_boundary: Path | None = None,
             source_contract: Path | None = None, source_lock: Path | None = None) -> dict[str, object]:
    expected = rendered(boundary(source_boundary, source_contract, source_lock))
    try:
        value = json.loads(ledger_path.read_text(encoding="utf-8"), object_pairs_hook=unique)
        if not isinstance(value, dict):
            reject("ledger is not a JSON object")
        body = {key: item for key, item in value.items() if key != "ledger_sha256"}
        if value.get("ledger_sha256") != hashlib.sha256(canonical(body)).hexdigest():
            reject("ledger has a stale self hash")
        if canonical(value) != canonical(expected):
            reject("ledger differs from the exact F03.2.1 source decision")
    except (OSError, UnicodeDecodeError, ValueError) as error:
        reject(f"invalid ledger: {error}")
    return copy.deepcopy(value)


def unadmitted_class(identifier: str, ledger_path: Path = LEDGER) -> dict[str, object]:
    value = validate(ledger_path)
    matches = [item for item in value["unadmitted_classes"] if item["id"] == identifier]
    if len(matches) != 1:
        reject("unknown or admitted class cannot use the unadmitted-source ledger")
    return copy.deepcopy(matches[0])


def reject_substitute(identifier: str, candidate: object, ledger_path: Path = LEDGER) -> None:
    unadmitted_class(identifier, ledger_path)
    reject(f"candidate source cannot replace unadmitted {identifier}; separate F02 admission is required")


def reject_matrix_row(row: object, ledger_path: Path = LEDGER) -> None:
    if not isinstance(row, dict) or not isinstance(row.get("requirement_kind"), str):
        reject("Matrix v2 row does not identify an unadmitted OpenGL class")
    unadmitted_class(row["requirement_kind"], ledger_path)
    if row.get("profile") != PROFILE:
        reject("Matrix v2 row has a missing or cross-profile identity")
    if row.get("status") != "blocked":
        reject("unadmitted source class cannot be promoted")
    reject("unadmitted source class cannot create a Matrix v2 row")


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
            print("PASS: 2 unadmitted OpenGL classes; matrix-incomplete")
    except LedgerError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
