#!/usr/bin/env python3
"""Fail closed at the OpenGL 4.6 source-authority boundary."""

from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BINDINGS_PATH = HERE.parents[1] / "01-profile-scope" / "role_aware_bindings.py"
BOUNDARY = HERE / "opengl_source_authority.json"
PROFILE = "opengl-4.6-core"
LOCATOR_SYNTAX = "opengl46-core-pdf-v1:page=<positive-decimal>;section=<section-path>"
LOCATOR = re.compile(r"^opengl46-core-pdf-v1:page=[1-9][0-9]*;section=[1-9][0-9]*(?:\.[1-9][0-9]*)*$")
NO_CLAIMS = {name: False for name in ("khronos_selector", "api_support", "conformance",
                                      "certification", "profile_support", "performance")}
STATES = {"mandatory_role_inventory_complete": True, "profile_status": "blocked",
          "blocker": "matrix-incomplete", "api_support": False, "conformance": False,
          "certification": False, "profile_support": False, "performance": False}
EXPECTED = {
    "normative-root": {"profile": PROFILE, "role": "normative-root", "record_id": "opengl-46-core-spec",
                       "record_kind": "upstream-source", "scope": "normative-source",
                       "revision": "1cdd228e34966dd6b95bd203e9f84faba0f371a1",
                       "sha256": "a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee",
                       "bytes": 3003752},
    "full-suite-root": {"profile": PROFILE, "role": "full-suite-root", "record_id": "opengl-cts-gl46-main",
                        "record_kind": "full-suite-root", "scope": "full-conformance-suite",
                        "revision": "067e8832315e79817ede1c4863804e440f5d1c80",
                        "sha256": "e28bbbbfd0f6c8d711554a01aa45819bdc7ca963c9426e997dfd1daaeb1d7b17",
                        "bytes": 1353085, "suite_id": "opengl-cts-gl46-main",
                        "selector_path": "external/openglcts/data/gl_cts/data/mustpass/gl/khronos_mustpass/4.6.1.x/gl46-main.txt",
                        "unfiltered": True, "selector_scope": "Khronos full root; not core-only"},
}


class BoundaryError(ValueError):
    """The OpenGL inventory boundary is not safe to consume."""

def reject(message: str) -> None:
    raise BoundaryError(message)

def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def unique_object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            reject("boundary has duplicate JSON fields")
        value[key] = item
    return value

def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_object)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"boundary cannot be read: {error}")
    if not isinstance(value, dict):
        reject("boundary is not a JSON object")
    return value

def bindings_api():
    """Load the one allowed F02-facing API by fixed path under a private name."""
    name, previous = "f0321_role_aware_bindings", sys.modules.get("f0321_role_aware_bindings")
    spec = importlib.util.spec_from_file_location(name, BINDINGS_PATH)
    if spec is None or spec.loader is None:
        reject("cannot load fixed-path role-aware bindings")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    try:
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != BINDINGS_PATH:
            reject("role-aware bindings resolved from an unexpected path")
        return module
    except BoundaryError:
        raise
    except Exception as error:
        reject(f"cannot load fixed-path role-aware bindings: {error}")
    finally:
        if previous is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = previous

def locator_classes() -> list[dict[str, object]]:
    available = (("command-object-state", "OpenGL 4.6 core commands, objects, state, and transitions"),
                 ("limit-format", "OpenGL 4.6 core limits and format properties"))
    result = [{"id": identifier, "decision": "derivable", "availability": "available",
               "source_role": "normative-root", "source_locator_syntax": LOCATOR_SYNTAX,
               "permitted_extraction_scope": scope, "blocker": ""}
              for identifier, scope in available]
    result.extend({"id": identifier, "decision": "requires-separate-admission", "availability": "unavailable",
                   "source_role": None, "source_locator_syntax": None, "permitted_extraction_scope": "",
                   "blocker": "unadmitted-distinct-source"}
                  for identifier in ("shader", "extension"))
    return result

def checked_contract(contract: object, api) -> tuple[dict[str, object], dict[str, object]]:
    try:
        normative = api.binding_for(contract, PROFILE, "normative-root")
        suite = api.binding_for(contract, PROFILE, "full-suite-root")
    except api.BindingError as error:
        reject(str(error))
    if (not isinstance(contract, dict) or normative != EXPECTED["normative-root"]
            or suite != EXPECTED["full-suite-root"]):
        reject("contract does not expose the exact active OpenGL roots")
    if contract.get("claims") != NO_CLAIMS or contract.get("cts_executions") != 0 or contract.get("states") != STATES:
        reject("contract does not retain zero CTS executions and matrix-incomplete no-claim state")
    return normative, suite

def rendered(contract: object, api) -> dict[str, object]:
    normative, suite = checked_contract(contract, api)
    body = {"schema": 1, "kind": "webboxvm-opengl-source-authority-boundary", "profile": PROFILE,
            "source_contract_sha256": contract["source_contract_sha256"],
            "inventory_lock_sha256": contract["inventory_lock_sha256"], "normative_root": normative,
            "full_suite_root": suite, "locator_classes": locator_classes(), "claims": NO_CLAIMS,
            "cts_executions": 0, "states": STATES}
    return {**body, "boundary_sha256": hashlib.sha256(canonical(body)).hexdigest()}

def locked_contract(api, source_contract: Path | None, source_lock: Path | None) -> dict[str, object]:
    if (source_contract is None) != (source_lock is None):
        reject("source contract and source lock must be supplied together")
    try:
        return api.load_locked_source_contract() if source_contract is None else api.load_locked_source_contract(source_contract, source_lock)
    except api.BindingError as error:
        reject(str(error))

def validate(boundary_path: Path = BOUNDARY, source_contract: Path | None = None,
             source_lock: Path | None = None) -> dict[str, object]:
    api = bindings_api()
    expected = rendered(locked_contract(api, source_contract, source_lock), api)
    value = document(boundary_path)
    body = {key: item for key, item in value.items() if key != "boundary_sha256"}
    if value.get("boundary_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        reject("boundary has a stale self hash")
    if value != expected:
        reject("boundary differs from the exact active role-aware source decision")
    return copy.deepcopy(value)

def consume(locator_class: str, locator: str, boundary_path: Path = BOUNDARY) -> dict[str, object]:
    value = validate(boundary_path)
    matches = [item for item in value["locator_classes"] if item["id"] == locator_class]
    if len(matches) != 1:
        reject("legacy, auxiliary, or unknown locator class is unavailable")
    decision = matches[0]
    if decision["availability"] != "available":
        reject(f"{locator_class} requires separate source admission")
    if not isinstance(locator, str) or not LOCATOR.fullmatch(locator):
        reject("locator does not match the versioned OpenGL core PDF syntax")
    return {"source": copy.deepcopy(value["normative_root"]), "decision": copy.deepcopy(decision), "locator": locator}

def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--boundary", type=Path, default=BOUNDARY)
    parser.add_argument("--consume-class")
    parser.add_argument("--locator")
    args = parser.parse_args()
    try:
        if args.consume_class is not None:
            if args.locator is None:
                reject("--consume-class requires --locator")
            consume(args.consume_class, args.locator, args.boundary)
            print(f"PASS: admitted {args.consume_class} locator remains matrix-incomplete")
        else:
            value = validate(args.boundary)
            print(f"PASS: {len(value['locator_classes'])} OpenGL classes; 2 derivable, 2 unavailable; matrix-incomplete")
    except BoundaryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)

if __name__ == "__main__":
    main()
