#!/usr/bin/env python3
"""Focused hostile checks for the F03.3.2.4 unavailable-source ledger."""

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
import types
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
PROBE = HERE / "gles_unavailable_ledger.py"


def load():
    spec = importlib.util.spec_from_file_location("f03324_test_ledger", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


LEDGER = load()


class GlesUnavailableLedgerTests(unittest.TestCase):
    def copied(self, source: Path = LEDGER.LEDGER) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / source.name
        shutil.copyfile(source, target)
        return temporary, target

    def mutate(self, path: Path, edit, rehash: bool = False, field: str = "ledger_sha256") -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        if rehash:
            body = {key: item for key, item in value.items() if key != field}
            value[field] = hashlib.sha256(LEDGER.canonical(body)).hexdigest()
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_exact_unavailable_classes_identities_and_no_claim_state(self) -> None:
        value = LEDGER.validate()
        self.assertEqual((value["normative_root"]["record_id"], value["full_suite_root"]["record_id"]),
                         ("gles-32-spec", "gles-cts-main"))
        self.assertEqual(value["source_contract_sha256"], "d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3")
        self.assertEqual(value["inventory_lock_sha256"], "44a0f280e0ed854091a33e458122bd8cd9a0f9fc2e4c51a7a5be92bf48d8c6f4")
        self.assertEqual([item["id"] for item in value["unavailable_classes"]], list(LEDGER.UNAVAILABLE))
        self.assertTrue(all(item["blocker"] == "unadmitted-distinct-source" for item in value["unavailable_classes"]))
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["cts_executions"], value["matrix_row_count"], value["states"]["blocker"]),
                         (0, 0, "matrix-incomplete"))

    def test_essl_registry_desktop_and_lower_version_substitutes_cannot_enter(self) -> None:
        for identifier in LEDGER.UNAVAILABLE:
            for candidate in LEDGER.SUBSTITUTES:
                with self.subTest(identifier=identifier, candidate=candidate), self.assertRaisesRegex(
                        LEDGER.LedgerError, "separate F02 admission"):
                    LEDGER.reject_substitute(identifier, candidate)
            with self.subTest(identifier=identifier), self.assertRaisesRegex(LEDGER.LedgerError, "Matrix v2 row"):
                LEDGER.reject_matrix_row({"requirement_kind": identifier, "status": "blocked"})
            with self.subTest(identifier=identifier), self.assertRaisesRegex(LEDGER.LedgerError, "promoted"):
                LEDGER.reject_matrix_row({"requirement_kind": identifier, "status": "supported"})
        with self.assertRaisesRegex(LEDGER.LedgerError, "does not identify"):
            LEDGER.reject_matrix_row({"status": "blocked"})

    def test_self_hash_stale_mixed_reordered_promotion_and_cross_profile_fail(self) -> None:
        edits = (
            (lambda value: value.update(profile="wrong"), False, "self hash"),
            (lambda value: value.update(profile="opengl-4.6-core"), True, "exact F03.3.1"),
            (lambda value: value.update(source_contract_sha256="0" * 64), True, "exact F03.3.1"),
            (lambda value: value.update(normative_root=copy.deepcopy(value["full_suite_root"])), True, "exact F03.3.1"),
            (lambda value: value["unavailable_classes"].reverse(), True, "exact F03.3.1"),
            (lambda value: value["unavailable_classes"][0].update(availability="available", source_role="normative-root"),
             True, "exact F03.3.1"),
            (lambda value: value.update(matrix_row_count=1), True, "exact F03.3.1"),
        )
        for edit, rehash, message in edits:
            temporary, path = self.copied()
            with temporary, self.assertRaisesRegex(LEDGER.LedgerError, message):
                self.mutate(path, edit, rehash)
                LEDGER.validate(path)

    def test_stale_f02_lock_and_stale_f0331_boundary_fail_before_ledger_use(self) -> None:
        api = LEDGER.authority_api()
        bindings = api.bindings_api()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            contract, lock = root / "source_contract.json", root / "source_contract.lock"
            shutil.copyfile(bindings.SOURCE_CONTRACT, contract)
            shutil.copyfile(bindings.SOURCE_LOCK, lock)
            contract.write_bytes(contract.read_bytes() + b"\n")
            with self.assertRaisesRegex(LEDGER.LedgerError, "raw bytes"):
                LEDGER.validate(source_contract=contract, source_lock=lock)
        temporary, path = self.copied(api.BOUNDARY)
        with temporary, self.assertRaisesRegex(LEDGER.LedgerError, "exact active"):
            self.mutate(path, lambda value: value.update(source_contract_sha256="0" * 64), True,
                        "boundary_sha256")
            LEDGER.validate(source_boundary=path)

    def test_fixed_private_authority_and_cli_are_immune_to_an_ambient_alias(self) -> None:
        code = "\n".join((
            "import importlib.util,sys,types",
            "decoy=types.ModuleType('gles_source_authority'); sys.modules['gles_source_authority']=decoy",
            f"spec=importlib.util.spec_from_file_location('ledger',{str(PROBE)!r})",
            "module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
            "assert module.validate()['profile']=='gles-3.2'",
            "assert sys.modules['gles_source_authority'] is decoy",
        ))
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        result = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, env=environment)
        self.assertEqual(result.returncode, 0, result.stderr)
        passed = subprocess.run([sys.executable, str(PROBE)], capture_output=True, text=True, env=environment)
        self.assertEqual((passed.returncode, passed.stdout.strip()), (0, "PASS: 3 unavailable GLES classes; matrix-incomplete"))
        failed = subprocess.run([sys.executable, str(PROBE), "--reject-class", "precision", "--candidate", "gl.xml"],
                                capture_output=True, text=True, env=environment)
        self.assertEqual(failed.returncode, 2)
        self.assertIn("registry XML", failed.stderr)


if __name__ == "__main__":
    unittest.main()
