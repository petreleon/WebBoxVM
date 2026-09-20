#!/usr/bin/env python3
"""Hostile checks for the profile-neutral future-F05 source accessor."""

from __future__ import annotations

import copy
import hashlib
import json
import sys
import tempfile
import types
import unittest
from pathlib import Path

import f05_source_adapter as adapter


def rehash(value: dict[str, object]) -> dict[str, object]:
    body = {key: item for key, item in value.items() if key != "source_contract_sha256"}
    value["source_contract_sha256"] = hashlib.sha256(adapter.canonical(body)).hexdigest()
    return value


class F05SourceAdapterTests(unittest.TestCase):
    def contract(self) -> dict[str, object]:
        return adapter._load_lock_api().load_locked_contract(adapter.SOURCE_CONTRACT, adapter.SOURCE_LOCK)

    def test_accessor_exposes_the_complete_profile_neutral_contract(self) -> None:
        value = adapter.admitted_source_contract()
        self.assertEqual(value["kind"], "webboxvm-f05-admitted-source-contract")
        self.assertEqual(value["record_count"], 8)
        self.assertEqual([(item["profile"], item["role"]) for item in value["bindings"]], list(adapter.PAIRS))
        self.assertEqual(len(value["closures"]), 3)
        self.assertEqual(len(value["auxiliary"]), 4)
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual(value["cts_executions"], 0)
        self.assertTrue(value["states"]["source_contract_complete"])
        self.assertFalse(value["states"]["f05_profile_registration"])
        value["records"].pop()
        self.assertEqual(adapter.admitted_source_contract()["record_count"], 8)

    def test_rejects_partial_mixed_auxiliary_and_promoted_contracts(self) -> None:
        edits = (
            lambda value: value["bindings"].pop(),
            lambda value: value["bindings"].reverse(),
            lambda value: value["bindings"][5].update(record_id="vulkan-cts-mustpass"),
            lambda value: value["bindings"][4].update(record_id="vulkan-registry"),
            lambda value: value["closures"].pop(),
            lambda value: value["claims"].update(profile_support=True),
            lambda value: value.update(cts_executions=1),
            lambda value: value["states"].update(profile_status="supported", blocker=""),
        )
        for edit in edits:
            value = copy.deepcopy(self.contract())
            edit(value)
            with self.subTest(edit=edit), self.assertRaises(adapter.F05SourceAdapterError):
                adapter._descriptor(rehash(value))

    def test_raw_lock_and_symlink_contract_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            source, lock = directory / "source_contract.json", directory / "source_contract.lock"
            source.write_bytes(adapter.SOURCE_CONTRACT.read_bytes() + b"\n")
            lock.write_bytes(adapter.SOURCE_LOCK.read_bytes())
            with self.assertRaises(adapter.F05SourceAdapterError):
                adapter._load_from_paths(source, lock)
            link = directory / "source-link.json"
            link.symlink_to(adapter.SOURCE_CONTRACT)
            with self.assertRaises(adapter.F05SourceAdapterError):
                adapter._load_from_paths(link, adapter.SOURCE_LOCK)

    def test_ambient_decoys_and_import_path_are_restored(self) -> None:
        original_path = list(sys.path)
        decoys = {name: types.ModuleType(name) for name in ("webboxvm_source_builder", "inventory_layout")}
        original = {name: sys.modules.get(name) for name in decoys}
        try:
            sys.modules.update(decoys)
            self.assertEqual(adapter.admitted_source_contract()["record_count"], 8)
            self.assertEqual(sys.path, original_path)
            self.assertTrue(all(sys.modules[name] is module for name, module in decoys.items()))
        finally:
            for name, module in original.items():
                if module is None:
                    sys.modules.pop(name, None)
                else:
                    sys.modules[name] = module

    def test_public_accessor_accepts_no_profile_or_registration(self) -> None:
        with self.assertRaises(TypeError):
            adapter.admitted_source_contract("vulkan-1.4-core")  # type: ignore[call-arg]
        value = json.loads(json.dumps(adapter.admitted_source_contract()))
        self.assertNotIn("profile", value)
        self.assertNotIn("registration", value)


if __name__ == "__main__":
    unittest.main()
