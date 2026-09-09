#!/usr/bin/env python3
"""Hermetic tests for the fixture-only Venus generated-output record."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
FIXTURE = HERE / "fixture"
MANIFEST = HERE.parents[2] / "01-input-inventory" / "manifest.toml"
GENERATOR = HERE / "venus_fixture_generator.py"
sys.path.insert(0, str(HERE))

from venus_record import ProvenanceError, validate_venus_record

REFERENCE = {
    "id": "webgpu-spec",
    "sha256": "85e732c1589c911ede74faccaefa439fb6222b96d86d352b9e44c7dc347d96a5",
    "license": "W3C Software and Document License (repo LICENSE.md)",
}
RUNTIME = {
    "id": "mesa-venus-device",
    "sha256": "68f06ca4ddc2d5a62bebaa469fc43dc5e66e640eec57d46f38172327d087a4c4",
    "license": "MIT (SPDX file notice)",
}


class VenusRecordTests(unittest.TestCase):
    def clone_fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory(prefix="webboxvm-venus-record-")
        root = Path(temporary.name) / "fixture"
        shutil.copytree(FIXTURE, root)
        return temporary, root

    def reject(self, change, message: str) -> None:
        temporary, root = self.clone_fixture()
        try:
            record_path = root / "venus-codec-record.json"
            record = json.loads(record_path.read_text(encoding="utf-8"))
            change(record, root)
            record_path.write_text(json.dumps(record, sort_keys=True) + "\n", encoding="utf-8")
            with self.assertRaisesRegex(ProvenanceError, message):
                validate_venus_record(record_path, MANIFEST)
        finally:
            temporary.cleanup()

    def test_generator_reproduces_the_checked_fixture_bytes(self) -> None:
        with tempfile.TemporaryDirectory(prefix="webboxvm-venus-generator-") as temporary:
            output = Path(temporary) / "marker.txt"
            result = subprocess.run(
                [sys.executable, str(GENERATOR), "--input-id", "venus-protocol-registry",
                 "--input-sha256", "d92839bc728fa9ad9a7decdc6b91df6fa1a0fb26cffae4009865f18a789e0535",
                 "--output", str(output)], text=True, capture_output=True, check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(output.read_bytes(), (FIXTURE / "venus-codec-record.txt").read_bytes())

    def test_valid_record_binds_the_real_artifact_and_manifest(self) -> None:
        record = validate_venus_record(FIXTURE / "venus-codec-record.json", MANIFEST)
        self.assertEqual(record["inputs"][0]["id"], "venus-protocol-registry")

    def test_wrong_generator_version_or_command_is_rejected(self) -> None:
        self.reject(lambda record, _: record["generator"].update(version="2"), "generator identity")
        self.reject(lambda record, _: record.update(command="not the fixture generator"), "command or artifact")

    def test_wrong_input_digest_and_unknown_identifier_are_rejected(self) -> None:
        self.reject(lambda record, _: record["inputs"][0].update(sha256="f" * 64), "stale sha256")
        self.reject(lambda record, _: record["inputs"][0].update(id="unknown-input"), "unknown")

    def test_reference_and_runtime_inputs_are_rejected(self) -> None:
        for substituted in (REFERENCE, RUNTIME):
            with self.subTest(identifier=substituted["id"]):
                self.reject(lambda record, _, value=substituted: record.update(inputs=[value]), "bind only")

    def test_wrong_output_hash_and_changed_artifact_are_rejected(self) -> None:
        self.reject(lambda record, _: record.update(output_sha256="f" * 64), "bytes do not match")
        self.reject(lambda _, root: (root / "venus-codec-record.txt").write_text("changed\n"), "bytes do not match")


if __name__ == "__main__":
    unittest.main()
