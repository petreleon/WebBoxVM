#!/usr/bin/env python3
"""Hermetic tests for the blocked WebGPU generator-input boundary."""

from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path

from webgpu_generator_boundary import (
    BoundaryError,
    audit_blocker,
    load_inventory,
    validate_webgpu_generator_input,
)

HERE = Path(__file__).resolve().parent
MANIFEST = HERE.parents[3] / "01-input-inventory" / "manifest.toml"


def entry(identifier: str, family: str, role: str) -> str:
    return f'''[[inputs]]
id = "{identifier}"
source_family = "{family}"
immutable_url = "https://example.invalid/{identifier}"
revision = "{'a' * 40}"
sha256 = "{'b' * 64}"
bytes = 1
license = "fixture"
local_cache = "webboxvm-graphics/f02/{identifier}/{'b' * 64}.source"
generated_code_role = "{role}"
provenance = "https://example.invalid/{identifier}"
'''


class WebGpuBoundaryTests(unittest.TestCase):
    def temporary_manifest(self, extra: str) -> tempfile.TemporaryDirectory[str]:
        temporary = tempfile.TemporaryDirectory(prefix="webboxvm-webgpu-boundary-")
        target = Path(temporary.name) / "manifest.toml"
        shutil.copyfile(MANIFEST, target)
        with target.open("a", encoding="utf-8") as output:
            output.write("\n" + extra)
        return temporary

    def test_current_inventory_reports_the_blocker(self) -> None:
        reasons = audit_blocker(MANIFEST)
        self.assertEqual(len(reasons), 2)
        self.assertIn("webgpu-spec is reference-only", reasons[0])
        self.assertIn("wgsl-spec is WGSL", reasons[1])

    def test_current_webgpu_and_wgsl_documents_are_rejected(self) -> None:
        inventory = load_inventory(MANIFEST)
        with self.assertRaisesRegex(BoundaryError, "reference-only"):
            validate_webgpu_generator_input(inventory, "webgpu-spec")
        with self.assertRaisesRegex(BoundaryError, "WGSL"):
            validate_webgpu_generator_input(inventory, "wgsl-spec")

    def test_wgsl_grammar_candidate_is_not_a_webgpu_input(self) -> None:
        temporary = self.temporary_manifest(entry(
            "wgsl-grammar-candidate", "wgsl-grammar", "future WGSL grammar generator input",
        ))
        try:
            inventory = load_inventory(Path(temporary.name) / "manifest.toml")
            with self.assertRaisesRegex(BoundaryError, "WGSL"):
                validate_webgpu_generator_input(inventory, "wgsl-grammar-candidate")
        finally:
            temporary.cleanup()

    def test_a_future_designated_webgpu_input_invalidates_this_blocker(self) -> None:
        temporary = self.temporary_manifest(entry(
            "webgpu-generator-source", "webgpu", "future WebGPU generator input",
        ))
        try:
            with self.assertRaisesRegex(BoundaryError, "now has an eligible"):
                audit_blocker(Path(temporary.name) / "manifest.toml")
        finally:
            temporary.cleanup()

    def test_missing_inventory_input_is_rejected(self) -> None:
        with self.assertRaisesRegex(BoundaryError, "not in the immutable inventory"):
            validate_webgpu_generator_input(load_inventory(MANIFEST), "missing-webgpu-input")

    def test_malformed_inventory_is_rejected_before_a_blocker_is_reported(self) -> None:
        with tempfile.TemporaryDirectory(prefix="webboxvm-webgpu-boundary-") as temporary:
            malformed = Path(temporary) / "manifest.toml"
            malformed.write_text("schema = 2\ninputs = []\n", encoding="utf-8")
            with self.assertRaisesRegex(BoundaryError, "schema version"):
                audit_blocker(malformed)


if __name__ == "__main__":
    unittest.main()
