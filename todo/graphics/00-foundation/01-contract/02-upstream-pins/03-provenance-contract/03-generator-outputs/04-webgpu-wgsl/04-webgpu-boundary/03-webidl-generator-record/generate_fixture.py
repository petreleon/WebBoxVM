#!/usr/bin/env python3
"""Generate a metadata-only WebGPU WebIDL provenance marker."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[4] / "01-input-inventory"))

from inventory_layout import InventoryLayoutError, load_inventory  # noqa: E402

INPUT_ID = "webgpu-idl"
GENERATOR = {"name": "f02-webgpu-webidl-fixture", "version": "1"}
SOURCE = {
    "id": INPUT_ID,
    "source_family": "webgpu-idl",
    "immutable_url": (
        "https://raw.githubusercontent.com/gpuweb/gpuweb/"
        "e95743d3940e0ff3c267ab55ced9ae6120c7d416/webgpu.idl"
    ),
    "revision": "e95743d3940e0ff3c267ab55ced9ae6120c7d416",
    "sha256": "bd35b2fc04f12f7ec22a9c1ae2826060778f980d8ce45dc055b8a5a8c644266a",
    "bytes": 38618,
    "license": "W3C Software License (webgpu.idl file header)",
    "local_cache": (
        "webboxvm-graphics/f02/webgpu-idl/"
        "bd35b2fc04f12f7ec22a9c1ae2826060778f980d8ce45dc055b8a5a8c644266a.source"
    ),
    "generated_code_role": (
        "future WebGPU WebIDL binding/interop generator input; "
        "no semantic or runtime implementation"
    ),
    "provenance": (
        "https://github.com/gpuweb/gpuweb/tree/"
        "e95743d3940e0ff3c267ab55ced9ae6120c7d416"
    ),
}
ARTIFACT = "fixture-only-webgpu-webidl-input-record"
NOTICE = (
    "Fixture-only provenance output; no upstream WebIDL bytes, parser, binding generator, "
    "browser API, guest device, renderer, compatibility, or performance data."
)
LIMITATION = (
    "Metadata-only WebIDL input accounting; this marker neither parses WebIDL nor generates "
    "bindings or implements WebGPU."
)


def reviewed_webidl(manifest_path: Path) -> dict[str, object]:
    try:
        entries = load_inventory(manifest_path).inputs
    except InventoryLayoutError as error:
        raise ValueError(f"inventory cannot be loaded: {error}") from error
    matches = [entry for entry in entries if entry.get("id") == INPUT_ID]
    if len(matches) != 1:
        raise ValueError("manifest must contain exactly one webgpu-idl input")
    entry = matches[0]
    for field, expected in SOURCE.items():
        if entry.get(field) != expected:
            raise ValueError(f"manifest input webgpu-idl has unexpected {field}")
    return entry


def payload() -> dict[str, object]:
    return {
        "artifact": ARTIFACT,
        "generator": GENERATOR,
        "input_id": INPUT_ID,
        "input_license": SOURCE["license"],
        "input_sha256": SOURCE["sha256"],
        "limitation": LIMITATION,
        "notice": NOTICE,
        "schema": 1,
    }


def write_fixture(manifest_path: Path, output_path: Path) -> None:
    reviewed_webidl(manifest_path)
    output_path.write_text(json.dumps(payload(), indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    write_fixture(args.manifest, args.output)


if __name__ == "__main__":
    main()
