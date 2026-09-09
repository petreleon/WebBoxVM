#!/usr/bin/env python3
"""Hostile regressions for the isolated actual-Docs identity grammar."""

from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path

from vulkan_docs_identity_contract import closure_value, witness, witness_value
from vulkan_docs_identity_fixture import closure, raw, reseal_closure, reseal_witness, witness as fixture_witness
from vulkan_docs_identity_members import raw as parse_raw
from vulkan_docs_identity_members import rendered, rendered_outputs
from vulkan_docs_identity_model import DocsIdentityError, MAX_RENDERED_BYTES


class DocsIdentityHostileTest(unittest.TestCase):
    def assert_witness_rejected(self, value):
        reseal_witness(value)
        with self.assertRaises(DocsIdentityError):
            witness_value(value)

    def assert_closure_rejected(self, value):
        reseal_closure(value)
        with self.assertRaises(DocsIdentityError):
            closure_value(value, witness_value(fixture_witness()))

    def test_root_alias_is_rejected_even_when_resealed(self):
        value = fixture_witness()
        value["root"]["id"] = "vulkan-docs-root"
        self.assert_witness_rejected(value)

    def test_mutable_container_context_is_rejected_even_when_resealed(self):
        value = fixture_witness()
        value["build"]["network"] = "bridge"
        self.assert_witness_rejected(value)

    def test_safe_directory_context_cannot_be_erased(self):
        value = fixture_witness()
        value["build"]["safe_directory"] = "unknown"
        self.assert_witness_rejected(value)

    def test_raw_selector_suffix_alias_is_rejected(self):
        value = raw("vulkan-docs-registry", "registry.adoc", "1" * 64, 211)
        value["immutable_url"] = value["immutable_url"].replace("/registry.adoc", "/other/registry.adoc")
        with self.assertRaises(DocsIdentityError):
            parse_raw(value)

    def test_source_input_over_eight_mib_is_rejected(self):
        value = raw("vulkan-docs-registry", "registry.adoc", "1" * 64, 8388609)
        with self.assertRaises(DocsIdentityError):
            parse_raw(value)

    def test_rendered_output_over_its_own_bound_is_rejected(self):
        value = copy.deepcopy(fixture_witness()["outputs"][0])
        value["bytes"] = MAX_RENDERED_BYTES + 1
        with self.assertRaises(DocsIdentityError):
            rendered(value)

    def test_rendered_outputs_reject_selector_collision_not_digest_reuse(self):
        rows = copy.deepcopy(fixture_witness()["outputs"])
        duplicate = copy.deepcopy(rows[0])
        duplicate["id"] = "vkspec-html-copy"
        duplicate["local_cache"] = duplicate["local_cache"].replace("vkspec-html/", "vkspec-html-copy/")
        with self.assertRaises(DocsIdentityError):
            rendered_outputs(rows + [duplicate])

    def test_root_only_closure_is_rejected(self):
        value = closure()
        value["inputs"] = value["inputs"][:1]
        value["scope"]["ordered_input_ids"] = ["vulkan-14-spec"]
        value["scope"]["derived_input_ids"] = []
        self.assert_closure_rejected(value)

    def test_scope_without_reviewed_extension_boundary_is_rejected(self):
        value = closure()
        value["scope"]["excluded_members"]["extensions"] = []
        self.assert_closure_rejected(value)

    def test_source_cache_alias_is_rejected(self):
        value = closure()
        value["inputs"][1]["local_cache"] = value["inputs"][0]["local_cache"]
        self.assert_closure_rejected(value)

    def test_active_closure_state_is_rejected(self):
        value = closure()
        value["status"] = "admitted-actual-docs-closure"
        self.assert_closure_rejected(value)

    def test_duplicate_or_nonfinite_json_is_rejected_before_validation(self):
        with tempfile.TemporaryDirectory() as directory:
            duplicate = Path(directory) / "duplicate.json"
            duplicate.write_text('{"schema":1,"schema":1}', encoding="utf-8")
            with self.assertRaises(DocsIdentityError):
                witness(duplicate)
            nonfinite = Path(directory) / "nonfinite.json"
            nonfinite.write_text('{"schema":NaN}', encoding="utf-8")
            with self.assertRaises(DocsIdentityError):
                witness(nonfinite)


if __name__ == "__main__":
    unittest.main()
