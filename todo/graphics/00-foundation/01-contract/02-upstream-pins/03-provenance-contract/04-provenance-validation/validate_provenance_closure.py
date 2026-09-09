#!/usr/bin/env python3
"""Bind every F02.3 record to a complete, verified external F02.2 cache."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import dataclass
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[6]
UPSTREAM = HERE.parents[1]
CONTRACT = HERE.parent
MANIFEST = UPSTREAM / "01-input-inventory" / "manifest.toml"
FETCH = UPSTREAM / "02-fetch-verifier" / "01-fetch-contract"
sys.path[:0] = [str(FETCH), str(CONTRACT / "01-provenance-record")]

from provenance_record import ProvenanceError, validate_record  # noqa: E402
from source_cache import verify_payload  # noqa: E402
from source_model import ContractError, ExternalCache, SourceInput, load_manifest  # noqa: E402

ABI = "02-abi-fixtures/records/"
GENERATED = "03-generator-outputs/"
REPO_CONTRACT = "todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/"

@dataclass(frozen=True)
class Sample:
    sidecar: str
    recorded_artifact: str
    artifact: str
    fingerprint: str

def adapter(name: str, artifact: str, fingerprint: str) -> Sample:
    return Sample(ABI + name, artifact, artifact, fingerprint)


def generated(sidecar: str, recorded: str, artifact: str, fingerprint: str) -> Sample:
    return Sample(GENERATED + sidecar, recorded, artifact, fingerprint)


SAMPLES = (
    adapter("emulator-virgl-capset.json", "emulator/src/devices/virtio_gpu/three_d/capset.rs",
            "3d5c0f8eaee6c555392b54e44fea658c8bbe57cd0895e2c98c58b04fc3ecb136"),
    adapter("emulator-virgl-draw-fixture.json", "emulator/src/devices/virtio_gpu/tests/virgl_draw_fixture.rs",
            "2d947a73cd2d62edc3cbba593a8f1b919435a2243befa4626a87525a2769565e"),
    adapter("guest-virgl-kms.json", "guest/virgl-clear-demo/kms.h",
            "f44a6457ab0f9cb0c3785502a33b52878ae8a2ea022e85d42753d42d68c0cf62"),
    adapter("guest-virgl-uapi.json", "guest/virgl-clear-demo/uapi.h",
            "7f6c1e079c06ab88b45fe5f2d8d252db6d1e5cc5c97227172f6a11c345d8bfa2"),
    adapter("guest-virgl-wire.json", "guest/virgl-clear-demo/virgl.h",
            "4003921aa664dc43312a6453feaad9d48709d84c6d12ffbe68826fddc46d4ec5"),
    adapter("guest-webgpu-uapi.json", "guest/webgpu-demo/uapi.h",
            "3d3d5a9c62cd72b469b59a0614c85f4faff235707fd612739f9c5d1360f9f5ea"),
    generated("01-venus-codec/fixture/venus-codec-record.json", "venus-codec-record.txt",
              REPO_CONTRACT + GENERATED + "01-venus-codec/fixture/venus-codec-record.txt",
              "b5cf425b0e5095631bcf8fab542ef8b85afc1785c738ad55e8be55c5fbd4ef72"),
    generated("02-gl-gles-registry/fixture-output.provenance.json",
              REPO_CONTRACT + GENERATED + "02-gl-gles-registry/fixture-output.json",
              REPO_CONTRACT + GENERATED + "02-gl-gles-registry/fixture-output.json",
              "e024d3cc9257768ae4cd011c06746ca1f7dca1e8bbff41bb0f838651e68c87c7"),
    generated("03-vulkan-spirv/spirv-core-grammar.provenance.json",
              REPO_CONTRACT + GENERATED + "03-vulkan-spirv/fixture/spirv-core-grammar.fixture",
              REPO_CONTRACT + GENERATED + "03-vulkan-spirv/fixture/spirv-core-grammar.fixture",
              "e0d62d479dda06c263d301323c2b1490b8b98ba8216ffc7d816c72d18187e18d"),
    generated("03-vulkan-spirv/vulkan-registry.provenance.json",
              REPO_CONTRACT + GENERATED + "03-vulkan-spirv/fixture/vulkan-registry.fixture",
              REPO_CONTRACT + GENERATED + "03-vulkan-spirv/fixture/vulkan-registry.fixture",
              "e071d5673e668e820a4573661285dadaa74c1188e6ca027eecdefb9130c43e12"),
    generated("04-webgpu-wgsl/03-wgsl-generator-record/fixture-output.provenance.json",
              REPO_CONTRACT + GENERATED + "04-webgpu-wgsl/03-wgsl-generator-record/fixture-output.json",
              REPO_CONTRACT + GENERATED + "04-webgpu-wgsl/03-wgsl-generator-record/fixture-output.json",
              "29fba0e91ca64f26f4ea5627bfb333c87703edabb82843e9a2c6f452a750c4b9"),
    generated("04-webgpu-wgsl/04-webgpu-boundary/03-webidl-generator-record/fixture-output.provenance.json",
              REPO_CONTRACT + GENERATED + "04-webgpu-wgsl/04-webgpu-boundary/03-webidl-generator-record/fixture-output.json",
              REPO_CONTRACT + GENERATED + "04-webgpu-wgsl/04-webgpu-boundary/03-webidl-generator-record/fixture-output.json",
              "99eb64f7239ee113813b1787492b34117d7e7c369390144d8770209c4301c311"),
)

class ClosureError(ValueError):
    """The fresh-cache provenance closure is incomplete or inconsistent."""


def reject(message: str) -> None:
    raise ClosureError(message)

def fingerprint(record: dict[str, object]) -> str:
    fields = dict(record)
    fields.pop("inventory_sha256", None)
    return hashlib.sha256(json.dumps(fields, sort_keys=True, separators=(",", ":")).encode()).hexdigest()

def sidecars(contract: Path, samples: tuple[Sample, ...] = SAMPLES) -> set[str]:
    outputs = {sample.artifact[len(REPO_CONTRACT):] for sample in samples
               if sample.artifact.startswith(REPO_CONTRACT) and sample.artifact.endswith(".json")}
    return {
        relative
        for path in contract.rglob("*.json")
        if (relative := path.relative_to(contract).as_posix()) not in outputs
    }

def cached_target(cache: ExternalCache, source: SourceInput) -> Path:
    raw = cache.root
    for part in source.local_cache.parts:
        raw /= part
        if raw.is_symlink():
            reject(f"cache input {source.identifier} is a symlink")
    try:
        target = cache.target(source)
        status = target.stat()
    except ContractError as error:
        reject(str(error))
    except OSError as error:
        reject(f"cache input {source.identifier} is unavailable: {error}")
    if not target.is_file():
        reject(f"cache input {source.identifier} is not a regular file")
    if status.st_size != source.byte_count:
        reject(f"cache input {source.identifier} byte count mismatch")
    return target

def verify_cached_inputs(cache: ExternalCache, sources: tuple[SourceInput, ...]) -> int:
    for source in sources:
        try:
            payload = cached_target(cache, source).read_bytes()
        except OSError as error:
            reject(f"cache input {source.identifier} is unavailable: {error}")
        try:
            verify_payload(source, payload)
        except ContractError as error:
            reject(str(error))
    return len(sources)

def check_output(path: Path, expected: object) -> None:
    try:
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        reject(f"artifact cannot be read: {error}")
    if actual != expected:
        reject(f"artifact {path} does not match output_sha256")

def validate_records(manifest: Path, repository: Path, contract: Path = CONTRACT,
                     samples: tuple[Sample, ...] = SAMPLES) -> int:
    expected = {sample.sidecar for sample in samples}
    if sidecars(contract, samples) != expected:
        reject("provenance record set does not match the reviewed closure catalog")
    for sample in samples:
        try:
            record = validate_record(contract / sample.sidecar, manifest)
        except ProvenanceError as error:
            reject(f"{sample.sidecar}: {error}")
        if record["artifact_path"] != sample.recorded_artifact:
            reject(f"{sample.sidecar} has an unexpected artifact_path")
        if fingerprint(record) != sample.fingerprint:
            reject(f"{sample.sidecar} differs from its reviewed command/input/output fingerprint")
        check_output(repository / sample.artifact, record["output_sha256"])
    return len(samples)

def validate_closure(manifest: Path, cache_root: Path, repository: Path = REPOSITORY,
                     contract: Path = CONTRACT, samples: tuple[Sample, ...] = SAMPLES) -> tuple[int, int]:
    sources = load_manifest(manifest)
    cache = ExternalCache.from_path(cache_root, repository)
    return verify_cached_inputs(cache, sources), validate_records(manifest, repository, contract, samples)

def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=MANIFEST)
    parser.add_argument("--cache-root", type=Path, required=True)
    args = parser.parse_args()
    try:
        cached, records = validate_closure(args.manifest, args.cache_root)
        print(f"PASS: {cached} cached sources and {records} provenance records")
    except (ClosureError, ContractError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)

if __name__ == "__main__":
    main()
