#!/usr/bin/env python3
"""Validate a blocked Vulkan-registry scaffold or inspect one exact payload."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from registry_inventory_contract import build_inventory
from registry_inventory_source import RegistryError, payload_bytes
from registry_inventory_validation import SCAFFOLD, validate_inventory, validate_scaffold


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scaffold", type=Path, default=SCAFFOLD)
    parser.add_argument("--payload", type=Path)
    parser.add_argument("--emit-json", action="store_true")
    args = parser.parse_args()
    try:
        identity = validate_scaffold(args.scaffold)
        if args.payload is None:
            print("BLOCKED: exact external vk.xml payload is required; scaffold has no technical rows")
            raise SystemExit(3)
        value = build_inventory(payload_bytes(args.payload, identity), identity)
        validate_inventory(value, identity)
    except RegistryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    if args.emit_json:
        print(json.dumps(value, sort_keys=True, separators=(",", ":")))
    else:
        print(f"BLOCKED: {len(value['rows'])} raw technical rows; every row remains unimplemented")


if __name__ == "__main__":
    main()
