#!/usr/bin/env python3
"""Hostile checks for the historical successor-closure anchor."""

from __future__ import annotations

import copy
import hashlib
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import successor_closure_anchor as anchor


def sealed(value: dict[str, object]) -> dict[str, object]:
    result = copy.deepcopy(value)
    result["anchor_sha256"] = hashlib.sha256(anchor.canonical(result)).hexdigest()
    return result


class SuccessorClosureAnchorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.value = anchor.document(anchor.RECORD, "successor closure anchor")

    def reject(self, mutate, pattern: str) -> None:
        value = sealed(self.value)
        mutate(value)
        with self.assertRaisesRegex(anchor.AnchorError, pattern):
            anchor.anchor_value(sealed(value))

    def reject_history(self, mutate, pattern: str) -> None:
        history = tuple(copy.deepcopy(value) for value in anchor.historical())
        mutate(*history)
        with patch.object(anchor, "historical", return_value=history):
            with self.assertRaisesRegex(anchor.AnchorError, pattern):
                anchor.anchor_value(sealed(self.value))

    def test_exact_historical_anchor_is_unadmitted(self) -> None:
        self.assertEqual(anchor.anchor(), self.value["anchor_sha256"])

    def test_scope_identity_and_false_effect_tampering_are_rejected(self) -> None:
        cases = (
            (lambda value: value.__setitem__("profile", "vulkan-1.3-core"), "unexpected contract"),
            (lambda value: value["historical"].__setitem__("build_witness_sha256", "a" * 64), "identities"),
            (lambda value: value["counts"].__setitem__("captures", 1), "counts"),
            (lambda value: value["counts"].__setitem__("raw", 298.0), "counts"),
            (lambda value: value["facts"].__setitem__("historical_evidence_is_fresh_closure", True), "effects"),
            (lambda value: value["facts"].__setitem__("rendered_output_can_satisfy_source", True), "effects"),
            (lambda value: value["facts"].__setitem__("source_role_proved", True), "effects"),
            (lambda value: value["effects"].__setitem__("admitted", True), "effects"),
        )
        for mutate, pattern in cases:
            with self.subTest(pattern=pattern):
                self.reject(mutate, pattern)

    def test_policy_hash_document_and_nonregular_paths_are_rejected(self) -> None:
        self.reject(lambda value: value.__setitem__("policy_sha256", "a" * 64), "derived Docs policy")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            duplicate = root / "duplicate.json"
            duplicate.write_text(anchor.RECORD.read_text().replace('"schema": 1', '"schema": 1, "schema": 1', 1))
            with self.assertRaisesRegex(anchor.AnchorError, "duplicate JSON key"):
                anchor.anchor(duplicate)
            oversized = root / "oversized.json"
            oversized.write_bytes(b"x" * (anchor.source_policy.MAX_DOCUMENT_BYTES + 1))
            with self.assertRaisesRegex(anchor.AnchorError, "bounded size"):
                anchor.anchor(oversized)
            fifo = root / "anchor.fifo"
            os.mkfifo(fifo)
            with self.assertRaisesRegex(anchor.AnchorError, "regular file"):
                anchor.anchor(fifo)
            link = root / "anchor-link.json"
            link.symlink_to(anchor.RECORD)
            with self.assertRaisesRegex(anchor.AnchorError, "regular file"):
                anchor.anchor(link)

    def test_historical_status_configuration_and_output_swaps_are_rejected(self) -> None:
        self.reject_history(lambda build, scope, comparison: build.__setitem__("status", "admitted"), "configuration")
        self.reject_history(lambda build, scope, comparison: build["build"].__setitem__("network", "host"), "configuration")
        self.reject_history(lambda build, scope, comparison: build["outputs"][0].__setitem__("kind", "raw-source-input"), "configuration")
        self.reject_history(lambda build, scope, comparison: build.__setitem__("output_tree", []), "configuration")
        self.reject_history(lambda build, scope, comparison: comparison.__setitem__("output_witness", []), "separately classified")

    def test_cli_reports_historical_unadmitted_state_without_writes(self) -> None:
        watched = (anchor.RECORD, anchor.BUILD, anchor.SCOPE, anchor.COMPARISON)
        before = {path: path.read_bytes() for path in watched}
        result = subprocess.run([sys.executable, str(HERE / "successor_closure_anchor.py")], text=True,
                                capture_output=True, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "ANCHOR: historical 298 raw 1462 derived unadmitted")
        self.assertEqual(before, {path: path.read_bytes() for path in watched})
        malformed = subprocess.run([sys.executable, str(HERE / "successor_closure_anchor.py"), "one", "two"], text=True,
                                  capture_output=True, check=False)
        self.assertEqual(malformed.returncode, 2)
        self.assertIn("usage", malformed.stderr)


if __name__ == "__main__":
    unittest.main()
