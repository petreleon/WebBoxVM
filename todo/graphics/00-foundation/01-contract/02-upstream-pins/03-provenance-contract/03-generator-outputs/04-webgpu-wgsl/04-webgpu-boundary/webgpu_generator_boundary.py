#!/usr/bin/env python3
"""Accept only the reviewed WebGPU WebIDL provenance input."""

from __future__ import annotations

import argparse
import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[3] / "01-input-inventory"))

from inventory_layout import InventoryLayoutError, load_inventory as load_layout  # noqa: E402


def load_reviewed_webidl_identity() -> tuple[str, dict[str, object]]:
    """Load the sibling marker under a name no other task can preempt."""
    marker = HERE / "03-webidl-generator-record" / "validate_fixture.py"
    spec = importlib.util.spec_from_file_location("f02_webgpu_webidl_marker", marker)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load reviewed WebIDL marker: {marker}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.INPUT_ID, module.SOURCE


INPUT_ID, REVIEWED_WEBGPU_IDL = load_reviewed_webidl_identity()


class BoundaryError(ValueError):
    """The inventory does not identify exactly the reviewed WebIDL input."""


def reject(message: str) -> None:
    raise BoundaryError(message)


def text(value: object, field: str) -> str:
    if not isinstance(value, str) or not value:
        reject(f"{field} must be a nonempty string")
    return value


def load_inventory(path: Path) -> dict[str, dict[str, object]]:
    try:
        layout = load_layout(path)
    except InventoryLayoutError as error:
        reject(f"inventory cannot be loaded: {error}")
    inventory: dict[str, dict[str, object]] = {}
    for entry in layout.inputs:
        identifier = text(entry["id"], "input id")
        text(entry["source_family"], f"input {identifier} source_family")
        text(entry["generated_code_role"], f"input {identifier} generated_code_role")
        if identifier in inventory:
            reject(f"manifest has duplicate input {identifier}")
        inventory[identifier] = entry
    return inventory


def validate_webgpu_generator_input(
    inventory: dict[str, dict[str, object]], identifier: str,
) -> dict[str, object]:
    if identifier != INPUT_ID:
        reject(f"input {identifier} is not the reviewed WebGPU WebIDL generator input")
    entry = inventory.get(identifier)
    if entry is None:
        reject(f"input {identifier} is not in the immutable inventory")
    for field, expected in REVIEWED_WEBGPU_IDL.items():
        if entry.get(field) != expected:
            reject(f"input {INPUT_ID} has unexpected {field}")
    return entry


def audit_boundary(path: Path) -> dict[str, object]:
    return validate_webgpu_generator_input(load_inventory(path), INPUT_ID)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    args = parser.parse_args()
    try:
        audit_boundary(args.manifest)
        print("PASS: reviewed WebGPU WebIDL generator input is valid")
    except BoundaryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
