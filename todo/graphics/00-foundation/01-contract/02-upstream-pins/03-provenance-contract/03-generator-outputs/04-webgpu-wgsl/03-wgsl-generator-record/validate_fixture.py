"""Offline validation for the fixture-only WGSL grammar provenance marker."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONTRACT = HERE.parents[2] / "01-provenance-record"
MANIFEST = HERE.parents[3] / "01-input-inventory" / "manifest.toml"
sys.path.insert(0, str(CONTRACT))
sys.path.insert(0, str(HERE.parents[3] / "01-input-inventory"))

from inventory_layout import InventoryLayoutError, load_inventory  # noqa: E402
from provenance_record import ProvenanceError, validate_record  # noqa: E402

INPUT = {
    "id": "wgsl-grammar-syntax",
    "sha256": "838b6fd1d01e4efd06e233200479d57667e8f8ba74783598e51f8f6195f762a1",
    "license": "W3C Software and Document License (repo LICENSE.md; document)",
}
GENERATOR = {"name": "f02-wgsl-grammar-fixture", "version": "1"}
COMMAND = (
    "cd todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/"
    "03-generator-outputs/04-webgpu-wgsl/03-wgsl-generator-record && "
    "PYTHONDONTWRITEBYTECODE=1 python3 generate_fixture.py "
    "--manifest ../../../../01-input-inventory/manifest.toml --output fixture-output.json"
)
ARTIFACT_PATH = (
    "todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/"
    "03-generator-outputs/04-webgpu-wgsl/03-wgsl-generator-record/fixture-output.json"
)
NOTICE = "Fixture-only provenance output; no upstream grammar bytes, parser, compiler, or WebGPU API data."
LIMITATION = "Nonstandard BNF dialect only; this marker neither parses nor compiles WGSL."
SOURCE_FAMILY = "wgsl-grammar"
ROLE = "future WGSL grammar-input/parser-validation generator input; nonstandard BNF dialect"
FIELDS = frozenset((
    "artifact", "generator", "input_id", "input_license", "input_sha256", "limitation", "notice", "schema",
))


def reject(message: str) -> None:
    raise ProvenanceError(message)


def fixture_value(raw: bytes) -> dict[str, object]:
    try:
        value = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"fixture output is not JSON: {error}")
    if not isinstance(value, dict) or set(value) != FIELDS or value.get("schema") != 1:
        reject("fixture output does not match the fixture-only schema")
    return value


def reviewed_grammar(manifest_path: Path) -> None:
    try:
        entries = load_inventory(manifest_path).inputs
    except InventoryLayoutError as error:
        reject(f"inventory cannot be loaded: {error}")
    matches = [entry for entry in entries if entry.get("id") == INPUT["id"]]
    if len(matches) != 1:
        reject("manifest must contain exactly one wgsl-grammar-syntax input")
    entry = matches[0]
    if entry.get("source_family") != SOURCE_FAMILY or entry.get("generated_code_role") != ROLE:
        reject("manifest WGSL grammar input has an unexpected family or generator role")


def validate_fixture(record_path: Path, artifact_path: Path, manifest_path: Path = MANIFEST) -> dict[str, object]:
    record = validate_record(record_path, manifest_path)
    reviewed_grammar(manifest_path)
    if record["artifact_kind"] != "generated":
        reject("fixture output must be generated")
    if record["artifact_path"] != ARTIFACT_PATH:
        reject("fixture output has an unexpected artifact_path")
    if record["generator"] != GENERATOR:
        reject("fixture output has an unexpected generator identity or version")
    if record["command"] != COMMAND:
        reject("fixture output has an unexpected generator command")
    if record["inputs"] != [INPUT]:
        reject("fixture output must use only wgsl-grammar-syntax")
    try:
        raw = artifact_path.read_bytes()
    except OSError as error:
        reject(f"fixture output cannot be read: {error}")
    if hashlib.sha256(raw).hexdigest() != record["output_sha256"]:
        reject("fixture output hash does not match the provenance record")
    value = fixture_value(raw)
    expected = {
        "artifact": "fixture-only-wgsl-grammar-binding",
        "generator": GENERATOR,
        "input_id": INPUT["id"],
        "input_license": INPUT["license"],
        "input_sha256": INPUT["sha256"],
        "limitation": LIMITATION,
        "notice": NOTICE,
        "schema": 1,
    }
    if value != expected:
        reject("fixture output does not bind the accepted grammar identity and limitation")
    return record


if __name__ == "__main__":
    validate_fixture(HERE / "fixture-output.provenance.json", HERE / "fixture-output.json")
    print("PASS: WGSL grammar fixture provenance is valid")
