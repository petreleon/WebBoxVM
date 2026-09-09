#!/usr/bin/env python3
"""Generate a fixture-only GL/GLES registry provenance artifact."""

from __future__ import annotations

import argparse
import json
import tomllib
from pathlib import Path

INPUT_ID = "opengl-gles-registry"
GENERATOR = {"name": "f02-gl-gles-fixture", "version": "1"}
NOTICE = "Fixture-only provenance output; no upstream registry bytes or runtime API data."


def registry_input(manifest_path: Path) -> dict[str, object]:
    document = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    matches = [entry for entry in document.get("inputs", []) if entry.get("id") == INPUT_ID]
    if len(matches) != 1:
        raise ValueError(f"manifest must contain exactly one {INPUT_ID} input")
    entry = matches[0]
    if not isinstance(entry.get("sha256"), str):
        raise ValueError(f"manifest input {INPUT_ID} must have a SHA-256")
    return entry


def payload(entry: dict[str, object]) -> dict[str, object]:
    return {
        "artifact": "fixture-only-gl-gles-registry-binding",
        "generator": GENERATOR,
        "input_id": INPUT_ID,
        "input_sha256": entry["sha256"],
        "notice": NOTICE,
        "schema": 1,
    }


def write_fixture(manifest_path: Path, output_path: Path) -> None:
    encoded = json.dumps(payload(registry_input(manifest_path)), indent=2, sort_keys=True)
    output_path.write_text(encoded + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    write_fixture(args.manifest, args.output)


if __name__ == "__main__":
    main()
