#!/usr/bin/env python3
"""Generate a fixture-only WGSL grammar provenance marker."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[3] / "01-input-inventory"))

from inventory_layout import InventoryLayoutError, load_inventory  # noqa: E402

INPUT_ID = "wgsl-grammar-syntax"
GENERATOR = {"name": "f02-wgsl-grammar-fixture", "version": "1"}
INPUT = {
    "id": INPUT_ID,
    "source_family": "wgsl-grammar",
    "sha256": "838b6fd1d01e4efd06e233200479d57667e8f8ba74783598e51f8f6195f762a1",
    "license": "W3C Software and Document License (repo LICENSE.md; document)",
    "generated_code_role": "future WGSL grammar-input/parser-validation generator input; nonstandard BNF dialect",
}
NOTICE = "Fixture-only provenance output; no upstream grammar bytes, parser, compiler, or WebGPU API data."
LIMITATION = "Nonstandard BNF dialect only; this marker neither parses nor compiles WGSL."


def grammar_input(manifest_path: Path) -> dict[str, object]:
    try:
        inventory = load_inventory(manifest_path)
    except InventoryLayoutError as error:
        raise ValueError(f"inventory cannot be loaded: {error}") from error
    matches = [entry for entry in inventory.inputs if entry.get("id") == INPUT_ID]
    if len(matches) != 1:
        raise ValueError(f"manifest must contain exactly one {INPUT_ID} input")
    entry = matches[0]
    for field, expected in INPUT.items():
        if entry.get(field) != expected:
            raise ValueError(f"manifest input {INPUT_ID} has unexpected {field}")
    return entry


def payload(entry: dict[str, object]) -> dict[str, object]:
    return {
        "artifact": "fixture-only-wgsl-grammar-binding",
        "generator": GENERATOR,
        "input_id": INPUT_ID,
        "input_license": entry["license"],
        "input_sha256": entry["sha256"],
        "limitation": LIMITATION,
        "notice": NOTICE,
        "schema": 1,
    }


def write_fixture(manifest_path: Path, output_path: Path) -> None:
    encoded = json.dumps(payload(grammar_input(manifest_path)), indent=2, sort_keys=True)
    output_path.write_text(encoded + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    write_fixture(args.manifest, args.output)


if __name__ == "__main__":
    main()
