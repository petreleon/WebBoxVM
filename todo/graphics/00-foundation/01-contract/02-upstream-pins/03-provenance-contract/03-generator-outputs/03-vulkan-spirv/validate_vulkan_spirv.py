#!/usr/bin/env python3
"""Hermetic F02.3.3.3 checks for fixture-only Vulkan and SPIR-V outputs."""

from __future__ import annotations

import copy
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[7]
MANIFEST = HERE.parents[2] / "01-input-inventory" / "manifest.toml"
sys.path[:0] = [str(HERE), str(HERE.parents[1] / "01-provenance-record")]

from fixture_generator import NAME, VERSION, render  # noqa: E402
from provenance_record import ProvenanceError, validate_record  # noqa: E402

SAMPLES = {
    "vulkan-registry": "vulkan-registry.provenance.json",
    "spirv-core-grammar": "spirv-core-grammar.provenance.json",
}
OTHER_INPUTS = [
    {
        "id": "spirv-core-grammar",
        "sha256": "db8581272b63d232268094a47b68d18a0464fc911e06004d57419924fe660ba4",
        "license": "MIT (grammar notice)",
    },
    {
        "id": "vk-gl-cts-api-version",
        "sha256": "875ca8c0d65dd8a1024c8e504fe433305151f6afda9e95b6f58c967441a6fd07",
        "license": "Apache-2.0 (file notice)",
    },
    {
        "id": "mesa-venus-device",
        "sha256": "68f06ca4ddc2d5a62bebaa469fc43dc5e66e640eec57d46f38172327d087a4c4",
        "license": "MIT (SPDX file notice)",
    },
]


class VulkanSpirvRecords(unittest.TestCase):
    def record(self, family: str) -> dict[str, object]:
        return json.loads((HERE / SAMPLES[family]).read_text(encoding="utf-8"))

    def validate(self, family: str, record: dict[str, object]) -> dict[str, object]:
        with tempfile.TemporaryDirectory() as temporary:
            record_path = Path(temporary) / "record.json"
            record_path.write_text(json.dumps(record, sort_keys=True), encoding="utf-8")
            accepted = validate_record(record_path, MANIFEST)
        if accepted["generator"] != {"name": NAME, "version": VERSION}:
            raise AssertionError("unexpected generator identity or version")
        if accepted["inputs"][0]["id"] != family:
            raise AssertionError("unexpected generator input ID")
        if accepted["command"] != f"python3 fixture_generator.py {family} fixture/{family}.fixture":
            raise AssertionError("unexpected generator command")
        artifact = ROOT / accepted["artifact_path"]
        if hashlib.sha256(artifact.read_bytes()).hexdigest() != accepted["output_sha256"]:
            raise AssertionError("output digest does not bind fixture bytes")
        if artifact.read_bytes() != render(family):
            raise AssertionError("fixture is not reproducible from its declared generator")
        return accepted

    def test_records_resolve_and_regenerate(self) -> None:
        for family in SAMPLES:
            with self.subTest(family=family):
                self.assertEqual(self.validate(family, self.record(family))["artifact_kind"], "generated")

    def test_generator_version_is_fixed(self) -> None:
        for family in SAMPLES:
            with self.subTest(family=family):
                record = self.record(family)
                record["generator"]["version"] = "different"
                with self.assertRaisesRegex(AssertionError, "generator identity"):
                    self.validate(family, record)

    def test_cross_family_cts_and_runtime_inputs_do_not_substitute(self) -> None:
        for candidate in OTHER_INPUTS:
            with self.subTest(candidate=candidate["id"]):
                record = self.record("vulkan-registry")
                record["inputs"] = [copy.deepcopy(candidate)]
                with self.assertRaisesRegex(AssertionError, "generator input ID"):
                    self.validate("vulkan-registry", record)

    def test_stale_input_digest_is_rejected_by_contract(self) -> None:
        record = self.record("vulkan-registry")
        record["inputs"][0]["sha256"] = "f" * 64
        with self.assertRaisesRegex(ProvenanceError, "stale sha256"):
            self.validate("vulkan-registry", record)

    def test_output_digest_must_bind_fixture_bytes(self) -> None:
        record = self.record("spirv-core-grammar")
        record["output_sha256"] = "f" * 64
        with self.assertRaisesRegex(AssertionError, "output digest"):
            self.validate("spirv-core-grammar", record)


if __name__ == "__main__":
    unittest.main()
