#!/usr/bin/env python3
"""Hostile role-boundary and selector-cache tests for F03.4.1 v2."""

from __future__ import annotations

import copy
import hashlib
import sys
import tempfile
import types
import unittest
from pathlib import Path

import registry_inventory_cache as cache
import registry_inventory_source as source


class SourceBoundaryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.contract = source._binding_api().load_locked_source_contract()

    def with_contract(self, value: dict[str, object]):
        original = source._binding_api
        source._binding_api = lambda: types.SimpleNamespace(load_locked_source_contract=lambda: value)
        return original

    def test_identity_is_exact_auxiliary_metadata_not_a_role(self) -> None:
        identity = source.registry_identity()
        self.assertEqual(identity["source_id"], "vulkan-registry")
        self.assertEqual(identity["scope"], "registry-metadata")
        self.assertEqual(identity["license"], source.LICENSE)
        self.assertEqual(identity["bytes"], 3309653)
        self.assertEqual(identity["version_marker"], "VK_VERSION_1_4")
        self.assertFalse(identity["discharges_required_role"])
        self.assertEqual(identity["cts_executions"], 0)
        self.assertTrue(all(item is False for item in identity["claims"].values()))

    def test_rejects_auxiliary_promotion_missing_evidence_or_binding_substitution(self) -> None:
        cases = []
        scope = copy.deepcopy(self.contract)
        next(item for item in scope["records"] if item["id"] == source.SOURCE_ID)["scope"] = "normative-source"
        cases.append(scope)
        role = copy.deepcopy(self.contract)
        next(item for item in role["auxiliary"] if item["id"] == source.SOURCE_ID)["discharges_required_role"] = True
        cases.append(role)
        bound = copy.deepcopy(self.contract)
        bound["bindings"].append({"profile": "vulkan-1.4-core", "role": "normative-root", "record_id": source.SOURCE_ID})
        cases.append(bound)
        missing = copy.deepcopy(self.contract)
        missing["auxiliary"] = [item for item in missing["auxiliary"] if item["id"] != source.SOURCE_ID]
        cases.append(missing)
        for value in cases:
            original = self.with_contract(value)
            try:
                with self.assertRaises(source.RegistryError):
                    source.registry_identity()
            finally:
                source._binding_api = original

    def test_rejects_promoted_contract_claim_and_ignores_ambient_binding_decoy(self) -> None:
        promoted = copy.deepcopy(self.contract)
        promoted["claims"]["profile_support"] = True
        original = self.with_contract(promoted)
        try:
            with self.assertRaises(source.RegistryError):
                source.registry_identity()
        finally:
            source._binding_api = original
        decoy, before = types.ModuleType("role_aware_bindings"), list(sys.path)
        prior = sys.modules.get("role_aware_bindings")
        sys.modules["role_aware_bindings"] = decoy
        try:
            self.assertEqual(source.registry_identity()["sha256"], source.SHA256)
            self.assertEqual(sys.path, before)
            self.assertIs(sys.modules["role_aware_bindings"], decoy)
        finally:
            if prior is None:
                sys.modules.pop("role_aware_bindings", None)
            else:
                sys.modules["role_aware_bindings"] = prior

    def test_cache_requires_an_external_complete_nine_file_tree(self) -> None:
        identity = source.registry_identity()
        class FakeCacheApi:
            source_lock = types.SimpleNamespace(load_locked_contract=lambda: {})
            @staticmethod
            def verify_selector_cache(_contract, _root):
                return {"source_contract_sha256": identity["source_contract_sha256"], "selector_record_ids": [], "release_proof_ids": []}
        original = cache._cache_api
        cache._cache_api = lambda: FakeCacheApi
        try:
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                for number in range(9):
                    (root / f"source-{number}").write_bytes(b"x")
                self.assertEqual(cache.verify_selector_cache(root)["file_count"], 9)
                (root / "unexpected").write_bytes(b"x")
                with self.assertRaises(cache.SelectorCacheError):
                    cache.verify_selector_cache(root)
                (root / "unexpected").unlink()
                (root / "source-8").unlink()
                with self.assertRaises(cache.SelectorCacheError):
                    cache.verify_selector_cache(root)
        finally:
            cache._cache_api = original

    def test_cache_rejects_nonexternal_paths_and_payload_without_spdx(self) -> None:
        with self.assertRaises(cache.SelectorCacheError):
            cache.external_root(Path("relative"))
        with self.assertRaises(cache.SelectorCacheError):
            cache.external_root(cache.REPO)
        with tempfile.TemporaryDirectory() as temporary:
            root, link = Path(temporary) / "root", Path(temporary) / "link"
            root.mkdir()
            link.symlink_to(root)
            with self.assertRaises(cache.SelectorCacheError):
                cache.external_root(link)
            payload = root / "payload"
            payload.write_bytes(b"abc")
            fake = copy.deepcopy(source.registry_identity())
            fake.update(bytes=3, sha256=hashlib.sha256(b"abc").hexdigest())
            original = source.registry_identity
            source.registry_identity = lambda: fake
            try:
                with self.assertRaisesRegex(source.RegistryError, "SPDX"):
                    source.payload_bytes(payload, fake)
            finally:
                source.registry_identity = original

    def test_cache_cannot_mix_a_different_sealed_contract(self) -> None:
        identity = source.registry_identity()
        original = cache.verify_selector_cache
        cache.verify_selector_cache = lambda _root: {"source_contract_sha256": "stale"}
        try:
            with tempfile.TemporaryDirectory() as temporary:
                with self.assertRaisesRegex(cache.SelectorCacheError, "different sealed contracts"):
                    cache.payload_from_selector_cache(Path(temporary), identity)
        finally:
            cache.verify_selector_cache = original


if __name__ == "__main__":
    unittest.main()
