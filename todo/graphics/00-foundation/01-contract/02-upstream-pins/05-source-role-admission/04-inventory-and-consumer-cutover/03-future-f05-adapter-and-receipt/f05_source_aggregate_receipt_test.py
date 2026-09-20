#!/usr/bin/env python3
"""Hostile checks for the compact F05 aggregate no-claim receipt."""

from __future__ import annotations

import copy
import hashlib
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

import f05_selector_cache as selector
import f05_source_adapter as adapter
import f05_source_aggregate_receipt as receipt


def rehash(value: dict[str, object]) -> dict[str, object]:
    body = {key: item for key, item in value.items() if key != "receipt_sha256"}
    value["receipt_sha256"] = hashlib.sha256(receipt.canonical(body)).hexdigest()
    return value


class AggregateReceiptTests(unittest.TestCase):
    def admission(self) -> dict[str, object]:
        return adapter.admitted_source_contract()

    def cache(self) -> dict[str, object]:
        value = self.admission()
        return {"mode": "fresh-refresh", "file_count": 9,
                "source_contract_sha256": value["source_contract_sha256"],
                "selector_record_ids": ["opengl-46-core-spec", "opengl-cts-gl46-main", "gles-32-spec",
                                        "gles-cts-main", "vulkan-14-spec", "vulkan-cts-default", "vulkan-registry"],
                "release_proof_ids": ["opengl-cts-4681-license", "vulkan-cts-1462-license"]}

    def value(self) -> dict[str, object]:
        f03 = {"source_gate": "complete", "profile_status": "blocked", "blocker": "matrix-incomplete"}
        matrix = {"mode": "schema-boundary-test", "imported_rows": 0, "profile_status": "blocked",
                  "blocker": "matrix-incomplete"}
        return receipt.seal(receipt._body(self.admission(), self.cache(), f03, matrix))

    def test_receipt_is_self_hashed_and_stays_no_claim(self) -> None:
        self.assertEqual(receipt.validate_receipt(self.value())["cts_executions"], 0)
        edits = (
            lambda value: value.update(cts_executions=1),
            lambda value: value["claims"].update(conformance=True),
            lambda value: value["source_admission"]["bindings"].pop(),
            lambda value: value["fresh_selector_cache"]["selector_record_ids"].pop(),
            lambda value: value["matrix_role_resolution"].update(imported_rows=3),
            lambda value: value["f05_boundary"].update(profile_bound_registration=True),
        )
        for edit in edits:
            value = copy.deepcopy(self.value())
            edit(value)
            with self.subTest(edit=edit), self.assertRaises(receipt.AggregateReceiptError):
                receipt.validate_receipt(rehash(value))
        stale = self.value()
        stale["states"]["performance"] = True
        with self.assertRaises(receipt.AggregateReceiptError):
            receipt.validate_receipt(stale)

    def test_build_runs_f03_gate_and_schema_boundary_without_registration(self) -> None:
        cache = self.cache()
        with patch.object(receipt, "refresh_selector_cache", return_value=cache), \
             patch.object(receipt, "verify_selector_cache", return_value=cache):
            value = receipt.build_receipt(Path("/private/tmp/webboxvm-f05-unit"), 1.0)
        self.assertEqual(value["f03_gate"]["source_gate"], "complete")
        self.assertEqual(value["matrix_role_resolution"]["imported_rows"], 0)
        self.assertFalse(value["f05_boundary"]["profile_bound_registration"])

    def test_selector_cache_loader_ignores_decoys_and_restores_them(self) -> None:
        before = list(sys.path)
        decoys = {name: types.ModuleType(name) for name in ("webboxvm_source_builder", "inventory_layout")}
        original = {name: sys.modules.get(name) for name in decoys}
        try:
            sys.modules.update(decoys)
            self.assertTrue(hasattr(selector._cache_api(), "verify_selector_cache"))
            self.assertEqual(sys.path, before)
            self.assertTrue(all(sys.modules[name] is module for name, module in decoys.items()))
        finally:
            for name, module in original.items():
                if module is None:
                    sys.modules.pop(name, None)
                else:
                    sys.modules[name] = module

    def test_cache_root_must_be_external_and_nonsymlink(self) -> None:
        with self.assertRaises(selector.F05SelectorCacheError):
            selector.external_root(selector.REPO)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            link = root / "cache-link"
            link.symlink_to(root / "target")
            with self.assertRaises(selector.F05SelectorCacheError):
                selector.external_root(link)


if __name__ == "__main__":
    unittest.main()
