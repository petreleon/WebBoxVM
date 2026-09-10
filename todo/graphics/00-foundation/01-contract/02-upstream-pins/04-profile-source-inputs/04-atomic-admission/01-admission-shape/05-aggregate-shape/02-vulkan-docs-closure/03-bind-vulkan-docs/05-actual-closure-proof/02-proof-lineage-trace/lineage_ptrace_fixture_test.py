#!/usr/bin/env python3
"""Guard the source-level isolation contract until the hosted witness executes it."""

from __future__ import annotations

import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
OBSERVER = HERE / "observer"


class PtraceFixtureTest(unittest.TestCase):
    def test_tracked_raw_hex_decodes_to_the_exact_fixture_bytes(self):
        payload = (OBSERVER / "lineage_ptrace_fixture.raw.hex").read_bytes()
        self.assertEqual((payload, bytes.fromhex(payload.decode("ascii").strip())), (b"726177\n", b"raw"))

    def test_collector_requires_child_traceme_directional_syscalls_and_safe_snapshot(self):
        text = (OBSERVER / "lineage_ptrace_fixture_collector.c").read_text(encoding="utf-8")
        for token in ("PTRACE_TRACEME", "PTRACE_GET_SYSCALL_INFO", "PTRACE_O_TRACEFORK", "PTRACE_O_EXITKILL", "PTRACE_EVENT_FORK", "PTRACE_EVENT_EXEC", "lpc_known", "multiple-child-stops", "dup2(null, STDIN_FILENO)", "PR_SET_DUMPABLE", "pre-run-snapshot", "post-exit-snapshot"):
            self.assertIn(token, text)
        self.assertLess(text.index("snapshot(\"/vulkan/raw.adoc\", &raw)"), text.index("observed-unadmitted"))
        self.assertNotIn("LD_PRELOAD", text)

    def test_fixture_and_state_cover_only_the_bounded_copy_flow(self):
        fixture = (OBSERVER / "lineage_ptrace_fixture.c").read_text(encoding="utf-8")
        state = (OBSERVER / "lineage_ptrace_state.c").read_text(encoding="utf-8")
        for token in ("SYS_openat", "SYS_read", "SYS_write", "SYS_close", "SYS_renameat", "O_NOFOLLOW", "fork()", "execl"):
            self.assertIn(token, fixture)
        for token in ("PTRACE_PEEKDATA", "LPC_PATH", "LPC_FDS", "SYS_openat", "SYS_renameat", "fixture_path", "O_NOFOLLOW", "args[2] != 64", "args[2] != 3", "result != 3", "live(item) || item->pending.kind"):
            self.assertIn(token, state)
        self.assertNotIn("system(", fixture)


if __name__ == "__main__": unittest.main()
