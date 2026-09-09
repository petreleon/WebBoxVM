"""Offline validation for the GL/GLES registry fixture-only output."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONTRACT = HERE.parents[1] / "01-provenance-record"
MANIFEST = HERE.parents[2] / "01-input-inventory" / "manifest.toml"
sys.path.insert(0, str(CONTRACT))

from provenance_record import ProvenanceError, validate_record  # noqa: E402

INPUT_ID = "opengl-gles-registry"
GENERATOR = {"name": "f02-gl-gles-fixture", "version": "1"}
ARTIFACT_PATH = (
    "todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/"
    "03-generator-outputs/02-gl-gles-registry/fixture-output.json"
)
FIXTURE_FIELDS = frozenset(("artifact", "generator", "input_id", "input_sha256", "notice", "schema"))
NOTICE = "Fixture-only provenance output; no upstream registry bytes or runtime API data."


def reject(message: str) -> None:
    raise ProvenanceError(message)


def output_bytes(path: Path) -> bytes:
    try:
        return path.read_bytes()
    except OSError as error:
        reject(f"fixture output cannot be read: {error}")


def fixture_value(raw: bytes) -> dict[str, object]:
    try:
        value = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"fixture output is not JSON: {error}")
    if not isinstance(value, dict) or set(value) != FIXTURE_FIELDS or value.get("schema") != 1:
        reject("fixture output does not match the fixture-only schema")
    return value


def validate_fixture(record_path: Path, artifact_path: Path, manifest_path: Path = MANIFEST) -> dict[str, object]:
    record = validate_record(record_path, manifest_path)
    if record["artifact_kind"] != "generated":
        reject("fixture output must be generated")
    if record["artifact_path"] != ARTIFACT_PATH:
        reject("fixture output has an unexpected artifact_path")
    if record["generator"] != GENERATOR:
        reject("fixture output has an unexpected generator identity or version")
    references = record["inputs"]
    if not isinstance(references, list) or len(references) != 1 or references[0]["id"] != INPUT_ID:
        reject("fixture output must use only opengl-gles-registry")
    raw = output_bytes(artifact_path)
    if hashlib.sha256(raw).hexdigest() != record["output_sha256"]:
        reject("fixture output hash does not match the provenance record")
    value = fixture_value(raw)
    reference = references[0]
    if value != {
        "artifact": "fixture-only-gl-gles-registry-binding",
        "generator": GENERATOR,
        "input_id": INPUT_ID,
        "input_sha256": reference["sha256"],
        "notice": NOTICE,
        "schema": 1,
    }:
        reject("fixture output does not bind the accepted registry identity")
    return record


if __name__ == "__main__":
    validate_fixture(HERE / "fixture-output.provenance.json", HERE / "fixture-output.json")
    print("PASS: GL/GLES registry fixture provenance is valid")
