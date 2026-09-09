#!/usr/bin/env python3
"""Hermetic tests for the blocked WebGPU generator-input boundary."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from webgpu_generator_boundary import (
    BoundaryError,
    audit_blocker,
    load_inventory,
    validate_webgpu_generator_input,
)
from inventory_layout import render_v2_lock

HERE = Path(__file__).resolve().parent
MANIFEST = HERE.parents[3] / "01-input-inventory" / "manifest.toml"


def v2_layout(root: Path, old: str = "", new: str = "") -> tuple[Path, Path]:
    root.mkdir()
    header, entries = MANIFEST.read_text(encoding="utf-8").split("[[inputs]]", 1)
    if old:
        entries = entries.replace(old, new, 1)
    manifest = root / "manifest.toml"
    manifest.write_text(header.replace("schema = 1", "schema = 2", 1)
                        + 'input_files = ["inputs/part-0001.toml"]\n', encoding="utf-8")
    part = root / "inputs/part-0001.toml"
    part.parent.mkdir()
    part.write_text("[[inputs]]" + entries, encoding="utf-8")
    lock = root / "inventory.lock"
    lock.write_bytes(render_v2_lock(manifest))
    return manifest, lock


class WebGpuBoundaryTests(unittest.TestCase):
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
        grammar = {"wgsl-grammar-candidate": {
            "source_family": "wgsl-grammar", "generated_code_role": "future WGSL grammar generator input",
        }}
        with self.assertRaisesRegex(BoundaryError, "WGSL"):
            validate_webgpu_generator_input(grammar, "wgsl-grammar-candidate")

    def test_a_future_designated_webgpu_input_invalidates_this_blocker(self) -> None:
        with tempfile.TemporaryDirectory(dir=HERE) as temporary:
            manifest, _ = v2_layout(
                Path(temporary) / "eligible",
                'generated_code_role = "host API semantic reference; no generated code"',
                'generated_code_role = "future WebGPU generator input"',
            )
            with self.assertRaisesRegex(BoundaryError, "unexpectedly cleared"):
                audit_blocker(manifest)

    def test_v2_inventory_rejects_a_stale_lock_before_a_blocker(self) -> None:
        with tempfile.TemporaryDirectory(dir=HERE) as temporary:
            manifest, lock = v2_layout(Path(temporary) / "v2")
            self.assertEqual(len(audit_blocker(manifest)), 2)
            lock.write_bytes(lock.read_bytes() + b"# stale\n")
            with self.assertRaisesRegex(BoundaryError, "inventory.lock"):
                audit_blocker(manifest)

    def test_missing_inventory_input_is_rejected(self) -> None:
        with self.assertRaisesRegex(BoundaryError, "not in the immutable inventory"):
            validate_webgpu_generator_input(load_inventory(MANIFEST), "missing-webgpu-input")

    def test_malformed_inventory_is_rejected_before_a_blocker_is_reported(self) -> None:
        with tempfile.TemporaryDirectory(dir=HERE) as temporary:
            malformed = Path(temporary) / "manifest.toml"
            malformed.write_text("schema = 2\ninputs = []\n", encoding="utf-8")
            with self.assertRaisesRegex(BoundaryError, "unexpected fields"):
                audit_blocker(malformed)


if __name__ == "__main__":
    unittest.main()
