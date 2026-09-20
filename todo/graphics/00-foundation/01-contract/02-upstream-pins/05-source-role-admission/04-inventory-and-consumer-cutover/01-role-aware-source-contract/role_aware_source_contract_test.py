#!/usr/bin/env python3
"""Focused positive and hostile checks for the sealed role-aware source contract."""

from __future__ import annotations

import copy
import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import role_aware_source_contract as contract
import role_aware_source_evidence as evidence
import role_aware_source_lock as source_lock


def rehash(value: dict[str, object]) -> dict[str, object]:
    body = {key: item for key, item in value.items() if key != "source_contract_sha256"}
    value["source_contract_sha256"] = hashlib.sha256(contract.canonical(body)).hexdigest()
    return value


class RoleAwareSourceContractTests(unittest.TestCase):
    def test_locked_contract_has_exact_six_roles_and_no_qualification_claim(self):
        value = source_lock.load_locked_contract()
        self.assertEqual(contract.validate_contract(value), (
            "opengl-46-core-spec", "opengl-cts-gl46-main", "gles-32-spec", "gles-cts-main",
            "vulkan-14-spec", "vulkan-cts-default", "vulkan-registry", "vulkan-14-core-definition"))
        self.assertEqual([(item["profile"], item["role"], item["record_id"]) for item in value["bindings"]], [
            ("opengl-4.6-core", "normative-root", "opengl-46-core-spec"),
            ("opengl-4.6-core", "full-suite-root", "opengl-cts-gl46-main"),
            ("gles-3.2", "normative-root", "gles-32-spec"),
            ("gles-3.2", "full-suite-root", "gles-cts-main"),
            ("vulkan-1.4-core", "normative-root", "vulkan-14-spec"),
            ("vulkan-1.4-core", "full-suite-root", "vulkan-cts-default"),
        ])
        self.assertEqual(contract.binding_for(value, "vulkan-1.4-core", "full-suite-root")["unfiltered"], True)
        self.assertIn("not core-only", contract.binding_for(value, "vulkan-1.4-core", "full-suite-root")["selector_scope"])
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual(value["cts_executions"], 0)
        self.assertTrue(all(not item["discharges_required_role"] for item in value["auxiliary"][:2]))

    def test_rejects_alias_mixed_role_partial_closure_and_positive_state(self):
        cases = []
        alias = copy.deepcopy(contract.contract()); alias["bindings"][5]["record_id"] = "vulkan-cts-mustpass"; cases.append(rehash(alias))
        auxiliary = copy.deepcopy(contract.contract()); auxiliary["bindings"][4]["record_id"] = "vulkan-registry"; cases.append(rehash(auxiliary))
        partial = copy.deepcopy(contract.contract()); partial["closures"].pop(); cases.append(rehash(partial))
        reordered = copy.deepcopy(contract.contract()); reordered["records"][0], reordered["records"][1] = reordered["records"][1], reordered["records"][0]; cases.append(rehash(reordered))
        claimed = copy.deepcopy(contract.contract()); claimed["states"]["conformance"] = True; cases.append(rehash(claimed))
        for value in cases:
            with self.subTest(value=value), self.assertRaises(contract.SourceContractError):
                contract.validate_contract(value)

    def test_rejects_promoted_or_root_mismatched_vulkan_receipts(self):
        value = contract.contract()
        with tempfile.TemporaryDirectory() as temporary:
            full_path, shard_path = Path(temporary, "full.json"), Path(temporary, "shards.json")
            full = json.loads(evidence.VULKAN_RECEIPT.read_text(encoding="utf-8"))
            shards = json.loads(evidence.SHARD_RECEIPT.read_text(encoding="utf-8"))
            full["states"]["admitted"] = True
            full_path.write_text(json.dumps(full), encoding="utf-8")
            shard_path.write_text(json.dumps(shards), encoding="utf-8")
            with patch.object(evidence, "VULKAN_RECEIPT", full_path), patch.object(evidence, "SHARD_RECEIPT", shard_path):
                with self.assertRaises(evidence.EvidenceError):
                    evidence.validate_receipt_files(value["closures"], value["auxiliary"])

    def test_raw_lock_rejects_reformatted_or_tampered_seal(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            contract_path, lock_path = directory / "source_contract.json", directory / "source_contract.lock"
            payload = json.dumps(contract.seal(), sort_keys=True, indent=2).encode("utf-8")
            contract_path.write_bytes(payload)
            lock_path.write_bytes(source_lock.lock_text(contract_path, payload))
            self.assertEqual(source_lock.load_locked_contract(contract_path, lock_path)["records"][0]["id"], "opengl-46-core-spec")
            contract_path.write_bytes(payload + b"\n")
            with self.assertRaises(source_lock.SourceLockError):
                source_lock.load_locked_contract(contract_path, lock_path)


if __name__ == "__main__":
    unittest.main()
