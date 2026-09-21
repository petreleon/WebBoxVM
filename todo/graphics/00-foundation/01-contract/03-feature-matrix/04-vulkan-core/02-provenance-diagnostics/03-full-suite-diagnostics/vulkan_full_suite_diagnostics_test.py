#!/usr/bin/env python3
"""Focused hostile checks for the root-wide VCTS diagnostic boundary."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "vulkan_full_suite_diagnostics.py"


def load():
    spec = importlib.util.spec_from_file_location("f03423_test_diagnostics", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


AUTH = load()
CACHE = {"mode": "sealed-external-selector-cache", "file_count": 9,
         "source_contract_sha256": "d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3",
         "selector_record_ids": ["opengl-46-core-spec", "opengl-cts-gl46-main", "gles-32-spec", "gles-cts-main",
                                 "vulkan-14-spec", "vulkan-cts-default", "vulkan-registry"],
         "release_proof_ids": ["opengl-cts-4681-license", "vulkan-cts-1462-license"]}


class FullSuiteDiagnosticTests(unittest.TestCase):
    def built(self) -> dict[str, object]:
        with patch.object(AUTH, "cache_fact", return_value=copy.deepcopy(CACHE)):
            return AUTH.build(Path("/private/tmp/f03423-unit"))

    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        path = Path(temporary.name) / "diagnostic.json"
        path.write_text(json.dumps(self.built()), encoding="utf-8")
        return temporary, path

    def checked(self, path: Path) -> dict[str, object]:
        with patch.object(AUTH, "cache_fact", return_value=copy.deepcopy(CACHE)):
            return AUTH.validate(path, Path("/private/tmp/f03423-unit"))

    def mutate(self, path: Path, edit, rehash: bool = False) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        if rehash:
            body = {key: item for key, item in value.items() if key != "diagnostic_sha256"}
            value["diagnostic_sha256"] = hashlib.sha256(AUTH.canonical(body)).hexdigest()
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_complete_ordered_taxonomy_is_diagnostic_only_and_claim_free(self) -> None:
        value = self.built()
        self.assertEqual((value["full_suite_root"]["binding"]["record_id"], value["ledger"]["member_count"]),
                         ("vulkan-cts-default", 98))
        self.assertEqual(value["taxonomy"]["category_counts"], AUTH.CATEGORIES)
        self.assertEqual(value["ledger"]["member_total_bytes"], 434669348)
        self.assertEqual(value["ledger"]["rows_sha256"], hashlib.sha256(AUTH.canonical(value["rows"])).hexdigest())
        self.assertEqual((value["rows"][0]["ordinal"], value["rows"][-1]["ordinal"]), (1, 98))
        self.assertTrue(all(item["relation"] == "root-wide-diagnostic-only" and item["implementation_owner"] is None
                            and item["independent_test_plan"] is None and item["coverage"] == "unassigned"
                            for item in value["rows"]))
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["cts_executions"], value["diagnostic_scope"]["matrix_row_count"]), (0, 0))

    def test_self_hash_root_taxonomy_order_owners_and_coverage_promotions_fail(self) -> None:
        edits = (
            (lambda value: value["diagnostic_scope"].update(matrix_row_count=1), False),
            (lambda value: value["full_suite_root"]["binding"].update(record_id="vulkan-cts-mustpass"), True),
            (lambda value: value["rows"].reverse(), True),
            (lambda value: value["rows"][0].update(category="core"), True),
            (lambda value: value["rows"][0].update(implementation_owner="F03.4.3"), True),
            (lambda value: value["rows"][0].update(independent_test_plan="CTS"), True),
            (lambda value: value["rows"][0].update(coverage="covered"), True),
            (lambda value: value["claims"].update(conformance=True), True),
        )
        for edit, rehash in edits:
            temporary, path = self.copied()
            with temporary, self.subTest(edit=edit), self.assertRaises(AUTH.DiagnosticError):
                self.mutate(path, edit, rehash)
                self.checked(path)

    def test_external_cache_summary_rejects_local_or_incomplete_inputs(self) -> None:
        for path in (Path("relative"), AUTH.source.REPO):
            with self.subTest(path=path), self.assertRaises(AUTH.DiagnosticError):
                AUTH.external_root(path)
        contract = AUTH.source.bindings_api().load_locked_source_contract()
        bad = copy.deepcopy(CACHE); bad["selector_record_ids"].pop()
        fake = types.SimpleNamespace(source_lock=types.SimpleNamespace(load_locked_contract=lambda: {}),
                                     verify_selector_cache=lambda *_: bad)
        with patch.object(AUTH.source, "cache_api", return_value=fake), self.assertRaises(AUTH.DiagnosticError):
            AUTH.cache_fact(Path("/private/tmp/f03423-unit"), contract)

    def test_fixed_private_loaders_ignore_ambient_aliases(self) -> None:
        code = "\n".join((
            "import importlib.util,sys,types",
            "names=('role_aware_bindings','vulkan_ledger_taxonomy','role_aware_source_cache')",
            "saved={n:types.ModuleType(n) for n in names}; sys.modules.update(saved)",
            f"sys.path.insert(0,{str(HERE)!r})",
            f"spec=importlib.util.spec_from_file_location('diagnostic',{str(PROBE)!r})",
            "module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
            "assert module.source.bindings_api().load_locked_source_contract()['cts_executions']==0",
            "assert module.source.ledger_api().build()['ledger']['member_count']==98",
            "assert all(sys.modules[n] is saved[n] for n in names)",
        ))
        result = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True,
                                env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_fixed_source_loader_ignores_an_ambient_sibling_decoy(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            decoy = Path(temporary) / "vulkan_full_suite_diagnostic_sources.py"
            decoy.write_text("raise RuntimeError('ambient decoy loaded')\n", encoding="utf-8")
            code = "\n".join((
                "import importlib.util,os,sys",
                f"sys.path.insert(0,{temporary!r}); os.chdir({temporary!r})",
                f"spec=importlib.util.spec_from_file_location('diagnostic',{str(PROBE)!r})",
                "module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
                f"assert module.source.__file__ == {str(HERE / 'vulkan_full_suite_diagnostic_sources.py')!r}",
                "assert module.source.canonical({'fixed':True}) == b'{\"fixed\":true}'",
            ))
            result = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True,
                                    env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_validation_requires_an_external_cache(self) -> None:
        temporary, path = self.copied()
        with temporary, self.assertRaisesRegex(AUTH.DiagnosticError, "external selector cache"):
            AUTH.validate(path)


if __name__ == "__main__":
    unittest.main()
