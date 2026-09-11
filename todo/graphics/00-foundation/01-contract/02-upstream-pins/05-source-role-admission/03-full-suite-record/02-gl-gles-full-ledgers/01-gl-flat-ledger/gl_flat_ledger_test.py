#!/usr/bin/env python3
"""Focused hostile checks for the local OpenGL flat-sequence observation."""

import copy
import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import gl_flat_ledger as ledger


class GlFlatLedgerTests(unittest.TestCase):
    def sample_record(self, data: bytes) -> dict[str, object]:
        record = copy.deepcopy(ledger.root_record())
        record.update(bytes=len(data), sha256=hashlib.sha256(data).hexdigest())
        return record

    def test_canonical_root_is_gl46_and_has_no_local_qualification_claim(self):
        record = ledger.root_record()
        self.assertEqual((record["id"], record["profile"], record["unfiltered"]),
                         ("opengl-cts-gl46-main", "opengl-4.6-core", True))
        self.assertTrue(record["claims"]["khronos_selector"])
        self.assertEqual(ledger.CASE_COUNT, 19714)

    def test_strict_fixture_binds_order_count_and_no_claim_receipt(self):
        data = b"KHR.a\nKHR.b\nKHR.c\n"
        record = self.sample_record(data)
        with patch.object(ledger, "root_record", return_value=record), patch.object(ledger, "CASE_COUNT", 3), \
                patch.object(ledger, "CASE_SEQUENCE_SHA256", hashlib.sha256(data).hexdigest()):
            value = ledger.ledger(data)
            ledger.validate_ledger(value, data)
        self.assertEqual(value["cts_executions"], 0)
        self.assertTrue(all(not claim for claim in value["claims"].values()))
        self.assertEqual(value["source_root"]["authority"], "Khronos")
        self.assertEqual(value["authority"], "WebBoxVM")

    def test_rejects_blank_padded_nul_duplicate_or_non_lf_cases(self):
        for data in (b"KHR.a\n\n", b" KHR.a\n", b"KHR.a\x00\n", b"KHR.a\nKHR.a\n", b"KHR.a"):
            with self.subTest(data=data), self.assertRaises(ledger.LedgerError):
                ledger.parse_cases(data)

    def test_rejects_reordered_substituted_or_wrong_count_sequence(self):
        data = b"KHR.a\nKHR.b\nKHR.c\n"
        record = self.sample_record(data)
        digest = hashlib.sha256(data).hexdigest()
        with patch.object(ledger, "CASE_COUNT", 3), patch.object(ledger, "CASE_SEQUENCE_SHA256", digest):
            with patch.object(ledger, "root_record", return_value=record):
                ledger.ledger(data)
            reordered = b"KHR.b\nKHR.a\nKHR.c\n"
            with patch.object(ledger, "root_record", return_value=self.sample_record(reordered)):
                with self.assertRaises(ledger.LedgerError):
                    ledger.ledger(reordered)
            with patch.object(ledger, "root_record", return_value={**record, "sha256": "0" * 64}):
                with self.assertRaises(ledger.LedgerError):
                    ledger.ledger(data)
        with patch.object(ledger, "CASE_COUNT", 4):
            with patch.object(ledger, "root_record", return_value=record):
                with self.assertRaises(ledger.LedgerError):
                    ledger.ledger(data)

    def test_nonempty_fresh_cache_fails_before_network(self):
        with tempfile.TemporaryDirectory() as temporary:
            Path(temporary, "old").write_bytes(b"old")
            with self.assertRaises(ledger.LedgerError):
                ledger.refresh(Path(temporary), 1)


if __name__ == "__main__":
    unittest.main()
