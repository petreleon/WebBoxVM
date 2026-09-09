#!/usr/bin/env python3
"""Hermetic F02.3.1 tests for manifest-bound provenance records."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from provenance_record import ProvenanceError, validate_record

HERE = Path(__file__).resolve().parent
MANIFEST = HERE.parents[1] / "01-input-inventory" / "manifest.toml"
MANIFEST_SHA256 = "8f81ece8dc895698f2715493c927b3f5ad10f24a9f20f1b48d82405a4a9a288e"
INPUT_SHA256 = "7c9e2f7d47fa0b1a2c737fc5a741f57c5cf25303dd5c68c2c9738e9bb761eee6"
INPUT_LICENSE = "BSD-3-Clause (pinned file's explicit BSD notice)"
MESA_SHA256 = "e558fa5550e572cffad113581b736696f33b3895e0516950181b8ff93eb38ff0"
MESA_LICENSE = "MIT (file notice)"
OUTPUT_SHA256 = "68f6a2d00f8e3dcfc5ddcec8e161188851264b216d7b8b80b4bbf50f50644c82"


def sample() -> dict[str, object]:
    return {
        "schema": 1,
        "manifest_sha256": MANIFEST_SHA256,
        "inputs": [{"id": "linux-virtio-gpu-uapi", "sha256": INPUT_SHA256, "license": INPUT_LICENSE}],
        "command": "manual provenance review; no generator",
        "generator": {"name": "none", "version": "none"},
        "artifact_kind": "handwritten",
        "artifact_path": "examples/provenance-sample.txt",
        "output_sha256": OUTPUT_SHA256,
    }


class ProvenanceRecordTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.record_path = Path(self.temporary.name) / "record.json"

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def write(self, record: object) -> None:
        self.record_path.write_text(json.dumps(record, sort_keys=True), encoding="utf-8")

    def reject(self, record: object, message: str) -> None:
        self.write(record)
        with self.assertRaisesRegex(ProvenanceError, message):
            validate_record(self.record_path, MANIFEST)

    def test_valid_handwritten_record_binds_manifest_identity(self) -> None:
        record = sample()
        self.write(record)
        accepted = validate_record(self.record_path, MANIFEST)
        self.assertEqual(accepted["manifest_sha256"], MANIFEST_SHA256)
        self.assertEqual(accepted["inputs"], record["inputs"])

    def test_missing_required_field_is_rejected(self) -> None:
        record = sample()
        record.pop("command")
        self.reject(record, "schema version 1")

    def test_wrong_schema_type_is_rejected(self) -> None:
        record = sample()
        record["schema"] = True
        self.reject(record, "schema version 1")

    def test_unknown_input_is_rejected(self) -> None:
        record = sample()
        record["inputs"][0]["id"] = "unknown-input"
        self.reject(record, "unknown")

    def test_stale_manifest_and_input_identities_are_rejected(self) -> None:
        stale_manifest = sample()
        stale_manifest["manifest_sha256"] = "f" * 64
        self.reject(stale_manifest, "stale manifest")
        stale_input = sample()
        stale_input["inputs"][0]["sha256"] = "f" * 64
        self.reject(stale_input, "stale sha256")

    def test_stale_license_and_dishonest_kind_are_rejected(self) -> None:
        stale_license = sample()
        stale_license["inputs"][0]["license"] = "wrong"
        self.reject(stale_license, "stale license")
        dishonest = sample()
        dishonest["generator"] = {"name": "tool", "version": "1"}
        self.reject(dishonest, "generator none")

    def test_copied_upstream_and_handwritten_origin_rules(self) -> None:
        copied = sample()
        copied["artifact_kind"], copied["output_sha256"] = "copied-upstream", INPUT_SHA256
        self.write(copied)
        self.assertEqual(validate_record(self.record_path, MANIFEST)["artifact_kind"], "copied-upstream")
        copied["output_sha256"] = OUTPUT_SHA256
        self.reject(copied, "copied-upstream artifact")
        masquerade = sample()
        masquerade["output_sha256"] = INPUT_SHA256
        self.reject(masquerade, "must not masquerade")

    def test_generated_duplicate_and_unsorted_references_are_rejected(self) -> None:
        generated = sample()
        generated["artifact_kind"] = "generated"
        generated["generator"] = {"name": "fixture-generator", "version": "1"}
        self.write(generated)
        self.assertEqual(validate_record(self.record_path, MANIFEST)["artifact_kind"], "generated")
        missing_generator = sample()
        missing_generator["artifact_kind"] = "generated"
        self.reject(missing_generator, "needs a generator")
        duplicate = sample()
        duplicate["inputs"].append(dict(duplicate["inputs"][0]))
        self.reject(duplicate, "duplicated")
        unsorted = sample()
        unsorted["inputs"].insert(0, {"id": "mesa-virgl-screen", "sha256": MESA_SHA256, "license": MESA_LICENSE})
        self.reject(unsorted, "not sorted")

    def test_malformed_record_is_rejected(self) -> None:
        self.record_path.write_text("{", encoding="utf-8")
        with self.assertRaisesRegex(ProvenanceError, "record cannot be read"):
            validate_record(self.record_path, MANIFEST)


if __name__ == "__main__":
    unittest.main()
