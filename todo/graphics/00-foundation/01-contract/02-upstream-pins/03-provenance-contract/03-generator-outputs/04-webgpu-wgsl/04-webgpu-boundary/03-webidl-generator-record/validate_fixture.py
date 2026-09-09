"""Offline validation for the metadata-only WebGPU WebIDL provenance marker."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONTRACT = HERE.parents[3] / "01-provenance-record"
MANIFEST = HERE.parents[4] / "01-input-inventory" / "manifest.toml"
sys.path.insert(0, str(CONTRACT))
sys.path.insert(0, str(HERE.parents[4] / "01-input-inventory"))

from inventory_layout import InventoryLayoutError, load_inventory  # noqa: E402
from provenance_record import ProvenanceError, validate_record  # noqa: E402

INPUT_ID = "webgpu-idl"
INPUT = {
    "id": INPUT_ID,
    "sha256": "bd35b2fc04f12f7ec22a9c1ae2826060778f980d8ce45dc055b8a5a8c644266a",
    "license": "W3C Software License (webgpu.idl file header)",
}
SOURCE = {
    **INPUT,
    "source_family": "webgpu-idl",
    "immutable_url": (
        "https://raw.githubusercontent.com/gpuweb/gpuweb/"
        "e95743d3940e0ff3c267ab55ced9ae6120c7d416/webgpu.idl"
    ),
    "revision": "e95743d3940e0ff3c267ab55ced9ae6120c7d416",
    "bytes": 38618,
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
GENERATOR = {"name": "f02-webgpu-webidl-fixture", "version": "1"}
COMMAND = (
    "cd todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/"
    "03-generator-outputs/04-webgpu-wgsl/04-webgpu-boundary/03-webidl-generator-record && "
    "PYTHONDONTWRITEBYTECODE=1 python3 generate_fixture.py "
    "--manifest ../../../../../01-input-inventory/manifest.toml --output fixture-output.json"
)
ARTIFACT_PATH = (
    "todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/"
    "03-generator-outputs/04-webgpu-wgsl/04-webgpu-boundary/03-webidl-generator-record/"
    "fixture-output.json"
)
ARTIFACT = "fixture-only-webgpu-webidl-input-record"
NOTICE = (
    "Fixture-only provenance output; no upstream WebIDL bytes, parser, binding generator, "
    "browser API, guest device, renderer, compatibility, or performance data."
)
LIMITATION = (
    "Metadata-only WebIDL input accounting; this marker neither parses WebIDL nor generates "
    "bindings or implements WebGPU."
)
FIELDS = frozenset((
    "artifact", "generator", "input_id", "input_license", "input_sha256", "limitation", "notice",
    "schema",
))


def reject(message: str) -> None:
    raise ProvenanceError(message)


def reviewed_webidl(manifest_path: Path) -> None:
    try:
        entries = load_inventory(manifest_path).inputs
    except InventoryLayoutError as error:
        reject(f"inventory cannot be loaded: {error}")
    matches = [entry for entry in entries if entry.get("id") == INPUT_ID]
    if len(matches) != 1:
        reject("manifest must contain exactly one webgpu-idl input")
    for field, expected in SOURCE.items():
        if matches[0].get(field) != expected:
            reject(f"manifest input webgpu-idl has unexpected {field}")


def fixture_value(raw: bytes) -> dict[str, object]:
    try:
        value = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"fixture output is not JSON: {error}")
    if (
        not isinstance(value, dict)
        or set(value) != FIELDS
        or type(value.get("schema")) is not int
        or value["schema"] != 1
    ):
        reject("fixture output does not match the metadata-only schema")
    return value


def expected_marker() -> dict[str, object]:
    return {
        "artifact": ARTIFACT,
        "generator": GENERATOR,
        "input_id": INPUT_ID,
        "input_license": INPUT["license"],
        "input_sha256": INPUT["sha256"],
        "limitation": LIMITATION,
        "notice": NOTICE,
        "schema": 1,
    }


def validate_fixture(
    record_path: Path, artifact_path: Path, manifest_path: Path = MANIFEST,
) -> dict[str, object]:
    record = validate_record(record_path, manifest_path)
    reviewed_webidl(manifest_path)
    if record["artifact_kind"] != "generated":
        reject("fixture output must be generated")
    if record["artifact_path"] != ARTIFACT_PATH:
        reject("fixture output has an unexpected artifact_path")
    if record["generator"] != GENERATOR:
        reject("fixture output has an unexpected generator identity or version")
    if record["command"] != COMMAND:
        reject("fixture output has an unexpected generator command")
    if record["inputs"] != [INPUT]:
        reject("fixture output must use only webgpu-idl")
    try:
        raw = artifact_path.read_bytes()
    except OSError as error:
        reject(f"fixture output cannot be read: {error}")
    if hashlib.sha256(raw).hexdigest() != record["output_sha256"]:
        reject("fixture output hash does not match the provenance record")
    if fixture_value(raw) != expected_marker():
        reject("fixture output does not bind the accepted WebIDL identity and limitation")
    return record


if __name__ == "__main__":
    validate_fixture(HERE / "fixture-output.provenance.json", HERE / "fixture-output.json")
    print("PASS: WebGPU WebIDL fixture provenance is valid")
