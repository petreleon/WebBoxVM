#!/usr/bin/env python3
"""Hostile tests for unchanged F02/F03 consumer revalidation."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("consumer_revalidation", HERE / "consumer_revalidation.py")
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot load consumer revalidation")
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)


class ConsumerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name).resolve()
        self.addCleanup(self.temporary.cleanup)

    def record(self):
        return json.loads(MOD.RECORD.read_text(encoding="utf-8"))

    def write(self, value, name="consumer.json"):
        path = self.root / name
        path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")
        return path

    def sealed(self, value):
        value["consumer_revalidation_sha256"] = MOD.digest(value)
        return value

    def rejected(self, change):
        value = self.record(); change(value); path = self.write(self.sealed(value))
        with self.assertRaisesRegex(MOD.ConsumerError, "exact blocked consumers"):
            MOD.validate(path)

    def test_committed_consumer_record_is_read_only_blocked_and_offline(self):
        watched = (MOD.RECORD, MOD.BINDING.RECORD, MOD.LOCK, MOD.RECEIPT.RECEIPT,
                   MOD.F03.REQUIREMENTS_PATH, MOD.F03.SCOPE)
        before = tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in watched)
        with mock.patch.object(MOD.BINDING, "replay", side_effect=AssertionError("replay")) as replay, \
             mock.patch.object(MOD.BINDING.RUN, "fetch_to_cache", side_effect=AssertionError("fetch")) as fetch:
            value = MOD.validate()
        self.assertEqual((value["status"], value["f03"]["missing_required_input_ids"]),
                         ("blocked-unchanged-consumers", list(MOD.MISSING)))
        self.assertFalse(value["active_f02"]["captured_member_ids_present"])
        self.assertTrue(all(item is False for item in value["effects"].values()))
        replay.assert_not_called(); fetch.assert_not_called()
        self.assertEqual(before, tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in watched))

    def test_resealed_alias_f03_omission_and_promotion_are_rejected(self):
        changes = (
            lambda value: value.__setitem__("status", "admitted"),
            lambda value: value["binding"].__setitem__("binding_sha256", "0" * 64),
            lambda value: value["binding"].__setitem__("active_alias_allowed", True),
            lambda value: value["active_f02"].__setitem__("inventory_lock_sha256", "0" * 64),
            lambda value: value["active_f02"].__setitem__("captured_member_ids_present", True),
            lambda value: value["f03"]["missing_required_input_ids"].pop(),
            lambda value: value["f03"]["missing_required_input_ids"].reverse(),
            lambda value: value["f03"]["missing_required_input_ids"].__setitem__(0, "gles-cts-manifest"),
            lambda value: value["f03"].__setitem__("blocker", "matrix-incomplete"),
            lambda value: value["historical_receipt"]["readiness"].__setitem__("admission_eligible", True),
            lambda value: value["effects"].__setitem__("admitted", True),
            lambda value: value["effects"].__setitem__("admitted", 0),
        )
        for change in changes:
            with self.subTest(change=change):
                self.rejected(change)

    def test_duplicate_oversize_fifo_and_symlink_records_are_rejected(self):
        root = self.root
        duplicate = root / "duplicate.json"; duplicate.write_text('{"schema":1,"schema":2}', encoding="utf-8")
        oversized = root / "oversized.json"; oversized.write_bytes(b"x" * (MOD.MAX_BYTES + 1))
        huge = root / "huge.json"; huge.write_text('{"schema":' + "9" * 5000 + '}', encoding="utf-8")
        nonfinite = root / "nonfinite.json"; nonfinite.write_text('{"schema":NaN}', encoding="utf-8")
        fifo = root / "consumer.fifo"; os.mkfifo(fifo)
        link = root / "consumer-link.json"; link.symlink_to(MOD.RECORD)
        for path in (duplicate, oversized, huge, nonfinite, fifo, link):
            with self.subTest(path=path.name), self.assertRaises(MOD.ConsumerError):
                MOD.validate(path)
        valid = self.write(self.record(), "valid.json")
        inside = root / "inside"; inside.mkdir()
        redirect = inside / "redirect"; redirect.symlink_to(root, target_is_directory=True)
        with self.assertRaises(MOD.ConsumerError):
            MOD.validate(redirect / valid.name)

    def test_predecessor_drift_and_active_aliases_fail_before_a_result(self):
        binding = copy.deepcopy(MOD.BINDING.validate())
        binding["capture"]["members"][0]["id"] = "mesa-virgl-screen"
        fingerprint = copy.deepcopy(MOD.BINDING.validate())
        active = MOD.LAYOUT.load_inventory(MOD.MANIFEST).inputs[0]
        fingerprint["capture"]["members"][0].update({"id": "not-active", "sha256": active["sha256"], "bytes": active["bytes"]})
        receipt = copy.deepcopy(MOD.RECEIPT.validate())
        receipt["readiness"]["f03_ready"] = True
        with mock.patch.object(MOD.BINDING, "validate", return_value=binding):
            with self.assertRaisesRegex(MOD.ConsumerError, "exact blocked F02/F03"):
                MOD.build()
        with mock.patch.object(MOD.BINDING, "validate", return_value=fingerprint):
            with self.assertRaisesRegex(MOD.ConsumerError, "exact blocked F02/F03"):
                MOD.build()
        with mock.patch.object(MOD.RECEIPT, "validate", return_value=receipt):
            with self.assertRaisesRegex(MOD.ConsumerError, "exact blocked F02/F03"):
                MOD.build()
        with mock.patch.object(MOD.F03, "validate", return_value=MOD.MISSING[:-1]):
            with self.assertRaisesRegex(MOD.ConsumerError, "exact blocked F02/F03"):
                MOD.build()
        original = MOD.sha256
        with mock.patch.object(MOD, "sha256", side_effect=lambda path: "0" * 64 if path == MOD.LOCK else original(path)):
            with self.assertRaisesRegex(MOD.ConsumerError, "exact blocked F02/F03"):
                MOD.build()


if __name__ == "__main__":
    unittest.main(verbosity=2)
