#!/usr/bin/env python3
"""Hermetic transaction-journal and publication tests for live VCTS metadata."""

from __future__ import annotations

import errno
import os
import stat
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import vcts_live_publish as publish


class LivePublishTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir="/private/tmp")
        directory = Path(self.temp.name)
        self.paths = tuple(directory / name for name in ("plan.json", "ledger.json", "receipt.json"))

    def tearDown(self):
        self.temp.cleanup()

    @staticmethod
    def _private(path: Path, value=b"interrupted"):
        path.write_bytes(value)
        path.chmod(0o600)

    def test_rolls_back_an_interrupted_multi_file_publication(self):
        original, calls = publish.os.link, 0
        def fail_second_link(*args, **kwargs):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError(errno.EIO, "injected link failure")
            return original(*args, **kwargs)

        with publish.preflight(self.paths) as transaction:
            publish.os.link = fail_second_link
            try:
                with self.assertRaises(publish.PublishError):
                    publish.publish(transaction, (b"plan", b"ledger", b"receipt"))
            finally:
                publish.os.link = original
        self.assertFalse(any(path.exists() or path.is_symlink() for path in self.paths))
        with publish.preflight(self.paths) as transaction:
            publish.publish(transaction, (b"plan", b"ledger", b"receipt"))
        self.assertTrue(self.paths[2].exists())

    def test_refuses_unowned_partial_output_without_deleting_it(self):
        self._private(self.paths[0], b"caller-owned")
        with self.assertRaises(publish.PublishError):
            publish.preflight(self.paths)
        self.assertEqual(self.paths[0].read_bytes(), b"caller-owned")

    def test_failed_initial_journal_creation_restores_a_fresh_directory(self):
        original, calls = publish.os.fsync, 0
        def fail_first_fsync(fd):
            nonlocal calls
            calls += 1
            if calls == 1:
                raise OSError(errno.EIO, "injected journal fsync failure")
            return original(fd)

        publish.os.fsync = fail_first_fsync
        try:
            with self.assertRaises(publish.PublishError):
                publish.preflight(self.paths)
        finally:
            publish.os.fsync = original
        self.assertFalse(any(Path(self.temp.name).iterdir()))

    def test_surfaces_a_receipt_durability_failure_after_its_visible_commit(self):
        with publish.preflight(self.paths) as transaction:
            original, directories = publish.os.fsync, 0
            def fail_second_directory_fsync(fd):
                nonlocal directories
                if stat.S_ISDIR(os.fstat(fd).st_mode):
                    directories += 1
                    if directories == 2:
                        raise OSError(errno.EIO, "injected final fsync failure")
                return original(fd)

            publish.os.fsync = fail_second_directory_fsync
            try:
                with self.assertRaises(publish.PublishError) as caught:
                    publish.publish(transaction, (b"plan", b"ledger", b"receipt"))
            finally:
                publish.os.fsync = original
        self.assertIn("durable commit", str(caught.exception))
        self.assertTrue(all(path.exists() for path in self.paths))
        with self.assertRaises(publish.PublishError):
            publish.preflight(self.paths)

    def test_receipt_bearing_preflight_safely_removes_its_own_staging_remnant(self):
        with publish.preflight(self.paths) as transaction:
            original, calls = publish.os.unlink, 0
            def fail_receipt_stage(name, *args, **kwargs):
                nonlocal calls
                calls += 1
                if calls == 3:
                    raise OSError(errno.EIO, "injected receipt stage cleanup failure")
                return original(name, *args, **kwargs)

            publish.os.unlink = fail_receipt_stage
            try:
                with self.assertRaises(publish.PublishError) as caught:
                    publish.publish(transaction, (b"plan", b"ledger", b"receipt"))
            finally:
                publish.os.unlink = original
        self.assertIn("staging cleanup", str(caught.exception))
        self.assertTrue(any(path.name.startswith(publish.PREFIX) for path in Path(self.temp.name).iterdir()))
        with self.assertRaises(publish.PublishError):
            publish.preflight(self.paths)
        self.assertFalse(any(path.name.startswith(publish.PREFIX) for path in Path(self.temp.name).iterdir()))

    def test_recovers_only_a_nonce_bound_interrupted_transaction(self):
        with publish.preflight(self.paths) as transaction:
            self._private(self.paths[0])
            self._private(self.paths[1])
            staged = Path(self.temp.name) / (publish.PREFIX + transaction.nonce + "-" + "0" * 32)
            self._private(staged)
        with publish.preflight(self.paths) as resumed:
            self.assertEqual(resumed.nonce, transaction.nonce)
            self.assertFalse(any(path.exists() for path in self.paths))
            self.assertFalse(staged.exists())
            publish.publish(resumed, (b"plan", b"ledger", b"receipt"))
        with self.assertRaises(publish.PublishError):
            publish.preflight(self.paths)

    def test_journal_cannot_recover_different_output_names(self):
        with publish.preflight(self.paths):
            pass
        other = tuple(Path(self.temp.name) / name for name in ("other-plan", "other-ledger", "other-receipt"))
        with self.assertRaises(publish.PublishError):
            publish.preflight(other)
        self.assertTrue((Path(self.temp.name) / ".vcts-live-output-journal").exists())


if __name__ == "__main__":
    unittest.main(verbosity=2)
