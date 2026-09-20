#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.1."""

from __future__ import annotations

import copy
import hashlib
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
PROBE = HERE / "gles_source_authority.py"


def load():
    spec = importlib.util.spec_from_file_location("f0331_test_authority", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


AUTH = load()


class GlesSourceAuthorityTests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / "boundary.json"
        shutil.copyfile(AUTH.BOUNDARY, target)
        return temporary, target

    def mutate(self, path: Path, edit, rehash: bool = False) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        if rehash:
            body = {key: item for key, item in value.items() if key != "boundary_sha256"}
            value["boundary_sha256"] = hashlib.sha256(AUTH.canonical(body)).hexdigest()
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_exact_roots_decisions_and_no_claim_state_are_sealed(self) -> None:
        value = AUTH.validate()
        self.assertEqual((value["normative_root"]["record_id"], value["full_suite_root"]["record_id"]),
                         ("gles-32-spec", "gles-cts-main"))
        self.assertEqual([item["decision"] for item in value["locator_classes"]],
                         ["derivable", "derivable", "requires-separate-admission",
                          "requires-separate-admission", "requires-separate-admission"])
        self.assertEqual(value["cts_executions"], 0)
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["states"]["profile_status"], value["states"]["blocker"]),
                         ("blocked", "matrix-incomplete"))

    def test_only_derivable_classes_consume_the_versioned_pdf_locator(self) -> None:
        locator = "gles32-pdf-v1:page=2;section=1.1"
        for identifier in ("command-state", "limit-format"):
            with self.subTest(identifier=identifier):
                result = AUTH.consume(identifier, locator)
                self.assertEqual(result["source"]["record_id"], "gles-32-spec")
        for identifier in ("shader", "precision", "extension", "essl-320-spec", "opengl-gles-registry",
                           "opengl-46-core-spec", "vulkan-registry"):
            with self.subTest(identifier=identifier), self.assertRaisesRegex(AUTH.BoundaryError, "admission|unavailable"):
                AUTH.consume(identifier, locator)
        for invalid in ("gles32-pdf-v1:page=0;section=1", "essl-320-spec:1", locator + ";extra=1"):
            with self.subTest(locator=invalid), self.assertRaisesRegex(AUTH.BoundaryError, "syntax"):
                AUTH.consume("command-state", invalid)

    def test_self_hash_stale_mixed_reordered_legacy_auxiliary_and_profile_spoofs_fail(self) -> None:
        edits = (
            ("self-hash", lambda value: value.update(profile="wrong"), False, "self hash"),
            ("stale", lambda value: value.update(source_contract_sha256="0" * 64), True, "exact active"),
            ("mixed", lambda value: value.update(normative_root=copy.deepcopy(value["full_suite_root"])), True, "exact active"),
            ("reordered", lambda value: value["locator_classes"].reverse(), True, "exact active"),
            ("legacy", lambda value: value["normative_root"].update(record_id="opengl-gles-registry"), True, "exact active"),
            ("auxiliary", lambda value: value["normative_root"].update(record_id="vulkan-registry"), True, "exact active"),
            ("profile", lambda value: value.update(profile="opengl-4.6-core"), True, "exact active"),
        )
        for name, edit, rehash, message in edits:
            temporary, path = self.copied()
            with temporary, self.subTest(name=name):
                self.mutate(path, edit, rehash)
                with self.assertRaisesRegex(AUTH.BoundaryError, message):
                    AUTH.validate(path)

    def test_stale_raw_f02_lock_fails_before_boundary_use(self) -> None:
        api = AUTH.bindings_api()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            contract, lock = root / "source_contract.json", root / "source_contract.lock"
            shutil.copyfile(api.SOURCE_CONTRACT, contract)
            shutil.copyfile(api.SOURCE_LOCK, lock)
            contract.write_bytes(contract.read_bytes() + b"\n")
            with self.assertRaisesRegex(AUTH.BoundaryError, "raw bytes"):
                AUTH.validate(AUTH.BOUNDARY, contract, lock)

    def test_ambient_module_cannot_preempt_the_fixed_private_import(self) -> None:
        code = "\n".join((
            "import importlib.util, sys, types",
            "decoy = types.ModuleType('role_aware_bindings')",
            "sys.modules['role_aware_bindings'] = decoy",
            f"spec = importlib.util.spec_from_file_location('authority', {str(PROBE)!r})",
            "module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
            "assert module.validate()['profile'] == 'gles-3.2'",
            "assert sys.modules['role_aware_bindings'] is decoy",
        ))
        result = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True,
                                env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(Path(AUTH.bindings_api().__file__).resolve(), AUTH.BINDINGS_PATH)

    def test_cli_reports_the_boundary_and_rejects_unresolved_consumption(self) -> None:
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        passed = subprocess.run([sys.executable, str(PROBE)], capture_output=True, text=True, env=environment)
        self.assertEqual(passed.returncode, 0, passed.stderr)
        self.assertEqual(passed.stdout.strip(), "PASS: 5 GLES classes; 2 derivable, 3 unavailable; matrix-incomplete")
        failed = subprocess.run([sys.executable, str(PROBE), "--consume-class", "precision", "--locator",
                                 "gles32-pdf-v1:page=2;section=1.1"], capture_output=True, text=True, env=environment)
        self.assertEqual(failed.returncode, 2)
        self.assertIn("requires separate source admission", failed.stderr)


if __name__ == "__main__":
    unittest.main()
