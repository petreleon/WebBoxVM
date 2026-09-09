#!/usr/bin/env python3
"""Hermetic positive and hostile checks for the OpenGL input audit."""

from __future__ import annotations

import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
PARENT = HERE.parent
AUDIT = HERE / "candidates.json"


def contract():
    spec = importlib.util.spec_from_file_location("f024_candidate_contract", PARENT / "candidate_contract.py")
    if spec is None or spec.loader is None:
        raise RuntimeError("candidate contract cannot load")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


CONTRACT = contract()


class OpenGlAuditTests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / "candidates.json"
        shutil.copyfile(AUDIT, target)
        return temporary, target

    def change(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_reviewed_candidates_are_complete_single_sources(self) -> None:
        self.assertEqual(CONTRACT.validate(AUDIT, "opengl-4.6-core"), ("accepted", "accepted"))

    def test_mutable_or_mismatched_raw_revision_is_rejected(self) -> None:
        temporary, path = self.copied()
        with temporary:
            self.change(path, lambda value: value["candidates"][0]["entry"].update(revision="a" * 40))
            with self.assertRaisesRegex(CONTRACT.AuditError, "pinned raw source"):
                CONTRACT.validate(path, "opengl-4.6-core")

    def test_existing_source_family_cannot_be_reused(self) -> None:
        temporary, path = self.copied()
        with temporary:
            self.change(path, lambda value: value["candidates"][0]["entry"].update(source_family="gl-gles-registry"))
            with self.assertRaisesRegex(CONTRACT.AuditError, "existing or duplicate"):
                CONTRACT.validate(path, "opengl-4.6-core")

    def test_exact_source_identity_and_selector_cannot_be_substituted(self) -> None:
        temporary, path = self.copied()
        with temporary:
            def substitute(value):
                entry = value["candidates"][0]["entry"]
                entry.update(source_family="alternate-source", bytes=2774722,
                             immutable_url="https://raw.githubusercontent.com/KhronosGroup/OpenGL-Registry/1cdd228e34966dd6b95bd203e9f84faba0f371a1/xml/gl.xml",
                             sha256="b9ca2cfa5c676e901c20d34af3407f1687cde0f1336a5ff7a8974d04c7494ad3",
                             local_cache="webboxvm-graphics/f02/opengl-46-core-spec/b9ca2cfa5c676e901c20d34af3407f1687cde0f1336a5ff7a8974d04c7494ad3.source")
                value["candidates"][0].update(selector="xml/gl.xml")
            self.change(path, substitute)
            with self.assertRaisesRegex(CONTRACT.AuditError, "reviewed exact"):
                CONTRACT.validate(path, "opengl-4.6-core")
        temporary, path = self.copied()
        with temporary:
            self.change(path, lambda value: value["candidates"][0]["entry"].update(source_family="arbitrary-new"))
            with self.assertRaisesRegex(CONTRACT.AuditError, "reviewed exact"):
                CONTRACT.validate(path, "opengl-4.6-core")
        temporary, path = self.copied()
        with temporary:
            self.change(path, lambda value: value["candidates"][1].update(selector="gl46-main.txt"))
            with self.assertRaisesRegex(CONTRACT.AuditError, "reviewed exact selector"):
                CONTRACT.validate(path, "opengl-4.6-core")

    def test_zero_case_manifest_is_rejected(self) -> None:
        temporary, path = self.copied()
        with temporary:
            self.change(path, lambda value: value["candidates"][1].update(selector_case_count=0))
            with self.assertRaisesRegex(CONTRACT.AuditError, "selector count"):
                CONTRACT.validate(path, "opengl-4.6-core")

    def test_exact_metadata_cannot_be_rewritten(self) -> None:
        for field, replacement in (("license", "false license"), ("generated_code_role", "false role"),
                                   ("provenance", "https://example.invalid/false")):
            with self.subTest(field=field), tempfile.TemporaryDirectory() as temporary:
                path = Path(temporary) / "candidates.json"
                shutil.copyfile(AUDIT, path)
                self.change(path, lambda value: value["candidates"][0]["entry"].update({field: replacement}))
                with self.assertRaisesRegex(CONTRACT.AuditError, "reviewed exact"):
                    CONTRACT.validate(path, "opengl-4.6-core")

    def test_accepted_compound_or_stale_audit_is_rejected(self) -> None:
        temporary, path = self.copied()
        with temporary:
            self.change(path, lambda value: value["candidates"][1].update(coverage="compound-unadmitted", admission_blocker="missing"))
            with self.assertRaisesRegex(CONTRACT.AuditError, "reviewed exact"):
                CONTRACT.validate(path, "opengl-4.6-core")
            self.change(path, lambda value: value.update(inventory_sha256="0" * 64))
            with self.assertRaisesRegex(CONTRACT.AuditError, "stale inventory"):
                CONTRACT.validate(path, "opengl-4.6-core")

    def test_drifted_canonical_f03_requirement_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            requirements = Path(temporary) / "requirements.json"
            shutil.copyfile(CONTRACT.PROFILE.REQUIREMENTS_PATH, requirements)
            self.change(requirements, lambda value: value["requirements"][0].update(required_input_id="wrong"))
            with self.assertRaisesRegex(CONTRACT.AuditError, "canonical F03 requirements"):
                CONTRACT.requirements("opengl-4.6-core", requirements)

    def test_exact_source_loader_restores_a_cached_inventory_layout_decoy(self) -> None:
        script = (
            "import sys,types; decoy=types.ModuleType('inventory_layout'); sys.modules['inventory_layout']=decoy; "
            f"sys.path.insert(0,{str(PARENT)!r}); import candidate_contract; "
            f"from pathlib import Path; candidate_contract.validate(Path({str(AUDIT)!r}),'opengl-4.6-core'); "
            "assert sys.modules['inventory_layout'] is decoy"
        )
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        result = subprocess.run([sys.executable, "-c", script], capture_output=True, text=True, env=environment)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_cli_reports_the_audit_decision(self) -> None:
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        result = subprocess.run([sys.executable, str(PARENT / "candidate_contract.py"), str(AUDIT), "opengl-4.6-core"],
                                capture_output=True, text=True, env=environment)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "AUDIT: opengl-4.6-core accepted, accepted")


if __name__ == "__main__":
    unittest.main()
