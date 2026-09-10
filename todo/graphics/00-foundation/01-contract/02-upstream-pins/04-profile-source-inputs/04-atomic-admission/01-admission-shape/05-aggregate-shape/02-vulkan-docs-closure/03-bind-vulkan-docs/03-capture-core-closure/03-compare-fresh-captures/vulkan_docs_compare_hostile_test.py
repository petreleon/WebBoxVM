#!/usr/bin/env python3
"""Hostile regressions for reused, divergent, or active-looking capture comparisons."""

from __future__ import annotations

import copy
from dataclasses import replace
import unittest

from vulkan_docs_compare_bind import _compare_fixture
from vulkan_docs_compare_contract import _receipt_value_fixture
from vulkan_docs_compare_fixture import pair
from vulkan_docs_compare_model import ComparisonError
from vulkan_docs_compare_parse import canonical


class ComparisonHostileTest(unittest.TestCase):
    def rejected(self, mutate, message: str) -> None:
        scopes = list(pair())
        mutate(scopes)
        with self.assertRaises(ComparisonError) as error:
            _compare_fixture(scopes)
        self.assertIn(message, str(error.exception))

    def test_semantic_identity_differences_are_rejected(self):
        cases = (
            (lambda scopes: scopes.__setitem__(1, replace(
                scopes[1], capture=replace(scopes[1].capture, input_manifest_digest="f" * 64))),
             "input_manifest_sha256"),
            (lambda scopes: scopes[1].value["raw_records"][0].__setitem__("sha256", "f" * 64),
             "raw_identity_sha256"),
            (lambda scopes: scopes[1].value["derived_records"][0].__setitem__("sha256", "f" * 64),
             "derived_identity_sha256"),
            (lambda scopes: scopes.__setitem__(1, replace(
                scopes[1], capture=replace(scopes[1].capture, include_digest="f" * 64))),
             "include_identity_sha256"),
            (lambda scopes: scopes[1].value["conditions"].__setitem__("identity_sha256", "f" * 64), "conditions_identity_sha256"),
            (lambda scopes: scopes[1].value["producer"].__setitem__("identity_sha256", "f" * 64), "producer_identity_sha256"),
            (lambda scopes: scopes[1].value.__setitem__("configuration_sha256", "f" * 64), "configuration_sha256"),
            (lambda scopes: scopes[1].value.__setitem__("build_witness_sha256", "f" * 64), "build_witness_sha256"),
            (lambda scopes: scopes.__setitem__(1, replace(scopes[1], scope_digest="f" * 64)), "scope_identity_sha256"),
            (lambda scopes: scopes.__setitem__(1, replace(scopes[1], capture=replace(scopes[1].capture, normalized_digest="f" * 64))),
             "normalized_sha256"),
        )
        for mutate, message in cases:
            with self.subTest(message=message):
                self.rejected(mutate, message)

    def test_capture_trace_and_observation_differences_are_rejected(self):
        cases = (
            (lambda scopes: scopes.__setitem__(1, replace(
                scopes[1], capture=replace(scopes[1].capture, include_trace_digest="f" * 64))),
             "include_trace_sha256"),
            (lambda scopes: scopes.__setitem__(1, replace(
                scopes[1], capture=replace(scopes[1].capture, observation_digest="f" * 64))),
             "observation identity"),
        )
        for mutate, message in cases:
            with self.subTest(message=message):
                self.rejected(mutate, message)

    def test_reused_capture_provenance_is_rejected(self):
        for name in ("identifier", "artifact", "run_digest", "io_trace_digest"):
            def mutate(scopes, field=name):
                scopes[1] = replace(scopes[1], capture=replace(scopes[1].capture, **{field: getattr(scopes[0].capture, field)}))
            with self.subTest(field=name):
                self.rejected(mutate, "comparison")

    def test_zero_evidence_and_output_witness_substitution_are_rejected(self):
        self.rejected(lambda scopes: (scopes[0].value.__setitem__("raw_records", []),
                                      scopes[1].value.__setitem__("raw_records", [])), "zero or malformed")
        self.rejected(lambda scopes: scopes.__setitem__(1, replace(
            scopes[1], capture=replace(scopes[1].capture, generated_tree_digest="f" * 64))), "output witness")

    def test_resealed_active_or_changed_receipt_is_rejected(self):
        result = _compare_fixture(pair())
        receipt = copy.deepcopy(result.value)
        receipt["status"] = "admitted"
        payload = dict(receipt)
        payload.pop("comparison_sha256", None)
        receipt["comparison_sha256"] = canonical(payload, "webboxvm-graphics-vulkan-docs-core-capture-comparison-v1")
        with self.assertRaises(ComparisonError):
            _receipt_value_fixture(receipt, result)


if __name__ == "__main__":
    unittest.main()
