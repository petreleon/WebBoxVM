#!/usr/bin/env python3
"""Hostile admission checks for F03.2.3.2; no source cache or GPU is used."""

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
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "opengl_unadmitted_ledger.py"
spec = importlib.util.spec_from_file_location("f03232_test_ledger", PROBE)
LEDGER = importlib.util.module_from_spec(spec)
spec.loader.exec_module(LEDGER)


class OpenGLUnadmittedLedgerTests(unittest.TestCase):
    def mutate(self, source, edit, rehash=True, field="ledger_sha256"):
        value = json.loads(source.read_text(encoding="utf-8"))
        edit(value)
        if rehash:
            body = {key: item for key, item in value.items() if key != field}
            value[field] = hashlib.sha256(LEDGER.canonical(body)).hexdigest()
        return json.dumps(value)

    def test_exact_authority_and_no_semantic_facts_or_claims(self):
        value = LEDGER.validate()
        self.assertEqual(value["source_contract_sha256"],
                         "d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3")
        self.assertEqual(value["inventory_lock_sha256"],
                         "44a0f280e0ed854091a33e458122bd8cd9a0f9fc2e4c51a7a5be92bf48d8c6f4")
        self.assertEqual(value["normative_root"]["record_id"], "opengl-46-core-spec")
        self.assertEqual(value["full_suite_root"]["record_id"], "opengl-cts-gl46-main")
        self.assertEqual([item["id"] for item in value["unadmitted_classes"]], ["shader", "extension"])
        self.assertTrue(all(item["source_role"] is None for item in value["unadmitted_classes"]))
        self.assertTrue(all(item["blocker"] == "unadmitted-distinct-source" for item in value["unadmitted_classes"]))
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual([value[key] for key in ("matrix_row_count", "semantic_fact_count", "cts_executions")], [0, 0, 0])
        self.assertEqual(value["states"]["blocker"], "matrix-incomplete")

    def test_all_replacements_aliases_and_matrix_statuses_rejected(self):
        for identifier in ("shader", "extension"):
            for candidate in (*LEDGER.SUBSTITUTES, "GLSL", "unknown", None, {}, "gl.xml#GL_VERSION_4_6"):
                with self.subTest(identifier=identifier, candidate=candidate), self.assertRaisesRegex(
                        LEDGER.LedgerError, "separate F02 admission"):
                    LEDGER.reject_substitute(identifier, candidate)
            for status in ("blocked", "supported", "optional", "excluded", None):
                with self.subTest(status=status), self.assertRaisesRegex(LEDGER.LedgerError, "cannot"):
                    LEDGER.reject_matrix_row({"profile": "opengl-4.6-core", "requirement_kind": identifier,
                                              "status": status, "owner": "fake", "test": "fake"})
        for identifier in ("Shader", "glsl", "precision", "limit-format", "registry-metadata"):
            with self.subTest(alias=identifier), self.assertRaisesRegex(LEDGER.LedgerError, "unknown or admitted"):
                LEDGER.unadmitted_class(identifier)
        for row in (None, {}, {"requirement_kind": "shader", "profile": "gles-3.2"}):
            with self.subTest(row=row), self.assertRaises(LEDGER.LedgerError):
                LEDGER.reject_matrix_row(row)

    def test_stale_rehashed_mixed_reordered_promoted_and_wrong_types_rejected(self):
        edits = (
            lambda value: value.update(profile="gles-3.2"),
            lambda value: value.update(source_contract_sha256="0" * 64),
            lambda value: value.update(inventory_lock_sha256="0" * 64),
            lambda value: value.update(normative_root=copy.deepcopy(value["full_suite_root"])),
            lambda value: value["unadmitted_classes"].reverse(),
            lambda value: value["unadmitted_classes"][0].update(availability="available", source_role="normative-root"),
            lambda value: value["unadmitted_classes"][1].update(id="GL_ARB_fake"),
            lambda value: value["claims"].update(api_support=True),
            lambda value: value["claims"].update(api_support=0),
            lambda value: value.update(matrix_row_count=1),
            lambda value: value.update(semantic_fact_count=1),
            lambda value: value.update(cts_executions=False),
            lambda value: value.update(schema=True),
        )
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "ledger.json"
            for edit in edits:
                for rehash in (False, True):
                    path.write_text(self.mutate(LEDGER.LEDGER, edit, rehash), encoding="utf-8")
                    with self.subTest(edit=edit, rehash=rehash), self.assertRaises(LEDGER.LedgerError):
                        LEDGER.validate(path)
            for text in ('{"schema":1,"schema":1}', '[]', '{"n":NaN}', '{bad json'):
                path.write_text(text, encoding="utf-8")
                with self.subTest(text=text), self.assertRaises(LEDGER.LedgerError):
                    LEDGER.validate(path)

    def test_stale_contract_lock_and_authority_fail_before_use(self):
        api = LEDGER.authority_api()
        bindings = api.bindings_api()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            contract, lock, boundary = (root / name for name in ("contract.json", "lock.json", "authority.json"))
            for corrupt in ("contract", "lock"):
                shutil.copyfile(bindings.SOURCE_CONTRACT, contract)
                shutil.copyfile(bindings.SOURCE_LOCK, lock)
                path = contract if corrupt == "contract" else lock
                path.write_bytes(b"stale" + path.read_bytes())
                with self.subTest(corrupt=corrupt), self.assertRaises(LEDGER.LedgerError):
                    LEDGER.validate(source_contract=contract, source_lock=lock)
            for edit in (lambda value: value.update(profile="gles-3.2"),
                         lambda value: value["locator_classes"][-1].update(availability="available")):
                boundary.write_text(self.mutate(api.BOUNDARY, edit, field="boundary_sha256"), encoding="utf-8")
                with self.assertRaises(LEDGER.LedgerError):
                    LEDGER.validate(source_boundary=boundary)
            with self.assertRaisesRegex(LEDGER.LedgerError, "together"):
                LEDGER.validate(source_contract=contract)

    def test_fixed_private_loader_ignores_and_restores_ambient_aliases(self):
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "opengl_source_authority.py").write_text("raise RuntimeError('ambient loaded')\n", encoding="utf-8")
            code = "\n".join((
                "import importlib.util,sys,types",
                "aliases=['opengl_source_authority','f03232_opengl_source_authority']",
                "decoy=types.ModuleType('decoy')",
                "for name in aliases: sys.modules[name]=decoy",
                f"spec=importlib.util.spec_from_file_location('ledger',{str(PROBE)!r})",
                "module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
                "assert module.validate()['profile']=='opengl-4.6-core'",
                "assert all(sys.modules[name] is decoy for name in aliases)",
            ))
            result = subprocess.run([sys.executable, "-c", code], cwd=root, env=environment,
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
        passed = subprocess.run([sys.executable, str(PROBE)], env=environment, capture_output=True, text=True)
        self.assertEqual((passed.returncode, passed.stdout.strip()),
                         (0, "PASS: 2 unadmitted OpenGL classes; matrix-incomplete"))
        failed = subprocess.run([sys.executable, str(PROBE), "--reject-class", "shader", "--candidate", "gl.xml"],
                                env=environment, capture_output=True, text=True)
        self.assertEqual(failed.returncode, 2)
        self.assertIn("separate F02 admission", failed.stderr)

    def test_missing_or_symlink_authority_fails_and_returns_are_isolated(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            link = root / "authority.py"
            link.symlink_to(LEDGER.AUTHORITY)
            for source in (root / "missing.py", link):
                with patch.object(LEDGER, "AUTHORITY", source), self.assertRaisesRegex(
                        LEDGER.LedgerError, "unavailable"):
                    LEDGER.validate()
        first = LEDGER.validate()
        first["unadmitted_classes"][0]["availability"] = "available"
        first["claims"]["api_support"] = True
        self.assertEqual(LEDGER.unadmitted_class("shader")["availability"], "unavailable")
        self.assertFalse(LEDGER.validate()["claims"]["api_support"])


if __name__ == "__main__":
    unittest.main()
