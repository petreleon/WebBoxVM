#!/usr/bin/env python3
"""Positive and boundary tests for the isolated actual-Docs identity grammar."""

from __future__ import annotations

import copy
import unittest

from vulkan_docs_identity_contract import closure_value, witness_value
from vulkan_docs_identity_fixture import closure, refresh_member_identity, reseal_closure, witness
from vulkan_docs_identity_members import rendered, rendered_outputs
from vulkan_docs_identity_model import DocsIdentityError
from vulkan_docs_identity_parse import F02


class DocsIdentityTest(unittest.TestCase):
    def test_actual_witness_is_explicitly_unadmitted(self):
        result = witness_value(witness())
        self.assertFalse(result.admitted)
        self.assertFalse(result.cutover_ready)
        self.assertEqual(result.outputs[0].byte_count, 10377052)

    def test_rendered_html_is_not_a_source_input(self):
        result = witness_value(witness())
        self.assertGreater(result.outputs[0].byte_count, F02.MAX_INPUT_BYTES)
        self.assertEqual(result.outputs[0].selector, "out/html/vkspec.html")
        self.assertFalse(hasattr(result.outputs[0], "immutable_url"))

    def test_actual_witness_binds_two_equal_trees(self):
        result = witness_value(witness())
        self.assertEqual(result.tree_digest, "26e8e484d34222d9ba8a72c883ff8bb65a4e5194f399c24aa9bd5d7e47c49f32")

    def test_unadmitted_closure_accepts_raw_and_derived_inputs(self):
        result = closure_value(closure(), witness_value(witness()))
        self.assertEqual(len(result.inputs), 3)
        self.assertFalse(result.admitted)
        self.assertFalse(result.cutover_ready)

    def test_equal_output_bytes_at_distinct_selectors_are_valid(self):
        rows = copy.deepcopy(witness()["outputs"])
        duplicate = copy.deepcopy(rows[0])
        duplicate["id"] = "vkspec-html-copy"
        duplicate["selector"] = "out/html/vkspec-copy.html"
        duplicate["local_cache"] = duplicate["local_cache"].replace("vkspec-html/", "vkspec-html-copy/")
        self.assertEqual(len(rendered_outputs(rows + [duplicate])), 2)

    def test_generated_input_has_no_raw_url_or_revision(self):
        result = closure_value(closure(), witness_value(witness()))
        generated = result.inputs[-1]
        self.assertFalse(hasattr(generated, "immutable_url"))

    def test_equal_source_bytes_at_distinct_inputs_are_valid(self):
        value = closure()
        duplicate = value["inputs"][1]
        duplicate["sha256"] = value["inputs"][0]["sha256"]
        duplicate["local_cache"] = (
            "webboxvm-graphics/successor/vulkan-docs-v1/raw/vulkan-docs-registry/"
            f"{duplicate['sha256']}.source")
        refresh_member_identity(value)
        reseal_closure(value)
        self.assertEqual(len(closure_value(value, witness_value(witness())).inputs), 3)

    def test_rendered_output_cannot_gain_raw_source_fields(self):
        for field, replacement in (("immutable_url", "https://example.invalid/source"),
                                   ("revision", "a" * 40)):
            with self.subTest(field=field):
                value = copy.deepcopy(witness()["outputs"][0])
                value[field] = replacement
                with self.assertRaises(DocsIdentityError):
                    rendered(value)


if __name__ == "__main__":
    unittest.main()
