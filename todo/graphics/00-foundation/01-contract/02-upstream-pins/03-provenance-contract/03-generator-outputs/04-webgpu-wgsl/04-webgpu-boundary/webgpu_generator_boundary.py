#!/usr/bin/env python3
"""Fail closed while F02.1 lacks an explicit WebGPU generator input."""

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path

INPUT_FIELDS = frozenset((
    "id", "source_family", "immutable_url", "revision", "sha256", "bytes", "license",
    "local_cache", "generated_code_role", "provenance",
))
MANIFEST_FIELDS = frozenset(("schema", "cache_root", "cache_note", "required_families", "inputs"))
WEBGPU_GENERATOR_ROLE = "future WebGPU generator input"


class BoundaryError(ValueError):
    """The inventory cannot support a WebGPU generator record yet."""


def reject(message: str) -> None:
    raise BoundaryError(message)


def text(value: object, field: str) -> str:
    if not isinstance(value, str) or not value:
        reject(f"{field} must be a nonempty string")
    return value


def load_inventory(path: Path) -> dict[str, dict[str, object]]:
    try:
        document = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        reject(f"manifest cannot be read: {error}")
    if (not isinstance(document, dict) or set(document) != MANIFEST_FIELDS
            or type(document.get("schema")) is not int or document["schema"] != 1):
        reject("manifest does not match the F02.1 schema version")
    if (document["cache_root"] != "$XDG_CACHE_HOME"
            or not isinstance(document["cache_note"], str) or not document["cache_note"]):
        reject("manifest does not retain the F02.1 external-cache contract")
    entries = document["inputs"]
    if not isinstance(entries, list) or not entries:
        reject("manifest has no input inventory")
    inventory: dict[str, dict[str, object]] = {}
    for entry in entries:
        if not isinstance(entry, dict) or set(entry) != INPUT_FIELDS:
            reject("manifest input does not match the F02.1 schema")
        identifier = text(entry["id"], "input id")
        text(entry["source_family"], f"input {identifier} source_family")
        text(entry["generated_code_role"], f"input {identifier} generated_code_role")
        if identifier in inventory:
            reject(f"manifest has duplicate input {identifier}")
        inventory[identifier] = entry
    return inventory


def validate_webgpu_generator_input(inventory: dict[str, dict[str, object]], identifier: str) -> None:
    entry = inventory.get(identifier)
    if entry is None:
        reject(f"input {identifier} is not in the immutable inventory")
    family = entry["source_family"]
    role = entry["generated_code_role"]
    if family in ("wgsl", "wgsl-grammar"):
        reject(f"input {identifier} is WGSL-derived and cannot be a WebGPU generator input")
    if family != "webgpu":
        reject(f"input {identifier} is not a WebGPU source definition")
    if role != WEBGPU_GENERATOR_ROLE:
        reject(f"input {identifier} is reference-only or lacks the WebGPU generator designation")


def audit_blocker(path: Path) -> tuple[str, ...]:
    inventory = load_inventory(path)
    reasons = []
    for identifier in ("webgpu-spec", "wgsl-spec"):
        try:
            validate_webgpu_generator_input(inventory, identifier)
        except BoundaryError as error:
            reasons.append(str(error))
        else:
            reject(f"input {identifier} unexpectedly cleared the WebGPU boundary")
    eligible = [
        identifier for identifier, entry in inventory.items()
        if entry["source_family"] == "webgpu"
        and entry["generated_code_role"] == WEBGPU_GENERATOR_ROLE
    ]
    if eligible:
        reject("inventory now has an eligible WebGPU generator input: " + ", ".join(sorted(eligible)))
    return tuple(reasons)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    args = parser.parse_args()
    try:
        for reason in audit_blocker(args.manifest):
            print(f"BLOCKED: {reason}")
        print("BLOCKED: no explicit WebGPU generator input is present")
    except BoundaryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
