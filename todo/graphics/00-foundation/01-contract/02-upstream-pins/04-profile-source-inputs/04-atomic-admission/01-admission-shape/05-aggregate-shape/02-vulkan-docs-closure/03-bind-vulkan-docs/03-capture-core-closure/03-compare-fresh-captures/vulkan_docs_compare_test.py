#!/usr/bin/env python3
"""Positive regressions for the unadmitted independent Docs capture comparison."""

from __future__ import annotations

import unittest
from pathlib import Path
from unittest.mock import call, patch

from vulkan_docs_compare_bind import _compare_fixture, bind_pair
from vulkan_docs_compare_contract import _receipt_value_fixture
from vulkan_docs_compare_fixture import pair
from vulkan_docs_compare_model import STATUS, ComparisonError


class ComparisonTest(unittest.TestCase):
    def test_equal_semantic_scopes_form_an_unadmitted_receipt(self):
        result = _compare_fixture(pair())
        self.assertEqual(result.state, STATUS)
        self.assertFalse(result.admitted)
        self.assertFalse(result.cutover_ready)
        self.assertIs(_receipt_value_fixture(result.value, result), result)

    def test_capture_provenance_is_distinct_but_shared_scope_is_equal(self):
        scopes = pair()
        result = _compare_fixture(scopes)
        first, second = result.value["captures"]
        self.assertNotEqual(first["artifact"], second["artifact"])
        self.assertNotEqual(first["run_sha256"], second["run_sha256"])
        self.assertNotEqual(first["io_trace_sha256"], second["io_trace_sha256"])
        self.assertEqual(first["normalized_sha256"], second["normalized_sha256"])
        self.assertNotEqual(scopes[0].manifest_digest, scopes[1].manifest_digest)
        self.assertEqual(result.value["counts"], {
            "capture_count": 2, "raw_count": 50, "derived_count": 12, "include_count": 14,
            "condition_count": 7, "promotion_count": 4, "extension_control_count": 9, "image_count": 42,
        })

    def test_output_tree_is_a_witness_relation_not_a_payload(self):
        result = _compare_fixture(pair())
        witness = result.value["output_witness"]
        self.assertEqual(witness["file_count"], 2530)
        self.assertEqual(witness["relation"], "two-distinct-observations-known-witness-only")

    def test_public_binder_calls_pinned_binding_before_location_checks(self):
        observation, root, scopes = Path("header.json"), Path("artifact-root"), pair()
        with patch("vulkan_docs_compare_bind.bind_capture", side_effect=scopes) as capture:
            with patch("vulkan_docs_compare_bind.independent_locations") as locations:
                result = bind_pair(observation, root)
        self.assertEqual(capture.call_args_list, [
            call(observation, root, "observer-a"), call(observation, root, "observer-b"),
        ])
        locations.assert_called_once_with(root, scopes)
        self.assertEqual(result.state, STATUS)

    def test_public_binder_stops_before_locations_for_an_unpinned_capture(self):
        with patch("vulkan_docs_compare_bind.bind_capture", side_effect=ComparisonError("unpinned")) as capture:
            with patch("vulkan_docs_compare_bind.independent_locations") as locations:
                with self.assertRaises(ComparisonError):
                    bind_pair(Path("header.json"), Path("artifact-root"))
        capture.assert_called_once_with(Path("header.json"), Path("artifact-root"), "observer-a")
        locations.assert_not_called()


if __name__ == "__main__":
    unittest.main()
