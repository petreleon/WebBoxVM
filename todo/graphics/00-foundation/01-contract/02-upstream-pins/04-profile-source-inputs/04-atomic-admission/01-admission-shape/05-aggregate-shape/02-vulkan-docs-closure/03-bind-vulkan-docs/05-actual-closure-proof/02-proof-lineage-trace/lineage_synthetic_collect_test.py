#!/usr/bin/env python3
"""Keep fixture pipe and post-exit snapshot boundaries fail-closed."""

from __future__ import annotations

import json
import unittest

from lineage_model import LineageError
from lineage_synthetic_collect import collect, decoded

ARGV = "797b8f932ac58c625cfb33c986ba1e4b5eb471598cb3847781a4eae40f8c21fb"


def wire() -> list[dict[str, object]]:
    return [
        {"kind": "process", "pid": 10, "parent": None, "argv_sha256": ARGV}, {"kind": "exec", "pid": 10, "argv_sha256": ARGV},
        {"kind": "open", "pid": 10, "fd": 3, "path": "/vulkan/raw.adoc", "mode": "read"}, {"kind": "read", "pid": 10, "fd": 3}, {"kind": "close", "pid": 10, "fd": 3},
        {"kind": "open", "pid": 10, "fd": 3, "path": "/work/temporary/out.tmp", "mode": "write"}, {"kind": "write", "pid": 10, "fd": 3}, {"kind": "close", "pid": 10, "fd": 3},
        {"kind": "rename", "pid": 10, "old": "/work/temporary/out.tmp", "new": "/work/generated/out.adoc"},
        {"kind": "open", "pid": 10, "fd": 3, "path": "/work/generated/out.adoc", "mode": "read"}, {"kind": "read", "pid": 10, "fd": 3}, {"kind": "close", "pid": 10, "fd": 3},
        {"kind": "process", "pid": 11, "parent": 10, "argv_sha256": ARGV}, {"kind": "exec", "pid": 11, "argv_sha256": ARGV}, {"kind": "exit", "pid": 11}, {"kind": "exit", "pid": 10},
    ]


def pipe(rows=None, terminal=None, snapshots=None) -> bytes:
    rows = wire() if rows is None else rows
    terminal = {"kind": "terminal", "status": "observed-unadmitted"} if terminal is None else terminal
    snapshots = [{"kind": "snapshot", "path": "/vulkan/raw.adoc", "hex": "726177"},
                 {"kind": "snapshot", "path": "/work/generated/out.adoc", "hex": "726177"}] if snapshots is None else snapshots
    return b"\n".join(json.dumps(row, separators=(",", ":")).encode() for row in (*rows, *snapshots, terminal)) + b"\n"


class SyntheticCollectTest(unittest.TestCase):
    def test_pipe_snapshot_normalize_and_bind_one_fixture_output(self):
        observation = collect(pipe())
        self.assertEqual((observation.contract, observation.status), ("webboxvm-graphics-synthetic-ptrace-observation-v1", "observed-unadmitted"))
        self.assertEqual([(item.selector, item.writer_instance, item.producer_ids) for item in observation.records],
                         [("generated/out.adoc", "p-10-2", ("raw-7261772e61646f63",))])

    def test_rejects_missing_or_forged_terminal_rows(self):
        with self.assertRaises(LineageError): decoded(pipe(terminal={"kind": "terminal", "status": "blocked"}))
        with self.assertRaises(LineageError): decoded(pipe(wire() + [{"kind": "terminal", "status": "observed-unadmitted"}]))
        with self.assertRaises(LineageError): decoded(b'{"kind":"process","kind":"exit"}\n')

    def test_snapshot_rows_reject_mismatch_escape_and_wire_after_snapshot(self):
        mismatch = [{"kind": "snapshot", "path": "/vulkan/raw.adoc", "hex": "726177"},
                    {"kind": "snapshot", "path": "/work/generated/out.adoc", "hex": "626164"}]
        with self.assertRaises(LineageError): decoded(pipe(snapshots=mismatch))
        with self.assertRaises(LineageError): decoded(pipe(snapshots=[{"kind": "snapshot", "path": "/etc/passwd", "hex": "726177"}]))
        rows = wire() + [{"kind": "snapshot", "path": "/vulkan/raw.adoc", "hex": "726177"}, {"kind": "exit", "pid": 99}]
        with self.assertRaises(LineageError): decoded(pipe(rows=rows, snapshots=[{"kind": "snapshot", "path": "/work/generated/out.adoc", "hex": "726177"}]))

    def test_collect_rejects_a_hand_authored_nonfixture_lifecycle(self):
        rows = wire(); rows[1], rows[2] = rows[2], rows[1]
        with self.assertRaises(LineageError): collect(pipe(rows=rows))

    def test_collect_rejects_noncanonical_fixture_fields(self):
        for index, field, value in ((2, "fd", 4), (5, "path", "/work/temporary/other"), (5, "mode", "read"), (1, "argv_sha256", "a" * 64)):
            rows = wire(); rows[index][field] = value
            with self.assertRaises(LineageError): collect(pipe(rows=rows))


if __name__ == "__main__": unittest.main()
