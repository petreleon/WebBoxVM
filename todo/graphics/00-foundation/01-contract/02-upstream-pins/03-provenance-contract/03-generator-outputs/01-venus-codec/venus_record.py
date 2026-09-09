"""Validate the one fixture-only Venus generated-output provenance record."""

from __future__ import annotations

import hashlib
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONTRACT = HERE.parents[1] / "01-provenance-record"
sys.path.insert(0, str(CONTRACT))

from provenance_record import ProvenanceError, reject, validate_record

VENUS_INPUT = {
    "id": "venus-protocol-registry",
    "sha256": "d92839bc728fa9ad9a7decdc6b91df6fa1a0fb26cffae4009865f18a789e0535",
    "license": "Apache-2.0 OR MIT (SPDX file notice)",
}
GENERATOR = {"name": "webboxvm.fixture.venus-marker", "version": "1"}
COMMAND = (
    "PYTHONDONTWRITEBYTECODE=1 python3 ../venus_fixture_generator.py "
    "--input-id venus-protocol-registry "
    "--input-sha256 d92839bc728fa9ad9a7decdc6b91df6fa1a0fb26cffae4009865f18a789e0535 "
    "--output venus-codec-record.txt"
)
ARTIFACT_PATH = "venus-codec-record.txt"


def artifact_path(record_path: Path, record: dict[str, object]) -> Path:
    root = record_path.parent.resolve()
    artifact = (root / str(record["artifact_path"])).resolve(strict=False)
    try:
        artifact.relative_to(root)
    except ValueError:
        reject("artifact path escapes its provenance record directory")
    if not artifact.is_file():
        reject("generated fixture artifact is missing")
    return artifact


def validate_venus_record(record_path: Path, manifest_path: Path) -> dict[str, object]:
    record = validate_record(record_path, manifest_path)
    if record["artifact_kind"] != "generated":
        reject("Venus fixture must be a generated artifact")
    if record["generator"] != GENERATOR:
        reject("Venus fixture generator identity or version is not approved")
    if record["command"] != COMMAND or record["artifact_path"] != ARTIFACT_PATH:
        reject("Venus fixture command or artifact path is not approved")
    if record["inputs"] != [VENUS_INPUT]:
        reject("Venus fixture must bind only venus-protocol-registry")
    artifact = artifact_path(record_path, record)
    actual = hashlib.sha256(artifact.read_bytes()).hexdigest()
    if actual != record["output_sha256"]:
        reject("generated fixture bytes do not match output_sha256")
    return record
