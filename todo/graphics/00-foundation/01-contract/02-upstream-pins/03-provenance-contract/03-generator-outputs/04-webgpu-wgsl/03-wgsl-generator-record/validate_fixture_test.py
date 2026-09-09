#!/usr/bin/env python3
"""Hermetic checks for the WGSL grammar fixture provenance binding."""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[3] / "01-input-inventory"))
sys.path.insert(0, str(HERE.parents[2] / "01-provenance-record"))

from generate_fixture import INPUT as GRAMMAR_INPUT, grammar_input, write_fixture
from inventory_layout import load_inventory
from provenance_record import ProvenanceError
from validate_fixture import COMMAND, INPUT, MANIFEST, validate_fixture

SIDECAR = HERE / "fixture-output.provenance.json"
OUTPUT = HERE / "fixture-output.json"
GENERATOR = HERE / "generate_fixture.py"


def reference(identifier: str) -> dict[str, str]:
    entry = next(value for value in load_inventory(MANIFEST).inputs if value["id"] == identifier)
    return {"id": identifier, "sha256": entry["sha256"], "license": entry["license"]}


class WgslGrammarFixtureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(dir=HERE)
        root = Path(self.temporary.name)
        self.record_path, self.output_path = root / "record.json", root / "fixture.json"
        self.record_path.write_bytes(SIDECAR.read_bytes())
        self.output_path.write_bytes(OUTPUT.read_bytes())

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def record(self) -> dict[str, object]:
        return json.loads(self.record_path.read_text(encoding="utf-8"))

    def write(self, value: dict[str, object]) -> None:
        self.record_path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")

    def reject(self, value: dict[str, object], message: str, manifest: Path = MANIFEST) -> None:
        candidate = self.record_path.with_name("candidate.json")
        candidate.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
        with self.assertRaisesRegex(ProvenanceError, message):
            validate_fixture(candidate, self.output_path, manifest)

    def copied_inventory(self) -> tuple[Path, Path]:
        root = self.output_path.parent / "inventory"
        root.mkdir()
        shutil.copy2(MANIFEST, root / "manifest.toml")
        shutil.copytree(MANIFEST.parent / "inputs", root / "inputs")
        lock = root / "inventory.lock"
        shutil.copy2(MANIFEST.with_name("inventory.lock"), lock)
        return root / "manifest.toml", lock

    def test_fixture_binds_the_reviewed_grammar_record(self) -> None:
        record = validate_fixture(self.record_path, self.output_path)
        self.assertEqual(record["inputs"], [INPUT])

    def test_generator_reproduces_the_tracked_fixture_without_network(self) -> None:
        regenerated = self.output_path.with_name("regenerated.json")
        subprocess.run(
            [sys.executable, str(GENERATOR), "--manifest", str(MANIFEST), "--output", str(regenerated)],
            check=True, capture_output=True, text=True,
        )
        self.assertEqual(regenerated.read_bytes(), OUTPUT.read_bytes())

    def test_generator_requires_the_current_lock(self) -> None:
        manifest, lock = self.copied_inventory()
        regenerated = self.output_path.with_name("lock-regenerated.json")
        write_fixture(manifest, regenerated)
        self.assertEqual(regenerated.read_bytes(), OUTPUT.read_bytes())
        lock.write_bytes(lock.read_bytes() + b"# stale\n")
        with self.assertRaisesRegex(ValueError, "inventory.lock"):
            write_fixture(manifest, regenerated)

    def test_generator_rejects_reclassified_or_reroled_grammar_input(self) -> None:
        for field, value in (("source_family", "webgpu"), ("generated_code_role", "future WebGPU generator input")):
            with self.subTest(field=field):
                entry = dict(GRAMMAR_INPUT)
                entry[field] = value
                with patch("generate_fixture.load_inventory", return_value=SimpleNamespace(inputs=[entry])):
                    with self.assertRaisesRegex(ValueError, f"unexpected {field}"):
                        grammar_input(MANIFEST)

    def test_reference_and_cross_family_inputs_are_rejected(self) -> None:
        for identifier in ("wgsl-spec", "webgpu-spec", "opengl-gles-registry"):
            with self.subTest(identifier=identifier):
                record = self.record()
                record["inputs"] = [reference(identifier)]
                self.reject(record, "only wgsl-grammar-syntax")

    def test_wrong_generator_version_or_command_is_rejected(self) -> None:
        version = self.record()
        version["generator"]["version"] = "2"
        self.reject(version, "unexpected generator")
        command = self.record()
        command["command"] = COMMAND + " --wrong"
        self.reject(command, "unexpected generator command")

    def test_stale_input_digest_and_license_are_rejected(self) -> None:
        digest = self.record()
        digest["inputs"][0]["sha256"] = "f" * 64
        self.reject(digest, "stale sha256")
        license = self.record()
        license["inputs"][0]["license"] = "wrong"
        self.reject(license, "stale license")

    def test_stale_inventory_lock_is_rejected(self) -> None:
        record = self.record()
        record["inventory_sha256"] = "f" * 64
        self.reject(record, "stale inventory")

    def test_grammar_family_and_role_are_frozen(self) -> None:
        entry = dict(INPUT, source_family="webgpu", generated_code_role="future WebGPU generator input")
        with patch("validate_fixture.load_inventory", return_value=SimpleNamespace(inputs=[entry])):
            with self.assertRaisesRegex(ProvenanceError, "unexpected family or generator role"):
                validate_fixture(self.record_path, self.output_path)

    def test_changed_output_and_declared_hash_are_rejected(self) -> None:
        declared = self.record()
        declared["output_sha256"] = "f" * 64
        self.reject(declared, "output hash")
        self.output_path.write_bytes(self.output_path.read_bytes() + b" ")
        with self.assertRaisesRegex(ProvenanceError, "output hash"):
            validate_fixture(self.record_path, self.output_path)
        value = json.loads(self.output_path.read_text(encoding="utf-8"))
        value["limitation"] = "wrong"
        raw = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
        self.output_path.write_bytes(raw)
        record = self.record()
        record["output_sha256"] = hashlib.sha256(raw).hexdigest()
        self.write(record)
        with self.assertRaisesRegex(ProvenanceError, "accepted grammar identity and limitation"):
            validate_fixture(self.record_path, self.output_path)


if __name__ == "__main__":
    unittest.main()
