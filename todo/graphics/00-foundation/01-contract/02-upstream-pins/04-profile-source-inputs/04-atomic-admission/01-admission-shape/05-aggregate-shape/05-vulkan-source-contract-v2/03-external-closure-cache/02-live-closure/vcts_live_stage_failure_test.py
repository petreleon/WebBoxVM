#!/usr/bin/env python3
"""Targeted cleanup regressions for VCTS live-stage setup and stream closure."""

from __future__ import annotations

import errno
import hashlib
import os
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE = HERE.parent / "01-cache-contract"
for location in (HERE, CACHE):
    sys.path.insert(0, str(location))
import vcts_cache_contract as cache
import vcts_live_stage as live


def blob(data: bytes) -> str:
    return hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest()


class LiveStageFailureTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir="/private/tmp")
        self.root = Path(self.temp.name) / "stage"
        self.repository = cache.repository_root(HERE)

    def tearDown(self):
        self.temp.cleanup()

    def test_setup_failure_removes_only_the_leaf_it_created(self):
        original = live.os.dup
        live.os.dup = lambda *_: (_ for _ in ()).throw(OSError(errno.EIO, "injected"))
        try:
            with self.assertRaises(OSError):
                live._directory(self.root, self.repository, "a" * 64, "b" * 64)
        finally:
            live.os.dup = original
        leaf = self.root / "webboxvm-graphics" / "v2" / "vcts-live-stage" / ("a" * 64) / ("b" * 64)
        self.assertFalse(leaf.exists())

    def test_close_error_is_not_retried_and_leaves_no_part_file(self):
        directory = Path(self.temp.name) / "payloads"
        directory.mkdir(mode=0o700)
        fd = os.open(directory, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
        original, calls = live.os.close, []
        def close_then_fail(value):
            calls.append(value)
            original(value)
            raise OSError(errno.EIO, "injected")

        live.os.close = close_then_fail
        try:
            with self.assertRaises(OSError) as caught:
                live._capture(fd, "payload", "https://example.invalid", "x", "r", {"bytes": 1, "blob_sha1": blob(b"x")}, iter((b"x",)))
        finally:
            live.os.close = original
            os.close(fd)
        self.assertEqual(caught.exception.errno, errno.EIO)
        self.assertEqual(len(calls), 1)
        self.assertFalse(any(directory.iterdir()))


if __name__ == "__main__":
    unittest.main(verbosity=2)
