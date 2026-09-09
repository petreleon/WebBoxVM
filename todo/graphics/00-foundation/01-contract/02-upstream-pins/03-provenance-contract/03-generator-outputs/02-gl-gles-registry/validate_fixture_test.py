#!/usr/bin/env python3
"""Hermetic checks for the GL/GLES registry fixture provenance binding."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from validate_fixture import HERE, MANIFEST, validate_fixture
from provenance_record import ProvenanceError
from generate_fixture import write_fixture
from inventory_layout import load_inventory, render_v2_lock

SIDECAR = HERE / "fixture-output.provenance.json"
OUTPUT = HERE / "fixture-output.json"
GENERATOR = HERE / "generate_fixture.py"


def reference(identifier: str, manifest_path: Path = MANIFEST) -> dict[str, str]:
    entries = load_inventory(manifest_path, allow_v1=True).inputs
    entry = next(value for value in entries if value["id"] == identifier)
    return {"id": identifier, "sha256": entry["sha256"], "license": entry["license"]}


class GlGlesFixtureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(dir=HERE)
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

    def v2_manifest(self) -> tuple[Path, Path]:
        root = self.output_path.parent / "v2"
        root.mkdir()
        header, entries = MANIFEST.read_text(encoding="utf-8").split("[[inputs]]", 1)
        manifest = root / "manifest.toml"
        manifest.write_text(header.replace("schema = 1", "schema = 2", 1)
                            + 'input_files = ["inputs/part-0001.toml"]\n', encoding="utf-8")
        part = root / "inputs/part-0001.toml"
        part.parent.mkdir()
        part.write_text("[[inputs]]" + entries, encoding="utf-8")
        lock = root / "inventory.lock"
        lock.write_bytes(render_v2_lock(manifest))
        return manifest, lock

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

    def test_generator_accepts_a_v2_lock_and_rejects_a_stale_one(self) -> None:
        manifest, lock = self.v2_manifest()
        regenerated = self.output_path.with_name("v2-regenerated.json")
        write_fixture(manifest, regenerated)
        self.assertEqual(regenerated.read_bytes(), OUTPUT.read_bytes())
        record = self.record()
        record["schema"] = 2
        record["inventory_sha256"] = hashlib.sha256(lock.read_bytes()).hexdigest()
        del record["manifest_sha256"]
        self.write(record)
        self.assertEqual(validate_fixture(self.record_path, self.output_path, manifest)["schema"], 2)
        lock.write_bytes(lock.read_bytes() + b"# stale\n")
        with self.assertRaisesRegex(ValueError, "inventory.lock"):
            write_fixture(manifest, regenerated)

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
