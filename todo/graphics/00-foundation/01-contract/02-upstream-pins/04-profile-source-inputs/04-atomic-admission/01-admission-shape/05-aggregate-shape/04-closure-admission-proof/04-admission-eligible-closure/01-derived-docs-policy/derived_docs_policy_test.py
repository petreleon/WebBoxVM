#!/usr/bin/env python3
"""Hostile checks for the derived Docs successor-policy boundary."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import derived_docs_policy as contract


def sealed(value: dict[str, object]) -> dict[str, object]:
    result = copy.deepcopy(value)
    result["policy_sha256"] = hashlib.sha256(contract.canonical(result)).hexdigest()
    return result


class DerivedDocsPolicyTest(unittest.TestCase):
    def setUp(self) -> None:
        self.value = contract.document(contract.POLICY)

    def reject(self, mutate, pattern: str) -> None:
        value = sealed(self.value)
        mutate(value)
        value = sealed(value)
        with self.assertRaisesRegex(contract.PolicyError, pattern):
            contract.policy_value(value)

    def test_policy_is_exactly_unadmitted(self) -> None:
        result = contract.policy()
        self.assertEqual(result.source_revision, contract.REVISION)
        self.assertEqual(result.source_member_cap, 8 * 1024 * 1024)

    def test_scope_source_and_builder_tampering_are_rejected(self) -> None:
        cases = (
            (lambda value: value.__setitem__("profile", "vulkan-1.3-core"), "unexpected contract"),
            (lambda value: value["source"].__setitem__("revision", "a" * 40), "reviewed immutable"),
            (lambda value: value["source"].__setitem__("raw_member_max_bytes", 1), "8 MiB"),
            (lambda value: value["source"].__setitem__("tree_identity_required", False), "root-only"),
            (lambda value: value["build"].__setitem__("network", "host"), "networked"),
            (lambda value: value["build"].__setitem__("required_argv_tokens", []), "core build route"),
        )
        for mutate, pattern in cases:
            with self.subTest(pattern=pattern):
                self.reject(mutate, pattern)

    def test_output_license_vcts_and_effect_tampering_are_rejected(self) -> None:
        cases = (
            (lambda value: value["generated"].__setitem__("rendered_output_may_satisfy_source", True), "generated-output"),
            (lambda value: value["generated"].__setitem__("authoritative_license_expression_required", False), "license"),
            (lambda value: value["vcts"].__setitem__("can_satisfy_docs", True), "VCTS"),
            (lambda value: value["vcts"].__setitem__("local_core_selector_allowed", True), "VCTS"),
            (lambda value: value["effects"].__setitem__("admitted", True), "active, support, or release"),
        )
        for mutate, pattern in cases:
            with self.subTest(pattern=pattern):
                self.reject(mutate, pattern)

    def test_hash_duplicate_oversize_and_requirements_drift_are_rejected(self) -> None:
        altered = sealed(self.value)
        altered["policy_sha256"] = "a" * 64
        with self.assertRaisesRegex(contract.PolicyError, "sha256"):
            contract.policy_value(altered)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            duplicate = root / "duplicate.json"
            duplicate.write_text(contract.POLICY.read_text().replace('"schema": 1', '"schema": 1, "schema": 1', 1))
            with self.assertRaisesRegex(contract.PolicyError, "duplicate JSON key"):
                contract.policy(duplicate)
            oversized = root / "oversized.json"
            oversized.write_bytes(b"x" * (contract.MAX_POLICY_BYTES + 1))
            with self.assertRaisesRegex(contract.PolicyError, "bounded size"):
                contract.policy(oversized)
            requirements = root / "requirements.json"
            requirements.write_bytes(contract.REQUIREMENTS.read_bytes() + b"\n")
            with self.assertRaisesRegex(contract.PolicyError, "F03 source requirements"):
                contract.policy(requirements=requirements)

    def test_cli_is_read_only_and_reports_unadmitted_state(self) -> None:
        watched = (contract.POLICY, contract.REQUIREMENTS)
        before = {path: path.read_bytes() for path in watched}
        result = subprocess.run([sys.executable, str(HERE / "derived_docs_policy.py")], text=True,
                                capture_output=True, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), f"POLICY: {contract.REVISION} raw-cap=8388608 unadmitted")
        self.assertEqual(before, {path: path.read_bytes() for path in watched})


if __name__ == "__main__":
    unittest.main()
