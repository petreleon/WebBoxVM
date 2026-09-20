#!/usr/bin/env python3
"""Hermetic hostile checks for the active F03 role-aware source gate."""

from __future__ import annotations

import copy
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from matrix_contract_v2 import MatrixError, validate_matrix
from profile_contract_v2 import REQUIREMENTS_PATH, SCOPE, ScopeError, validate
import role_aware_bindings as bindings
from role_aware_bindings import ROW_FIELDS, SOURCE_CONTRACT, SOURCE_LOCK, canonical, load_locked_source_contract

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[5]


class ProfileScopeV2Tests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path, Path]:
        temporary = tempfile.TemporaryDirectory(dir=ROOT)
        scope, requirements = Path(temporary.name) / "scope.json", Path(temporary.name) / "requirements.json"
        shutil.copyfile(SCOPE, scope)
        shutil.copyfile(REQUIREMENTS_PATH, requirements)
        return temporary, scope, requirements

    def change(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        path.write_text(json.dumps(value), encoding="utf-8")

    def matrix(self, profiles=("opengl-4.6-core", "gles-3.2")) -> dict[str, object]:
        contract = load_locked_source_contract()
        rows = []
        for profile in profiles:
            rows.append({"profile": profile, "requirement_kind": "command", "name": f"{profile}-example",
                         "mandatory": True, "source_role": "normative-root", "source_locator": "pinned-source",
                         "condition": "always", "owner_task": "G01", "test_source_role": "full-suite-root",
                         "status": "blocked", "evidence": "evidence.md#blocked", "blocker": "matrix-incomplete"})
        return {"schema": 2, "source_contract_sha256": contract["source_contract_sha256"],
                "inventory_lock_sha256": contract["inventory_lock_sha256"], "rows": rows}

    def write_matrix(self, value: dict[str, object]) -> Path:
        temporary = tempfile.TemporaryDirectory(dir=ROOT)
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        (root / "evidence.md").write_text("# synthetic evidence\n", encoding="utf-8")
        path = root / "matrix.json"
        path.write_text(json.dumps(value), encoding="utf-8")
        return path

    def test_locked_contract_leaves_only_matrix_work(self) -> None:
        self.assertEqual(validate(), ())
        scope = json.loads(SCOPE.read_text(encoding="utf-8"))
        self.assertTrue(all(item["status"] == "blocked" and item["blocker"] == "matrix-incomplete"
                            for item in scope["profiles"]))

    def test_cli_has_no_legacy_manifest_fallback(self) -> None:
        result = subprocess.run([sys.executable, str(HERE / "validate_profile_scope.py")], capture_output=True,
                                text=True, check=False)
        self.assertEqual(result.returncode, 0)
        self.assertIn("source gate is complete", result.stdout)
        legacy = subprocess.run([sys.executable, str(HERE / "validate_profile_scope.py"), "--manifest", "x"],
                                capture_output=True, text=True, check=False)
        self.assertEqual(legacy.returncode, 2)

    def test_scope_rejects_stale_headers_and_profile_promotion(self) -> None:
        edits = (
            lambda value: value.update(source_contract_sha256="0" * 64),
            lambda value: value.update(inventory_lock_sha256="0" * 64),
            lambda value: value["profiles"][0].update(status="supported", blocker=""),
        )
        for edit in edits:
            temporary, scope, requirements = self.copied()
            with temporary, self.subTest(edit=edit):
                self.change(scope, edit)
                with self.assertRaises(ScopeError):
                    validate(scope, requirements)

    def test_requirements_reject_alias_roles_order_and_identity_changes(self) -> None:
        edits = (
            lambda value: value["requirements"][5].update(record_id="vulkan-cts-mustpass"),
            lambda value: value["requirements"].pop(),
            lambda value: value["requirements"].append(copy.deepcopy(value["requirements"][0])),
            lambda value: value["requirements"].reverse(),
            lambda value: value["requirements"][0].update(profile="vulkan-1.4-core"),
            lambda value: value["requirements"][0].update(record_kind="webboxvm-transform"),
            lambda value: value["requirements"][0].update(scope="registry-metadata"),
            lambda value: value["requirements"][0].update(revision="0" * 40),
            lambda value: value["requirements"][0].update(sha256="0" * 64),
            lambda value: value["requirements"][1].update(unfiltered=False),
            lambda value: value["requirements"][1].update(selector_path="core-only-filter"),
        )
        for edit in edits:
            temporary, scope, requirements = self.copied()
            with temporary, self.subTest(edit=edit):
                self.change(requirements, edit)
                with self.assertRaisesRegex(ScopeError, "locked role|malformed profile"):
                    validate(scope, requirements)

    def test_raw_source_lock_and_cache_decoy_are_rejected_or_ignored(self) -> None:
        temporary = tempfile.TemporaryDirectory(dir=ROOT)
        with temporary:
            source, lock = Path(temporary.name) / "source_contract.json", Path(temporary.name) / "source_contract.lock"
            shutil.copyfile(SOURCE_CONTRACT, source)
            shutil.copyfile(SOURCE_LOCK, lock)
            source.write_bytes(source.read_bytes() + b"\n")
            with self.assertRaisesRegex(ScopeError, "raw bytes"):
                validate(source_contract=source, source_lock=lock)
        before = list(sys.path)
        self.assertEqual(validate(), ())
        self.assertEqual(sys.path, before)
        self.assertTrue({"webboxvm_source_builder", "inventory_layout"}.issubset(bindings._BARE_MODULES))
        code = ("import sys,types; names=('role_aware_source_lock','webboxvm_source_builder','inventory_layout'); "
                "saved={n:types.ModuleType(n) for n in names}; saved['role_aware_source_lock'].load_locked_contract="
                "lambda *a: (_ for _ in ()).throw(RuntimeError('decoy')); sys.modules.update(saved); "
                "import validate_profile_scope; validate_profile_scope.main(); "
                "assert all(sys.modules[n] is saved[n] for n in names)")
        result = subprocess.run([sys.executable, "-c", code], cwd=HERE, capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 0)

    def test_matrix_resolves_only_the_same_three_role_pairs(self) -> None:
        self.assertEqual(validate(matrix_path=self.write_matrix(self.matrix())), ())
        edits = (
            lambda value: value["rows"][0].update(source_role="full-suite-root"),
            lambda value: value["rows"][0].update(test_source_role="normative-root"),
            lambda value: value["rows"][0].update(source_id="vulkan-registry"),
            lambda value: value["rows"][0].update(status="supported", blocker=""),
        )
        for edit in edits:
            value = self.matrix()
            edit(value)
            with self.subTest(edit=edit), self.assertRaises(ScopeError):
                validate(matrix_path=self.write_matrix(value))

    def test_vulkan_matrix_rows_need_citations_and_reject_registry_locators(self) -> None:
        value = self.matrix(("vulkan-1.4-core",))
        with self.assertRaisesRegex(ScopeError, "citation map"):
            validate(matrix_path=self.write_matrix(value))
        value["rows"][0]["source_locator"] = "xml/vk.xml#feature[@name='VK_VERSION_1_4']"
        with self.assertRaisesRegex(ScopeError, "registry structural"):
            validate(matrix_path=self.write_matrix(value))

    def test_matrix_rejects_a_root_without_its_closure_or_claim_free_state(self) -> None:
        for mutate in (lambda value: value.update(closures=[]), lambda value: value.update(cts_executions=1)):
            contract = copy.deepcopy(load_locked_source_contract())
            mutate(contract)
            body = {key: item for key, item in contract.items() if key != "source_contract_sha256"}
            contract["source_contract_sha256"] = hashlib.sha256(canonical(body)).hexdigest()
            with self.subTest(mutate=mutate), self.assertRaisesRegex(MatrixError, "closure|promote"):
                validate_matrix(self.write_matrix(self.matrix()), contract,
                                {"opengl-4.6-core", "gles-3.2", "vulkan-1.4-core"})

    def test_matrix_rows_have_no_legacy_identity_fields(self) -> None:
        self.assertEqual(set(self.matrix()["rows"][0]), set(ROW_FIELDS))


if __name__ == "__main__":
    unittest.main()
