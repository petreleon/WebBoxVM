#!/usr/bin/env python3
"""Hermetic checks for the GL/GLES registry fixture provenance binding."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

from validate_fixture import HERE, MANIFEST, validate_fixture
from provenance_record import ProvenanceError

SIDECAR = HERE / "fixture-output.provenance.json"
OUTPUT = HERE / "fixture-output.json"
GENERATOR = HERE / "generate_fixture.py"


def reference(identifier: str) -> dict[str, str]:
    manifest = tomllib.loads(MANIFEST.read_text(encoding="utf-8"))
    entry = next(value for value in manifest["inputs"] if value["id"] == identifier)
    return {"id": identifier, "sha256": entry["sha256"], "license": entry["license"]}


class GlGlesFixtureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.record_path = root / "record.json"
        self.output_path = root / "fixture.json"
        self.record_path.write_bytes(SIDECAR.read_bytes())
        self.output_path.write_bytes(OUTPUT.read_bytes())

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def record(self) -> dict[str, object]:
        return json.loads(self.record_path.read_text(encoding="utf-8"))

    def write(self, value: dict[str, object]) -> None:
        self.record_path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")

    def reject(self, value: dict[str, object], message: str) -> None:
        candidate = self.record_path.with_name("candidate.json")
        candidate.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
        with self.assertRaisesRegex(ProvenanceError, message):
            validate_fixture(candidate, self.output_path)

    def test_generated_fixture_matches_the_reviewed_registry_record(self) -> None:
        record = validate_fixture(self.record_path, self.output_path)
        self.assertEqual(record["inputs"], [reference("opengl-gles-registry")])

    def test_generator_reproduces_the_tracked_fixture_without_network(self) -> None:
        regenerated = self.output_path.with_name("regenerated.json")
        subprocess.run(
            [sys.executable, str(GENERATOR), "--manifest", str(MANIFEST), "--output", str(regenerated)],
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertEqual(regenerated.read_bytes(), OUTPUT.read_bytes())

    def test_reference_specs_are_rejected_as_generator_inputs(self) -> None:
        for identifier in ("glsl-460-spec", "essl-320-spec"):
            with self.subTest(identifier=identifier):
                record = self.record()
                record["inputs"] = [reference(identifier)]
                self.reject(record, "only opengl-gles-registry")

    def test_wrong_generator_version_and_input_digest_are_rejected(self) -> None:
        wrong_version = self.record()
        wrong_version["generator"]["version"] = "2"
        self.reject(wrong_version, "unexpected generator")
        wrong_digest = self.record()
        wrong_digest["inputs"][0]["sha256"] = "f" * 64
        self.reject(wrong_digest, "stale sha256")

    def test_wrong_record_hash_and_changed_output_are_rejected(self) -> None:
        wrong_hash = self.record()
        wrong_hash["output_sha256"] = "f" * 64
        self.reject(wrong_hash, "output hash")
        self.output_path.write_bytes(self.output_path.read_bytes() + b" ")
        with self.assertRaisesRegex(ProvenanceError, "output hash"):
            validate_fixture(self.record_path, self.output_path)


if __name__ == "__main__":
    unittest.main()
