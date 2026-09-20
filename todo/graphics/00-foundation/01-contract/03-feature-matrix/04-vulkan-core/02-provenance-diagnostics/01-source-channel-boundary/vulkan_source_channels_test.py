#!/usr/bin/env python3
"""Focused hostile checks for F03.4.2.1 source-channel admission."""

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
PROBE = HERE / "vulkan_source_channels.py"


def load():
    spec = importlib.util.spec_from_file_location("f03421_test_channels", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


AUTH = load()


class VulkanSourceChannelTests(unittest.TestCase):
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

    def test_exact_roles_metadata_registry_policy_and_no_claim_state(self) -> None:
        value = AUTH.validate()
        self.assertEqual((value["normative_root"]["binding"]["record_id"], value["full_suite_root"]["binding"]["record_id"]),
                         ("vulkan-14-spec", "vulkan-cts-default"))
        self.assertEqual(value["normative_root"]["identity"]["license"], "CC-BY-4.0 (vkspec.adoc SPDX-License-Identifier)")
        self.assertEqual(value["full_suite_root"]["identity"]["attribution"], "KhronosGroup/VK-GL-CTS repository; immutable upstream full-suite selector")
        self.assertEqual(value["registry"]["identity"]["scope"], "registry-metadata")
        self.assertEqual(value["registry"]["row_policy"], {"status": "blocked", "implementation_owner": None, "independent_test_plan": None})
        self.assertFalse(value["registry"]["matrix_ingress"])
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["cts_executions"], value["matrix_row_count"], value["states"]["blocker"]), (0, 0, "matrix-incomplete"))

    def test_docs_and_registry_cannot_enter_the_matrix_before_a_citation_map(self) -> None:
        with self.assertRaisesRegex(AUTH.BoundaryError, "citation map"):
            AUTH.matrix_ingress({"source_locator": "vkspec.adoc#foo"})
        with self.assertRaisesRegex(AUTH.BoundaryError, "registry structural"):
            AUTH.matrix_ingress({"source_locator": "xml/vk.xml#feature[@name='VK_VERSION_1_4']"})

    def test_raw_registry_rows_must_remain_unassigned_and_blocked(self) -> None:
        row = {key: None for key in AUTH.SOURCE.inventory_api().ROW_FIELDS}
        row.update(status="blocked", implementation_owner=None, independent_test_plan=None)
        self.assertEqual(AUTH.structural_rows([row]), 1)
        for field, replacement in (("implementation_owner", "F03.4.3"),
                                   ("independent_test_plan", "CTS"), ("status", "supported")):
            altered = copy.deepcopy(row)
            altered[field] = replacement
            with self.subTest(field=field), self.assertRaisesRegex(AUTH.BoundaryError, "owner, test plan, or promoted"):
                AUTH.structural_rows([altered])

    def test_self_hash_mixed_roles_promotions_and_docs_closure_fail(self) -> None:
        edits = (
            (lambda value: value.update(profile="wrong"), False, "self hash"),
            (lambda value: value["normative_root"]["binding"].update(record_id="vulkan-registry"), True, "exact role-aware"),
            (lambda value: value["full_suite_root"]["binding"].update(unfiltered=False), True, "exact role-aware"),
            (lambda value: value["normative_root"]["identity"].update(license="other"), True, "exact role-aware"),
            (lambda value: value.update(docs_include_closure="admitted"), True, "exact role-aware"),
            (lambda value: value["registry"].update(matrix_ingress=True), True, "exact role-aware"),
            (lambda value: value["registry"]["row_policy"].update(implementation_owner="F03.4.3"), True, "exact role-aware"),
        )
        for edit, rehash, message in edits:
            temporary, path = self.copied()
            with temporary, self.assertRaisesRegex(AUTH.BoundaryError, message):
                self.mutate(path, edit, rehash)
                AUTH.validate(path)

    def test_stale_lock_pair_and_ambient_alias_fail_or_are_ignored(self) -> None:
        api = AUTH.SOURCE.bindings_api()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            contract, lock = root / "source_contract.json", root / "source_contract.lock"
            shutil.copyfile(api.SOURCE_CONTRACT, contract)
            shutil.copyfile(api.SOURCE_LOCK, lock)
            contract.write_bytes(contract.read_bytes() + b"\n")
            with self.assertRaisesRegex(AUTH.BoundaryError, "raw bytes"):
                AUTH.validate(source_contract=contract, source_lock=lock)
        with self.assertRaisesRegex(AUTH.BoundaryError, "together"):
            AUTH.validate(source_contract=api.SOURCE_CONTRACT)
        code = "\n".join((
            "import importlib.util,sys,types",
            "decoy=types.ModuleType('role_aware_bindings'); sys.modules['role_aware_bindings']=decoy",
            f"spec=importlib.util.spec_from_file_location('channels',{str(PROBE)!r})",
            "module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
            "assert module.validate()['profile']=='vulkan-1.4-core'",
            "assert sys.modules['role_aware_bindings'] is decoy",
        ))
        result = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True,
                                env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_cli_and_fixed_registry_scaffold_are_revalidated(self) -> None:
        result = subprocess.run([sys.executable, str(PROBE)], capture_output=True, text=True,
                                env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "PASS: 3 sealed Vulkan channels; matrix-incomplete")
        self.assertEqual(AUTH.registry(AUTH.locked(AUTH.SOURCE.bindings_api(), None, None))["scaffold_row_count"], 0)


if __name__ == "__main__":
    unittest.main()
