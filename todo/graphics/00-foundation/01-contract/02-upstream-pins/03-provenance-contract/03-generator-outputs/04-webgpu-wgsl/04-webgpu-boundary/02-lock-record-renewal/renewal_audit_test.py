#!/usr/bin/env python3
"""Verify every current F02 provenance record binds the reviewed WebIDL lock."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[9]
CONTRACT = HERE.parents[3]
MANIFEST = HERE.parents[4] / "01-input-inventory" / "manifest.toml"
LOCK = MANIFEST.with_name("inventory.lock")
SPEC = ROOT / (
    "todo/graphics/00-foundation/02-reproducibility/03-file-layout/"
    "03-generation-and-checker/01-chunk-generator/fixture/input.json"
)
CHUNKER = ROOT / "scripts/graphics_chunker.py"
OLD_LOCK = "db22bb053108cb7dbd413d4ee221a8134c081c842ead538e4d5fe0c45789e75e"
RECORDS = {
    "02-abi-fixtures/records/emulator-virgl-capset.json": "3d5c0f8eaee6c555392b54e44fea658c8bbe57cd0895e2c98c58b04fc3ecb136",
    "02-abi-fixtures/records/emulator-virgl-draw-fixture.json": "2d947a73cd2d62edc3cbba593a8f1b919435a2243befa4626a87525a2769565e",
    "02-abi-fixtures/records/guest-virgl-kms.json": "f44a6457ab0f9cb0c3785502a33b52878ae8a2ea022e85d42753d42d68c0cf62",
    "02-abi-fixtures/records/guest-virgl-uapi.json": "7f6c1e079c06ab88b45fe5f2d8d252db6d1e5cc5c97227172f6a11c345d8bfa2",
    "02-abi-fixtures/records/guest-virgl-wire.json": "4003921aa664dc43312a6453feaad9d48709d84c6d12ffbe68826fddc46d4ec5",
    "02-abi-fixtures/records/guest-webgpu-uapi.json": "3d3d5a9c62cd72b469b59a0614c85f4faff235707fd612739f9c5d1360f9f5ea",
    "03-generator-outputs/01-venus-codec/fixture/venus-codec-record.json": "b5cf425b0e5095631bcf8fab542ef8b85afc1785c738ad55e8be55c5fbd4ef72",
    "03-generator-outputs/02-gl-gles-registry/fixture-output.provenance.json": "e024d3cc9257768ae4cd011c06746ca1f7dca1e8bbff41bb0f838651e68c87c7",
    "03-generator-outputs/03-vulkan-spirv/spirv-core-grammar.provenance.json": "e0d62d479dda06c263d301323c2b1490b8b98ba8216ffc7d816c72d18187e18d",
    "03-generator-outputs/03-vulkan-spirv/vulkan-registry.provenance.json": "e071d5673e668e820a4573661285dadaa74c1188e6ca027eecdefb9130c43e12",
    "03-generator-outputs/04-webgpu-wgsl/03-wgsl-generator-record/fixture-output.provenance.json": "29fba0e91ca64f26f4ea5627bfb333c87703edabb82843e9a2c6f452a750c4b9",
    "03-generator-outputs/04-webgpu-wgsl/04-webgpu-boundary/03-webidl-generator-record/fixture-output.provenance.json": "99eb64f7239ee113813b1787492b34117d7e7c369390144d8770209c4301c311",
}
SPEC_FINGERPRINT = "16db3db47a8ddfefe60a781d9ed65e2f3d5f4c0ddb9446c2fed2b5d6b27c1ac0"

sys.path.insert(0, str(CONTRACT / "01-provenance-record"))

from provenance_record import ProvenanceError, validate_record  # noqa: E402


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def nonderived_fingerprint(document: dict[str, object]) -> str:
    fields = dict(document)
    fields.pop("inventory_sha256")
    return digest(json.dumps(fields, sort_keys=True, separators=(",", ":")).encode())


def sidecars() -> set[str]:
    return {
        path.relative_to(CONTRACT).as_posix()
        for path in CONTRACT.rglob("*.json")
        if "inventory_sha256" in load(path)
    }


def chunker(spec: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(CHUNKER), "--check", "--spec", str(spec), "--inventory-lock",
         str(LOCK), "--output", str(SPEC.parent / "expected")],
        cwd=ROOT, text=True, capture_output=True, check=False,
    )


class RenewalAuditTests(unittest.TestCase):
    def test_current_records_bind_the_new_lock_and_preserve_all_other_fields(self) -> None:
        revision = digest(LOCK.read_bytes())
        self.assertNotEqual(revision, OLD_LOCK)
        self.assertEqual(set(RECORDS), sidecars())
        for relative, expected in RECORDS.items():
            with self.subTest(record=relative):
                path = CONTRACT / relative
                document = load(path)
                self.assertEqual(document["inventory_sha256"], revision)
                self.assertEqual(nonderived_fingerprint(document), expected)
                self.assertEqual(validate_record(path, MANIFEST)["inventory_sha256"], revision)

    def test_old_lock_is_rejected_for_every_current_record(self) -> None:
        with tempfile.TemporaryDirectory(dir=HERE) as temporary:
            root = Path(temporary)
            for index, relative in enumerate(RECORDS):
                with self.subTest(record=relative):
                    candidate = root / f"record-{index}.json"
                    document = load(CONTRACT / relative)
                    document["inventory_sha256"] = OLD_LOCK
                    candidate.write_text(json.dumps(document, sort_keys=True) + "\n", encoding="utf-8")
                    with self.assertRaisesRegex(ProvenanceError, "stale inventory"):
                        validate_record(candidate, MANIFEST)

    def test_f06_bundle_is_current_and_rejects_the_old_lock(self) -> None:
        document = load(SPEC)
        self.assertEqual(document["inventory_sha256"], digest(LOCK.read_bytes()))
        self.assertEqual(nonderived_fingerprint(document), SPEC_FINGERPRINT)
        current = chunker(SPEC)
        self.assertEqual(current.returncode, 0, current.stderr)
        with tempfile.TemporaryDirectory(dir=HERE) as temporary:
            stale = Path(temporary) / "input.json"
            document["inventory_sha256"] = OLD_LOCK
            stale.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
            rejected = chunker(stale)
            self.assertNotEqual(rejected.returncode, 0, rejected.stdout)
            self.assertIn("inventory lock identity does not match", rejected.stderr)


if __name__ == "__main__":
    unittest.main()
