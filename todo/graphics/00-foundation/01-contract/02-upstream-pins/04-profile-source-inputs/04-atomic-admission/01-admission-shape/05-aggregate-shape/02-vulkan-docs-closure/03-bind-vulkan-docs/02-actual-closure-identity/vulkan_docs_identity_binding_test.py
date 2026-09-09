#!/usr/bin/env python3
"""Identity-binding regressions for the unadmitted actual-Docs grammar."""

from __future__ import annotations

import copy
import unittest

from vulkan_docs_identity_contract import closure_value, witness_value
from vulkan_docs_identity_fixture import (
    COMMIT,
    closure,
    raw,
    refresh_member_identity,
    reseal_closure,
    reseal_witness,
    witness,
)
from vulkan_docs_identity_model import DocsIdentityError


class DocsIdentityBindingTest(unittest.TestCase):
    def assert_witness_rejected(self, value):
        reseal_witness(value)
        with self.assertRaises(DocsIdentityError):
            witness_value(value)

    def assert_closure_rejected(self, value, refresh=False):
        if refresh:
            refresh_member_identity(value)
        reseal_closure(value)
        with self.assertRaises(DocsIdentityError):
            closure_value(value, witness_value(witness()))

    def test_root_metadata_cannot_be_substituted(self):
        for field, replacement in (
                ("source_family", "other-docs"), ("bytes", 8686),
                ("license", "LicenseRef-Other"), ("generated_code_role", "other source"),
                ("provenance", "other provenance")):
            with self.subTest(field=field):
                value = closure()
                value["inputs"][0][field] = replacement
                self.assert_closure_rejected(value, refresh=True)

    def test_every_raw_member_must_use_the_pinned_docs_commit(self):
        value = closure()
        alternate = "a" * 40
        value["inputs"][1]["revision"] = alternate
        value["inputs"][1]["immutable_url"] = value["inputs"][1]["immutable_url"].replace(COMMIT, alternate)
        self.assert_closure_rejected(value, refresh=True)

    def test_predecessor_adapter_cannot_be_resealed_to_an_alias(self):
        value = witness()
        value["predecessor_adapter"]["candidate_root_id"] = "vulkan-docs-root"
        self.assert_witness_rejected(value)

    def test_output_recipe_and_metadata_are_not_free_text(self):
        for field, replacement in (("producer_generation_id", "unrelated-generation"),
                                   ("provenance", "unrelated-build")):
            with self.subTest(field=field):
                value = witness()
                value["outputs"][0][field] = replacement
                self.assert_witness_rejected(value)

    def test_scope_binds_the_pinned_configuration_and_generation(self):
        value = closure()
        value["scope"]["configuration_sha256"] = "f" * 64
        self.assert_closure_rejected(value)
        value = closure()
        value["inputs"][-1]["generation_id"] = "unrelated-generation"
        self.assert_closure_rejected(value, refresh=True)

    def test_wsi_and_video_targets_cannot_enter_the_resolved_scope(self):
        for identifier, selector in (
                ("wsi-source", "chapters/VK_KHR_surface/wsi.adoc"),
                ("video-source", "chapters/videocoding.adoc")):
            with self.subTest(selector=selector):
                value = closure()
                value["inputs"].insert(2, raw(identifier, selector, "4" * 64, 42))
                value["scope"]["ordered_input_ids"].insert(2, identifier)
                self.assert_closure_rejected(value, refresh=True)

    def test_extensions_control_input_is_not_an_extension_expansion(self):
        value = closure()
        value["inputs"].insert(2, raw("extensions-control", "chapters/extensions.adoc", "4" * 64, 42))
        value["scope"]["ordered_input_ids"].insert(2, "extensions-control")
        refresh_member_identity(value)
        reseal_closure(value)
        self.assertEqual(len(closure_value(value, witness_value(witness())).inputs), 4)

    def test_partial_or_stale_member_data_is_rejected(self):
        value = closure()
        value["inputs"] = value["inputs"][:2]
        value["scope"]["ordered_input_ids"] = ["vulkan-14-spec", "vulkan-docs-registry"]
        value["scope"]["derived_input_ids"] = []
        self.assert_closure_rejected(value, refresh=True)
        value = closure()
        value["inputs"][1]["provenance"] = "modified-but-unsealed-member"
        self.assert_closure_rejected(value)

    def test_schema_must_be_an_exact_integer(self):
        for malformed in (True, 1.0):
            with self.subTest(witness_schema=malformed):
                value = witness()
                value["schema"] = malformed
                self.assert_witness_rejected(value)
            with self.subTest(closure_schema=malformed):
                value = closure()
                value["schema"] = malformed
                self.assert_closure_rejected(value)

    def test_root_url_or_selector_mismatch_is_rejected(self):
        value = closure()
        value["inputs"][0]["selector"] = "other.adoc"
        self.assert_closure_rejected(value, refresh=True)
        value = closure()
        value["inputs"][0]["immutable_url"] = value["inputs"][0]["immutable_url"].replace(
            "/vkspec.adoc", "/other/vkspec.adoc")
        self.assert_closure_rejected(value, refresh=True)


if __name__ == "__main__":
    unittest.main()
