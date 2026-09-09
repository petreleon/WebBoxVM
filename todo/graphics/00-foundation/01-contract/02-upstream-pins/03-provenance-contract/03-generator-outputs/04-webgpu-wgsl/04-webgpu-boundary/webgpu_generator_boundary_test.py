#!/usr/bin/env python3
"""Hermetic tests for the reviewed WebGPU WebIDL-input boundary."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[8]
sys.path.insert(0, str(HERE.parents[3] / "01-input-inventory"))

from webgpu_generator_boundary import (
    BoundaryError,
    INPUT_ID,
    REVIEWED_WEBGPU_IDL,
    audit_boundary,
    load_inventory,
    validate_webgpu_generator_input,
)

MANIFEST = HERE.parents[3] / "01-input-inventory" / "manifest.toml"
PROBE = HERE / "webgpu_generator_boundary.py"
WGSL_RECORD = HERE.parent / "03-wgsl-generator-record"


def v2_layout(root: Path) -> tuple[Path, Path]:
    shutil.copytree(MANIFEST.parent, root)
    manifest = root / "manifest.toml"
    lock = root / "inventory.lock"
    return manifest, lock


class WebGpuBoundaryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.inventory = load_inventory(MANIFEST)

    def test_current_inventory_accepts_only_the_reviewed_webidl_input(self) -> None:
        self.assertEqual(audit_boundary(MANIFEST), REVIEWED_WEBGPU_IDL)

    def test_cli_reports_the_positive_boundary(self) -> None:
        result = subprocess.run(
            [sys.executable, str(PROBE), "--manifest", str(MANIFEST)],
            env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"),
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "PASS: reviewed WebGPU WebIDL generator input is valid\n")

    def test_another_task_cannot_preempt_the_marker_module(self) -> None:
        code = "\n".join((
            "import sys",
            f"sys.path.insert(0, {str(WGSL_RECORD)!r})",
            "import validate_fixture",
            f"sys.path.insert(0, {str(HERE)!r})",
            "import webgpu_generator_boundary as boundary",
            "assert boundary.INPUT_ID == 'webgpu-idl'",
            "assert boundary.REVIEWED_WEBGPU_IDL['id'] == 'webgpu-idl'",
        ))
        result = subprocess.run(
            [sys.executable, "-c", code],
            env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"),
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_real_substitutes_and_renamed_spoofs_are_rejected(self) -> None:
        for identifier in (
            "webgpu-spec", "wgsl-spec", "wgsl-grammar-syntax", "webgpu-cts-buffer-map",
            "vk-gl-cts-api-version",
        ):
            with self.subTest(identifier=identifier):
                with self.assertRaisesRegex(BoundaryError, "not the reviewed"):
                    validate_webgpu_generator_input(self.inventory, identifier)
                spoof = dict(self.inventory[identifier])
                spoof.update(
                    id=INPUT_ID,
                    source_family=REVIEWED_WEBGPU_IDL["source_family"],
                    generated_code_role=REVIEWED_WEBGPU_IDL["generated_code_role"],
                )
                with self.assertRaisesRegex(BoundaryError, "unexpected"):
                    validate_webgpu_generator_input({INPUT_ID: spoof}, INPUT_ID)

    def test_every_reviewed_identity_field_is_frozen(self) -> None:
        for field, wrong in (
            ("id", "wrong"), ("source_family", "webgpu"), ("immutable_url", "wrong"),
            ("revision", "wrong"), ("sha256", "f" * 64), ("bytes", 1), ("license", "wrong"),
            ("local_cache", "wrong"), ("generated_code_role", "wrong"), ("provenance", "wrong"),
        ):
            with self.subTest(field=field):
                candidate = dict(REVIEWED_WEBGPU_IDL, **{field: wrong})
                with self.assertRaisesRegex(BoundaryError, f"unexpected {field}"):
                    validate_webgpu_generator_input({INPUT_ID: candidate}, INPUT_ID)

    def test_native_c_spoof_cannot_pass_with_the_accepted_name_and_role(self) -> None:
        native = dict(
            REVIEWED_WEBGPU_IDL,
            immutable_url="https://example.invalid/webgpu.h",
            revision="native-c",
            sha256="a" * 64,
            bytes=1,
            license="native",
            local_cache="native/webgpu.h",
            provenance="https://example.invalid/native",
        )
        with self.assertRaisesRegex(BoundaryError, "unexpected"):
            validate_webgpu_generator_input({INPUT_ID: native}, INPUT_ID)

    def test_v2_inventory_rejects_a_stale_lock_before_acceptance(self) -> None:
        with tempfile.TemporaryDirectory(dir=ROOT) as temporary:
            manifest, lock = v2_layout(Path(temporary) / "v2")
            self.assertEqual(audit_boundary(manifest), REVIEWED_WEBGPU_IDL)
            lock.write_bytes(lock.read_bytes() + b"# stale\n")
            with self.assertRaisesRegex(BoundaryError, "inventory.lock"):
                audit_boundary(manifest)

    def test_missing_or_malformed_inventory_is_rejected(self) -> None:
        missing = dict(self.inventory)
        missing.pop(INPUT_ID)
        with self.assertRaisesRegex(BoundaryError, "not in the immutable inventory"):
            validate_webgpu_generator_input(missing, INPUT_ID)
        with tempfile.TemporaryDirectory(dir=ROOT) as temporary:
            malformed = Path(temporary) / "manifest.toml"
            malformed.write_text("schema = 2\ninputs = []\n", encoding="utf-8")
            with self.assertRaisesRegex(BoundaryError, "unexpected fields"):
                audit_boundary(malformed)

    def test_other_identifier_is_rejected_before_lookup(self) -> None:
        with self.assertRaisesRegex(BoundaryError, "not the reviewed"):
            validate_webgpu_generator_input(self.inventory, "missing-webgpu-input")


if __name__ == "__main__":
    unittest.main()
