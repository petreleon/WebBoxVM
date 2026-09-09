#!/usr/bin/env python3
"""Hermetic checks for the metadata-only WebGPU WebIDL provenance marker."""

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
ROOT = HERE.parents[9]
sys.path.insert(0, str(HERE.parents[4] / "01-input-inventory"))
sys.path.insert(0, str(HERE.parents[3] / "01-provenance-record"))

from generate_fixture import SOURCE, reviewed_webidl as generated_webidl, write_fixture
from inventory_layout import load_inventory
from provenance_record import ProvenanceError
from validate_fixture import INPUT, MANIFEST, validate_fixture

SIDECAR = HERE / "fixture-output.provenance.json"
OUTPUT = HERE / "fixture-output.json"
GENERATOR = HERE / "generate_fixture.py"


def reference(identifier: str) -> dict[str, str]:
    if identifier == "native-c-webgpu-header":
        return {"id": identifier, "sha256": "a" * 64, "license": "native"}
    entry = next(value for value in load_inventory(MANIFEST).inputs if value["id"] == identifier)
    return {"id": identifier, "sha256": entry["sha256"], "license": entry["license"]}


class WebIdlFixtureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(dir=ROOT)
        root = Path(self.temporary.name)
        self.record_path, self.output_path = root / "record.json", root / "fixture.json"
        self.record_path.write_bytes(SIDECAR.read_bytes())
        self.original_output = OUTPUT.read_bytes()
        self.output_path.write_bytes(self.original_output)

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

    def test_fixture_binds_only_the_reviewed_webidl_record(self) -> None:
        self.assertEqual(validate_fixture(self.record_path, self.output_path)["inputs"], [INPUT])

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

    def test_source_identity_is_frozen_for_generator_and_validator(self) -> None:
        for field, wrong in (
            ("id", "webgpu-spec"), ("source_family", "webgpu"), ("immutable_url", "wrong"),
            ("revision", "wrong"), ("sha256", "f" * 64), ("bytes", 1), ("license", "wrong"),
            ("local_cache", "wrong"), ("generated_code_role", "wrong"), ("provenance", "wrong"),
        ):
            with self.subTest(field=field):
                entry = dict(SOURCE, **{field: wrong})
                message = "exactly one" if field == "id" else f"unexpected {field}"
                with patch("generate_fixture.load_inventory", return_value=SimpleNamespace(inputs=[entry])):
                    with self.assertRaisesRegex(ValueError, message):
                        generated_webidl(MANIFEST)
                with patch("validate_fixture.load_inventory", return_value=SimpleNamespace(inputs=[entry])):
                    with self.assertRaisesRegex(ProvenanceError, message):
                        validate_fixture(self.record_path, self.output_path)

    def test_semantic_wgsl_cts_and_native_c_substitutes_are_rejected(self) -> None:
        for identifier in (
            "webgpu-spec", "wgsl-spec", "wgsl-grammar-syntax", "webgpu-cts-buffer-map",
            "native-c-webgpu-header",
        ):
            with self.subTest(identifier=identifier):
                record = self.record()
                record["inputs"] = [reference(identifier)]
                self.reject(record, "unknown" if identifier.startswith("native-c") else "only webgpu-idl")

    def test_wrong_generator_command_and_stale_identity_are_rejected(self) -> None:
        version = self.record()
        version["generator"]["version"] = "2"
        self.reject(version, "unexpected generator")
        command = self.record()
        command["command"] += " --wrong"
        self.reject(command, "unexpected generator command")
        digest = self.record()
        digest["inputs"][0]["sha256"] = "f" * 64
        self.reject(digest, "stale sha256")
        license = self.record()
        license["inputs"][0]["license"] = "wrong"
        self.reject(license, "stale license")
        stale = self.record()
        stale["inventory_sha256"] = "f" * 64
        self.reject(stale, "stale inventory")

    def test_changed_bytes_and_scope_are_rejected_even_with_a_new_hash(self) -> None:
        declared = self.record()
        declared["output_sha256"] = "f" * 64
        self.reject(declared, "output hash")
        self.output_path.write_bytes(self.original_output + b" ")
        with self.assertRaisesRegex(ProvenanceError, "output hash"):
            validate_fixture(self.record_path, self.output_path)
        for field, value in (("limitation", "wrong"), ("idl", ["WebGPU"])):
            with self.subTest(field=field):
                marker = json.loads(self.original_output.decode("utf-8"))
                marker[field] = value
                raw = (json.dumps(marker, indent=2, sort_keys=True) + "\n").encode("utf-8")
                self.output_path.write_bytes(raw)
                record = self.record()
                record["output_sha256"] = hashlib.sha256(raw).hexdigest()
                self.write(record)
                with self.assertRaisesRegex(ProvenanceError, "fixture output"):
                    validate_fixture(self.record_path, self.output_path)


if __name__ == "__main__":
    unittest.main()
