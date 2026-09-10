#!/usr/bin/env python3
"""Positive and capture-binding regressions for the core input/scope model."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from vulkan_docs_scope_bind import _bind_fixture, bind_value, expected
from vulkan_docs_scope_contract import compact, receipt_value
from vulkan_docs_scope_fixture import expectation, valid
from vulkan_docs_scope_model import ScopeError
from vulkan_docs_scope_parse import canonical
from vulkan_docs_observer_run import seal_run


class ScopeBindingTest(unittest.TestCase):
    def test_valid_scope_and_receipt_remain_unadmitted(self):
        value, capture = valid()
        result = _bind_fixture(value, capture)
        self.assertEqual(result.state, "input-scope-only-unadmitted")
        self.assertFalse(result.admitted)
        self.assertFalse(result.cutover_ready)
        self.assertIs(receipt_value(compact(result), result), result)

    def test_public_binder_refuses_an_unpinned_fixture_capture(self):
        value, capture = valid()
        with self.assertRaises(ScopeError):
            bind_value(value, capture)

    def test_self_sealed_observation_cannot_replace_the_pinned_capture(self):
        source = Path(__file__).resolve().parent.parent / "01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json"
        value = json.loads(source.read_text(encoding="utf-8"))
        run = dict(value["runs"][0])
        run["input_manifest_sha256"] = "f" * 64
        run.pop("run_sha256", None)
        value["runs"][0] = seal_run(run)
        value.pop("observation_sha256", None)
        value["observation_sha256"] = canonical(value, "webboxvm-graphics-vulkan-docs-observation-v1")
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "observation.json"
            path.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(ScopeError):
                expected(path, "observer-a")

    def test_fixture_expectation_matches_the_normalized_identities(self):
        value, capture = valid()
        self.assertEqual(capture, expectation(value))


if __name__ == "__main__":
    unittest.main()
