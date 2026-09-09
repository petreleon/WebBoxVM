#!/usr/bin/env python3
"""Hermetic F02.3.1 tests for lock-bound provenance records."""

from __future__ import annotations

import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "01-input-inventory"))

from inventory_layout import load_inventory
from provenance_record import ProvenanceError, validate_record

MANIFEST = HERE.parents[1] / "01-input-inventory" / "manifest.toml"
INPUT_SHA256 = "7c9e2f7d47fa0b1a2c737fc5a741f57c5cf25303dd5c68c2c9738e9bb761eee6"
INPUT_LICENSE = "BSD-3-Clause (pinned file's explicit BSD notice)"
MESA_SHA256 = "e558fa5550e572cffad113581b736696f33b3895e0516950181b8ff93eb38ff0"
MESA_LICENSE = "MIT (file notice)"
OUTPUT_SHA256 = "68f6a2d00f8e3dcfc5ddcec8e161188851264b216d7b8b80b4bbf50f50644c82"


def sample(revision: str, schema: int = 2) -> dict[str, object]:
    identity = "manifest_sha256" if schema == 1 else "inventory_sha256"
    return {
        "schema": schema,
        identity: revision,
        "inputs": [{"id": "linux-virtio-gpu-uapi", "sha256": INPUT_SHA256, "license": INPUT_LICENSE}],
        "command": "manual provenance review; no generator",
        "generator": {"name": "none", "version": "none"},
        "artifact_kind": "handwritten",
        "artifact_path": "examples/provenance-sample.txt",
        "output_sha256": OUTPUT_SHA256,
}


class ProvenanceRecordTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(dir=HERE)
        self.record_path = Path(self.temporary.name) / "record.json"

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def active_sample(self) -> dict[str, object]:
        return sample(load_inventory(MANIFEST).revision)

    def write(self, record: object) -> None:
        self.record_path.write_text(json.dumps(record, sort_keys=True), encoding="utf-8")

    def reject(self, record: object, message: str, manifest: Path = MANIFEST) -> None:
        self.write(record)
        with self.assertRaisesRegex(ProvenanceError, message):
            validate_record(self.record_path, manifest)

    def copied_inventory(self) -> Path:
        root = self.record_path.parent / "inventory"
        root.mkdir()
        shutil.copy2(MANIFEST, root / "manifest.toml")
        shutil.copytree(MANIFEST.parent / "inputs", root / "inputs")
        shutil.copy2(MANIFEST.with_name("inventory.lock"), root / "inventory.lock")
        return root / "manifest.toml"

    def test_valid_handwritten_record_binds_active_lock_identity(self) -> None:
        record = self.active_sample()
        self.write(record)
        accepted = validate_record(self.record_path, MANIFEST)
        self.assertEqual(accepted["inventory_sha256"], record["inventory_sha256"])
        self.assertEqual(accepted["inputs"], record["inputs"])

    def test_missing_required_field_is_rejected(self) -> None:
        record = self.active_sample()
        record.pop("command")
        self.reject(record, "schema version 2")

    def test_wrong_schema_type_is_rejected(self) -> None:
        record = self.active_sample()
        record["schema"] = True
        self.reject(record, "schema version 1 or 2")

    def test_unknown_input_is_rejected(self) -> None:
        record = self.active_sample()
        record["inputs"][0]["id"] = "unknown-input"
        self.reject(record, "unknown")

    def test_stale_lock_and_input_identities_are_rejected(self) -> None:
        stale_lock = self.active_sample()
        stale_lock["inventory_sha256"] = "f" * 64
        self.reject(stale_lock, "stale inventory")
        stale_input = self.active_sample()
        stale_input["inputs"][0]["sha256"] = "f" * 64
        self.reject(stale_input, "stale sha256")

    def test_stale_license_and_dishonest_kind_are_rejected(self) -> None:
        stale_license = self.active_sample()
        stale_license["inputs"][0]["license"] = "wrong"
        self.reject(stale_license, "stale license")
        dishonest = self.active_sample()
        dishonest["generator"] = {"name": "tool", "version": "1"}
        self.reject(dishonest, "generator none")

    def test_copied_upstream_and_handwritten_origin_rules(self) -> None:
        copied = self.active_sample()
        copied["artifact_kind"], copied["output_sha256"] = "copied-upstream", INPUT_SHA256
        self.write(copied)
        self.assertEqual(validate_record(self.record_path, MANIFEST)["artifact_kind"], "copied-upstream")
        copied["output_sha256"] = OUTPUT_SHA256
        self.reject(copied, "copied-upstream artifact")
        masquerade = self.active_sample()
        masquerade["output_sha256"] = INPUT_SHA256
        self.reject(masquerade, "must not masquerade")

    def test_generated_duplicate_and_unsorted_references_are_rejected(self) -> None:
        generated = self.active_sample()
        generated["artifact_kind"] = "generated"
        generated["generator"] = {"name": "fixture-generator", "version": "1"}
        self.write(generated)
        self.assertEqual(validate_record(self.record_path, MANIFEST)["artifact_kind"], "generated")
        missing_generator = self.active_sample()
        missing_generator["artifact_kind"] = "generated"
        self.reject(missing_generator, "needs a generator")
        duplicate = self.active_sample()
        duplicate["inputs"].append(dict(duplicate["inputs"][0]))
        self.reject(duplicate, "duplicated")
        unsorted = self.active_sample()
        unsorted["inputs"].insert(0, {"id": "mesa-virgl-screen", "sha256": MESA_SHA256, "license": MESA_LICENSE})
        self.reject(unsorted, "not sorted")

    def test_malformed_record_is_rejected(self) -> None:
        self.record_path.write_text("{", encoding="utf-8")
        with self.assertRaisesRegex(ProvenanceError, "record cannot be read"):
            validate_record(self.record_path, MANIFEST)

    def test_stale_lock_and_legacy_v1_states_fail_closed(self) -> None:
        manifest = self.copied_inventory()
        record = sample(load_inventory(manifest).revision)
        self.write(record)
        self.assertEqual(validate_record(self.record_path, manifest)["schema"], 2)
        lock = manifest.with_name("inventory.lock")
        lock.write_bytes(lock.read_bytes() + b"# stale\n")
        self.reject(record, "inventory.lock", manifest)
        self.reject(sample("a" * 64, schema=1), "does not match inventory")
        legacy = self.record_path.parent / "legacy" / "manifest.toml"
        legacy.parent.mkdir()
        legacy.write_text("schema = 1\n", encoding="utf-8")
        self.reject(sample("a" * 64, schema=1), "explicit compatibility", legacy)


if __name__ == "__main__":
    unittest.main()
