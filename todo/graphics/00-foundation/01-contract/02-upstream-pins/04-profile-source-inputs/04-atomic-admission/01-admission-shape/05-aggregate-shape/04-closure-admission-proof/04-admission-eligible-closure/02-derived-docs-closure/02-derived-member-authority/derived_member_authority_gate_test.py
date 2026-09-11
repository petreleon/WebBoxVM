#!/usr/bin/env python3
"""Hostile checks for the external derived-member-authority blocker."""

from __future__ import annotations

import copy
import hashlib
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
TEMP_ROOT = Path("/private/tmp") if Path("/private/tmp").is_dir() else Path(tempfile.gettempdir()).resolve()
sys.path.insert(0, str(HERE))
import derived_member_authority_gate as gate


def sealed(value: dict[str, object]) -> dict[str, object]:
    result = copy.deepcopy(value)
    result["gate_sha256"] = hashlib.sha256(gate.canonical(result)).hexdigest()
    return result


class DerivedMemberAuthorityGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.value = gate.document(gate.RECORD)

    def reject(self, mutate, pattern: str) -> None:
        value = sealed(self.value)
        mutate(value)
        with self.assertRaisesRegex(gate.AuthorityError, pattern):
            gate.gate_value(sealed(value))

    def test_current_gate_is_exactly_external_authority_blocked(self) -> None:
        self.assertEqual(gate.gate(), self.value["gate_sha256"])
        with self.assertRaisesRegex(gate.AuthorityBlocked, "per-derived external authority"):
            gate.require_authority_manifest(self.value["external_authority_manifest"])

    def test_scope_identity_and_effect_tampering_are_rejected(self) -> None:
        cases = (
            (lambda value: value.__setitem__("profile", "vulkan-1.3-core"), "unexpected contract"),
            (lambda value: value.__setitem__("anchor_sha256", "a" * 64), "historical successor"),
            (lambda value: value.__setitem__("expected_derived_count", 1462.0), "derived-member count"),
            (lambda value: value["identity_fields"].pop(), "identity fields"),
            (lambda value: value["authority_fields"].reverse(), "authority fields"),
            (lambda value: value["prohibitions"].pop(), "prohibitions"),
            (lambda value: value["facts"].__setitem__("authority_manifest_present", True), "authority facts"),
            (lambda value: value["effects"].__setitem__("admitted", True), "authority effects"),
        )
        for mutate, pattern in cases:
            with self.subTest(pattern=pattern):
                self.reject(mutate, pattern)

    def test_local_manifest_and_type_aliases_are_rejected(self) -> None:
        fabricated = {"members": [{"license_expression": "Apache-2.0"}]}
        self.reject(lambda value: value.__setitem__("external_authority_manifest", fabricated), "unanchored")
        self.reject(lambda value: value.__setitem__("expected_derived_count", True), "derived-member count")
        with self.assertRaisesRegex(gate.AuthorityError, "unanchored"):
            gate.require_authority_manifest(fabricated)

    def test_duplicate_oversize_and_nonregular_gate_paths_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory(dir=TEMP_ROOT) as temporary:
            root = Path(temporary)
            duplicate = root / "duplicate.json"
            duplicate.write_text(gate.RECORD.read_text().replace('"schema": 1', '"schema": 1, "schema": 1', 1))
            with self.assertRaisesRegex(gate.AuthorityError, "duplicate JSON key"):
                gate.gate(duplicate)
            oversized = root / "oversized.json"
            oversized.write_bytes(b"x" * (gate.closure_anchor.source_policy.MAX_DOCUMENT_BYTES + 1))
            with self.assertRaisesRegex(gate.AuthorityError, "bounded size"):
                gate.gate(oversized)
            fifo = root / "gate.fifo"
            os.mkfifo(fifo)
            with self.assertRaisesRegex(gate.AuthorityError, "regular file"):
                gate.gate(fifo)
            link = root / "gate-link.json"
            link.symlink_to(gate.RECORD)
            with self.assertRaisesRegex(gate.AuthorityError, "safely opened"):
                gate.gate(link)
            redirect = root / "redirect"
            redirect.symlink_to(gate.HERE, target_is_directory=True)
            with self.assertRaisesRegex(gate.AuthorityError, "safely opened"):
                gate.gate(redirect / gate.RECORD.name)

    def test_cli_is_read_only_and_reports_blocked_state(self) -> None:
        watched = (gate.RECORD, gate.closure_anchor.RECORD, *gate.closure_anchor.source_policy.ANCHORS.values())
        paths = tuple(item[0] if isinstance(item, tuple) else item for item in watched)
        before = {path: path.read_bytes() for path in paths}
        result = subprocess.run([sys.executable, str(HERE / "derived_member_authority_gate.py")], text=True,
                                capture_output=True, check=False)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(result.stdout.strip(), "AUTHORITY-GATE: 1462 derived; external authority unavailable\n"
                         "BLOCKED: per-derived external authority is unavailable")
        self.assertEqual(before, {path: path.read_bytes() for path in paths})
        malformed = subprocess.run([sys.executable, str(HERE / "derived_member_authority_gate.py"), "one", "two"],
                                  text=True, capture_output=True, check=False)
        self.assertEqual(malformed.returncode, 2)
        self.assertIn("usage", malformed.stderr)


if __name__ == "__main__":
    unittest.main()
