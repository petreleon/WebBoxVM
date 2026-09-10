#!/usr/bin/env python3
"""Exercise the synthetic collector boundary before a real tracer uses it."""

from __future__ import annotations

import copy
import unittest

from lineage_bind import bind
from lineage_capture import SealedScope, raw_member_id
from lineage_events import parse
from lineage_model import LineageError
from lineage_parse import stable_member_id
from lineage_synthetic_normalize import normalize

RAW, DERIVED, ARGV = "a" * 64, "b" * 64, "c" * 64


def wire() -> list[dict[str, object]]:
    return [
        {"kind": "process", "pid": 10, "parent": None, "argv_sha256": ARGV},
        {"kind": "open", "pid": 10, "fd": 3, "path": "/vulkan/raw.adoc", "mode": "read"},
        {"kind": "read", "pid": 10, "fd": 3}, {"kind": "close", "pid": 10, "fd": 3},
        {"kind": "open", "pid": 10, "fd": 4, "path": "/work/temporary/out.tmp", "mode": "write"},
        {"kind": "write", "pid": 10, "fd": 4}, {"kind": "close", "pid": 10, "fd": 4},
        {"kind": "rename", "pid": 10, "old": "/work/temporary/out.tmp", "new": "/work/generated/out.adoc"},
        {"kind": "process", "pid": 11, "parent": 10, "argv_sha256": ARGV},
        {"kind": "open", "pid": 11, "fd": 3, "path": "/work/generated/out.adoc", "mode": "read"},
        {"kind": "read", "pid": 11, "fd": 3}, {"kind": "close", "pid": 11, "fd": 3},
        {"kind": "exec", "pid": 11, "argv_sha256": ARGV}, {"kind": "exit", "pid": 11}, {"kind": "exit", "pid": 10},
    ]


def snapshots() -> dict[str, dict[str, object]]:
    return {"raw.adoc": {"sha256": RAW, "bytes": 3}, "generated/out.adoc": {"sha256": DERIVED, "bytes": 7}}


def scope() -> SealedScope:
    return SealedScope("1" * 64, "2" * 64, "3" * 64, "fixture-a", ((raw_member_id("raw.adoc"), "raw.adoc", RAW, 3),))


class SyntheticNormalizeTest(unittest.TestCase):
    def assert_rejected(self, change, changed_snapshots=None):
        observed, state = wire(), snapshots(); change(observed)
        if changed_snapshots is not None: changed_snapshots(state)
        with self.assertRaises(LineageError): normalize(observed, state)

    def test_normalized_collector_wire_binds_one_forked_derived_input(self):
        events = normalize(wire(), snapshots())
        records = bind(parse(events), scope())
        derived_id = stable_member_id("derived-source-input", "generated/out.adoc")
        self.assertEqual([row["kind"] for row in events], ["process", "read", "write", "close", "rename", "final", "process", "read", "exec", "exit", "exit"])
        self.assertEqual([(item.identifier, item.selector, item.writer_instance, item.producer_ids) for item in records],
                         [(derived_id, "generated/out.adoc", "p-10-1", (raw_member_id("raw.adoc"),))])

    def test_rejects_schema_root_mode_and_missing_snapshot_escapes(self):
        self.assert_rejected(lambda rows: rows[0].__setitem__("extra", True))
        self.assert_rejected(lambda rows: rows[0].__setitem__("kind", []))
        self.assert_rejected(lambda rows: rows[1].__setitem__("path", "/etc/passwd"))
        self.assert_rejected(lambda rows: rows[1].__setitem__("mode", []))
        self.assert_rejected(lambda rows: rows[4].__setitem__("path", "/work/generated/out.adoc"))
        self.assert_rejected(lambda rows: None, lambda state: state.pop("generated/out.adoc"))

    def test_rejects_unwritten_duplicate_or_unclosed_writer_and_lifecycle_breaks(self):
        self.assert_rejected(lambda rows: rows.__setitem__(5, {"kind": "close", "pid": 10, "fd": 4}))
        self.assert_rejected(lambda rows: rows.insert(5, {"kind": "open", "pid": 10, "fd": 5, "path": "/work/temporary/out.tmp", "mode": "write"}))
        self.assert_rejected(lambda rows: rows.pop())
        self.assert_rejected(lambda rows: rows.insert(1, copy.deepcopy(rows[0])))

    def test_rejects_live_fds_wrong_access_and_rename_order(self):
        self.assert_rejected(lambda rows: rows.insert(2, {"kind": "process", "pid": 12, "parent": 10, "argv_sha256": ARGV}))
        self.assert_rejected(lambda rows: rows[8].__setitem__("parent", True))
        self.assert_rejected(lambda rows: rows[2].__setitem__("kind", "write"))
        self.assert_rejected(lambda rows: rows[5].__setitem__("kind", "read"))
        self.assert_rejected(lambda rows: rows.__setitem__(slice(6, 8), [rows[7], rows[6]]))
        self.assert_rejected(lambda rows: rows.__setitem__(11, {"kind": "exec", "pid": 11, "argv_sha256": ARGV}))

    def test_binder_rejects_unsealed_raw_and_derived_read_before_final(self):
        state = snapshots(); state["raw.adoc"]["sha256"] = "d" * 64
        with self.assertRaises(LineageError): bind(parse(normalize(wire(), state)), scope())
        events, final = normalize(wire(), snapshots()), None
        final = events.pop(5); events.insert(7, final)
        for sequence, event in enumerate(events, 1): event["sequence"] = sequence
        with self.assertRaises(LineageError): bind(parse(events), scope())


if __name__ == "__main__": unittest.main()
