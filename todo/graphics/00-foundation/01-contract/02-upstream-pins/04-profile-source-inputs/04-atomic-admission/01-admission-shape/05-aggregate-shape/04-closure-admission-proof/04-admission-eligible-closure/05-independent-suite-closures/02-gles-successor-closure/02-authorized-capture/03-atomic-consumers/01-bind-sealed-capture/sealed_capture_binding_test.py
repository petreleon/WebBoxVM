#!/usr/bin/env python3
"""Hostile tests for the non-admitting sealed GLES capture binding."""

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
SPEC = importlib.util.spec_from_file_location("sealed_capture_binding", HERE / "sealed_capture_binding.py")
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot load sealed capture binding")
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)


class BindingTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)

    def record(self):
        return json.loads(MOD.RECORD.read_text(encoding="utf-8"))

    def write(self, value, name="binding.json"):
        path = Path(self.temporary.name) / name
        path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")
        return path

    def sealed(self, value):
        value["binding_sha256"] = MOD.digest(value)
        return value

    def cache(self, marker):
        root = Path(self.temporary.name) / "cache"; root.mkdir()
        cache = MOD.RUN.ExternalCache.from_path(root, Path(self.temporary.name) / "repository")
        target = cache.root.joinpath(*MOD.MARKER.relative(MOD.PLAN.plan()))
        target.parent.mkdir(parents=True)
        target.write_bytes(json.dumps(marker, sort_keys=True, separators=(",", ":")).encode())
        return cache, target

    def rejected(self, change):
        value = self.record(); change(value); path = self.write(self.sealed(value))
        with self.assertRaisesRegex(MOD.BindingError, "exact sealed closure"):
            MOD.validate(path)

    def test_committed_binding_is_deterministic_and_read_only(self):
        watched = (MOD.RECORD, MOD.INTEGRATION.RECORD, MOD.PLAN.CONTRACT_DIR / "gles_successor_contract.json")
        before = tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in watched)
        value = MOD.validate()
        self.assertEqual((value["status"], len(value["capture"]["members"])), ("captured-unadmitted", 6))
        self.assertEqual(tuple(row["role"] for row in value["capture"]["members"]),
                         ("root", "core", "core", "core", "core", "excluded-extension"))
        self.assertTrue(all(item is False for item in value["readiness"].values()))
        self.assertEqual(before, tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in watched))

    def test_resealed_identity_partial_and_promotion_changes_are_rejected(self):
        changes = (
            lambda value: value.__setitem__("status", "admitted"),
            lambda value: value.__setitem__("integration_sha256", "0" * 64),
            lambda value: value["capture"].__setitem__("contract_sha256", "0" * 64),
            lambda value: value["capture"].__setitem__("configuration_document_sha256", "0" * 64),
            lambda value: value["capture"]["members"].pop(),
            lambda value: value["capture"]["members"].reverse(),
            lambda value: value["capture"].__setitem__("marker_relative_path", "webboxvm-graphics/f02/x"),
            lambda value: value["readiness"].__setitem__("admitted", True),
            lambda value: value["readiness"].__setitem__("admitted", 0),
        )
        for change in changes:
            with self.subTest(change=change):
                self.rejected(change)

    def test_duplicate_oversize_fifo_and_symlink_records_are_rejected(self):
        root = Path(self.temporary.name)
        duplicate = root / "duplicate.json"; duplicate.write_text('{"schema":1,"schema":2}', encoding="utf-8")
        oversized = root / "oversized.json"; oversized.write_bytes(b"x" * (MOD.MAX_BYTES + 1))
        fifo = root / "binding.fifo"; os.mkfifo(fifo)
        link = root / "binding-link.json"; link.symlink_to(MOD.RECORD)
        for path in (duplicate, oversized, fifo, link):
            with self.subTest(path=path.name), self.assertRaises(MOD.BindingError):
                MOD.validate(path)

    def test_live_binding_requires_replay_and_the_same_marker(self):
        marker = MOD.MARKER.value(MOD.PLAN.plan()); cache, target = self.cache(marker)
        with mock.patch.object(MOD.RUN, "fetch_to_cache") as fetch, mock.patch.object(MOD.RUN, "replay") as replay, mock.patch.object(MOD.MARKER, "read", return_value=marker):
            self.assertEqual(MOD.replay(cache)["binding_sha256"], MOD.build()["binding_sha256"])
            replay.assert_called_once(); fetch.assert_not_called()
        with mock.patch.object(MOD.RUN, "replay", side_effect=RuntimeError("partial")), mock.patch.object(MOD.MARKER, "read") as read:
            with self.assertRaisesRegex(MOD.BindingError, "cannot replay"):
                MOD.replay(cache)
            read.assert_not_called()
        forged = copy.deepcopy(marker); forged["marker_sha256"] = "0" * 64
        with mock.patch.object(MOD.RUN, "replay"), mock.patch.object(MOD.MARKER, "read", return_value=forged):
            with self.assertRaisesRegex(MOD.BindingError, "does not bind"):
                MOD.replay(cache)
        target.write_text(json.dumps(marker, indent=2), encoding="utf-8")
        with mock.patch.object(MOD.RUN, "replay"), mock.patch.object(MOD.MARKER, "read", return_value=marker):
            with self.assertRaisesRegex(MOD.BindingError, "does not bind"):
                MOD.replay(cache)


if __name__ == "__main__":
    unittest.main(verbosity=2)
