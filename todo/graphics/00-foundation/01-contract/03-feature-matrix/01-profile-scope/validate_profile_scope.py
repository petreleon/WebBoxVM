#!/usr/bin/env python3
"""Fail closed when F03 profiles lack their reviewed inventory inputs."""

from __future__ import annotations

import argparse
import importlib.util
import sys
from pathlib import Path


def load_contract():
    path = Path(__file__).resolve().with_name("profile_contract.py")
    spec = importlib.util.spec_from_file_location("profile_contract", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load profile contract: {path}")
    module = importlib.util.module_from_spec(spec)
    previous = sys.modules.get(spec.name)
    sys.modules[spec.name] = module
    try:
        spec.loader.exec_module(module)
    except Exception:
        if previous is None:
            sys.modules.pop(spec.name, None)
        else:
            sys.modules[spec.name] = previous
        raise
    return module


contract = load_contract()
MANIFEST, REQUIREMENTS_PATH, SCOPE = contract.MANIFEST, contract.REQUIREMENTS_PATH, contract.SCOPE
ScopeError, inventory, validate = contract.ScopeError, contract.inventory, contract.validate


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scope", type=Path, default=SCOPE)
    parser.add_argument("--requirements", type=Path, default=REQUIREMENTS_PATH)
    parser.add_argument("--manifest", type=Path, default=MANIFEST)
    parser.add_argument("--matrix", type=Path)
    args = parser.parse_args()
    try:
        missing = validate(args.scope, args.requirements, args.manifest, args.matrix)
    except ScopeError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    if missing:
        print("BLOCKED: missing required inventory inputs: " + ", ".join(missing))
        raise SystemExit(3)
    print("PASS: source gate is complete; profile matrices remain blocked")


if __name__ == "__main__":
    main()
