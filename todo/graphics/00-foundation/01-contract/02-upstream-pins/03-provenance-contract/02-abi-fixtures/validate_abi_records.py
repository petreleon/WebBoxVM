"""Validate the checked-in ABI provenance sidecars without network access."""

from __future__ import annotations

import hashlib
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[6]
MANIFEST = HERE.parents[1] / "01-input-inventory" / "manifest.toml"
RECORDS = HERE / "records"
sys.path.insert(0, str(HERE.parent / "01-provenance-record"))

from provenance_record import ProvenanceError, validate_record

EXPECTED = {
    "emulator-virgl-capset.json": (
        "emulator/src/devices/virtio_gpu/three_d/capset.rs",
        ("virglrenderer-protocol",),
    ),
    "emulator-virgl-draw-fixture.json": (
        "emulator/src/devices/virtio_gpu/tests/virgl_draw_fixture.rs",
        ("mesa-virgl-screen", "virglrenderer-protocol"),
    ),
    "guest-virgl-kms.json": (
        "guest/virgl-clear-demo/kms.h",
        ("linux-virtio-gpu-uapi",),
    ),
    "guest-virgl-uapi.json": (
        "guest/virgl-clear-demo/uapi.h",
        ("linux-virtio-gpu-uapi",),
    ),
    "guest-virgl-wire.json": (
        "guest/virgl-clear-demo/virgl.h",
        ("mesa-virgl-screen", "virglrenderer-protocol"),
    ),
    "guest-webgpu-uapi.json": (
        "guest/webgpu-demo/uapi.h",
        ("linux-virtio-gpu-uapi",),
    ),
}


class AbiRecordError(ValueError):
    """An ABI sidecar does not match the selected local adapter."""


def reject(message: str) -> None:
    raise AbiRecordError(message)


def sha256(path: Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        reject(f"artifact cannot be read: {error}")


def validate_all(records: Path = RECORDS, repository: Path = REPOSITORY) -> int:
    if not records.is_dir():
        reject("ABI record directory is missing")
    names = {path.name for path in records.glob("*.json")}
    if names != set(EXPECTED):
        reject("ABI record set does not match the selected fixture inventory")
    for name, (artifact, identifiers) in EXPECTED.items():
        record = validate_record(records / name, MANIFEST)
        if record["artifact_kind"] != "handwritten":
            reject(f"{name} must identify a maintained handwritten adapter")
        if record["artifact_path"] != artifact:
            reject(f"{name} has an unexpected artifact path")
        declared = tuple(reference["id"] for reference in record["inputs"])
        if declared != identifiers:
            reject(f"{name} declared inputs do not match the selected ABI scope")
        if sha256(repository / artifact) != record["output_sha256"]:
            reject(f"{name} output_sha256 does not match its artifact")
    return len(EXPECTED)


def main() -> int:
    try:
        print(f"PASS: {validate_all()} ABI provenance records")
    except (AbiRecordError, ProvenanceError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
