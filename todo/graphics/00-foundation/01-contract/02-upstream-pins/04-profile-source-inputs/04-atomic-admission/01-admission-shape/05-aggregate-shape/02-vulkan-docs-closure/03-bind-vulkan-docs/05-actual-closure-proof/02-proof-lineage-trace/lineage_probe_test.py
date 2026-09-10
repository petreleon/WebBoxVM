#!/usr/bin/env python3
"""Unit checks for the no-privilege parent/child ptrace capability gate."""

from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from lineage_model import LineageError
from lineage_probe import CONTRACT, IMAGE, WORK_PARENT, WORK_PREFIX, command, decode, probe
from lineage_probe_paths import SOURCE, recheck, safe_dir, safe_file, work_root


def work(token: str = "unittest") -> Path:
    return WORK_PARENT / f"{WORK_PREFIX}{token}"


class LineageProbeTest(unittest.TestCase):
    def test_command_has_exact_security_prefix_without_aliases(self):
        result = command(work())
        self.assertEqual(result[:9], ["docker", "run", "--rm", "--network", "none", "--platform", "linux/amd64", "--user", "501:20"])
        self.assertIn(IMAGE, result); self.assertIn("readonly", " ".join(result))
        for flag in ("--privileged", "--cap-add", "--security-opt"):
            self.assertFalse(any(item == flag or item.startswith(f"{flag}=") for item in result))

    def test_never_accepts_a_daemon_path_mounted_available_result(self):
        available = f'{{"contract":"{CONTRACT}","status":"available","stage":"parent-fork-exec-complete","errno":0}}'
        blocked = f'{{"contract":"{CONTRACT}","status":"blocked","stage":"parent-fork-exec-complete","errno":0}}'
        with self.assertRaises(LineageError): decode(0, available)
        self.assertFalse(decode(77, blocked).available)
        blocked = f'{{"contract":"{CONTRACT}","status":"blocked","stage":"child-traceme","errno":38}}'
        self.assertEqual(decode(77, blocked).stage, "child-traceme")
        incomplete = f'{{"contract":"{CONTRACT}","status":"available","stage":"parent-syscall-entry-stop","errno":0}}'
        with self.assertRaises(LineageError): decode(0, incomplete)

    def test_gate_source_requires_syscall_direction_and_fork_exec_events(self):
        text = SOURCE.read_text(encoding="utf-8")
        for token in ("PTRACE_GET_SYSCALL_INFO", "PTRACE_SYSCALL_INFO_ENTRY", "PTRACE_SYSCALL_INFO_EXIT",
                      "PTRACE_O_TRACEFORK", "PTRACE_O_TRACEEXEC", "PTRACE_EVENT_FORK", "PTRACE_EVENT_EXEC"):
            self.assertIn(token, text)
        self.assertIn("if (write(pipefd[1], &error, sizeof(error))", text)
        self.assertNotIn("(void)write(pipefd[1]", text)

    def test_rejects_inconsistent_or_malformed_gate_results(self):
        with self.assertRaises(LineageError): decode(0, f'{{"contract":"{CONTRACT}","status":"blocked","stage":"child-traceme","errno":38}}')
        with self.assertRaises(LineageError): decode(77, "not-json")

    def test_rejects_external_invalid_and_existing_work_roots(self):
        with self.assertRaises(LineageError): command(Path("/private/tmp/webboxvm-lineage-probe"))
        with self.assertRaises(LineageError): command(WORK_PARENT)
        root = work("existing")
        if root.exists() or root.is_symlink(): self.skipTest("reserved probe name exists")
        root.mkdir(mode=0o700)
        try:
            with self.assertRaises(LineageError): probe(root)
        finally:
            root.rmdir()

    def test_rejects_a_direct_symlink_work_root(self):
        with tempfile.TemporaryDirectory() as target:
            link = work("link")
            if link.exists() or link.is_symlink(): self.skipTest("reserved probe name exists")
            os.symlink(target, link)
            try:
                with self.assertRaises(LineageError): probe(link)
            finally:
                link.unlink()

    def test_rejects_symlinked_source_and_artifact_ancestors(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "repo"; external = Path(temporary) / "external"
            root.mkdir(); external.mkdir(); (root / ".artifacts").symlink_to(external, target_is_directory=True)
            with self.assertRaises(LineageError): safe_dir(root, root / ".artifacts/graphics", "test artifact")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "repo"; external = Path(temporary) / "external"
            root.mkdir(); external.mkdir(); (root / "observer").symlink_to(external, target_is_directory=True)
            with self.assertRaises(LineageError): safe_file(root, root / "observer/lineage_ptrace_probe.c", "test source")

    def test_recheck_rejects_a_replaced_work_root(self):
        root = work("replaced")
        if root.exists() or root.is_symlink(): self.skipTest("reserved probe name exists")
        with tempfile.TemporaryDirectory() as target:
            state = work_root(root)
            try:
                root.rmdir(); os.symlink(target, root)
                with self.assertRaises(LineageError): recheck(state)
            finally:
                state.close()
                if root.is_symlink(): root.unlink()

    def test_reports_a_docker_start_failure_after_creating_only_valid_root(self):
        result = subprocess.CompletedProcess([], 125, "", "socket denied")
        root = work("mock-start")
        if root.exists() or root.is_symlink(): self.skipTest("reserved probe name exists")
        try:
            with patch("lineage_probe.subprocess.run", return_value=result):
                with self.assertRaisesRegex(LineageError, "did not start"):
                    probe(root)
            self.assertTrue(root.is_dir())
        finally:
            if root.is_dir(): root.rmdir()


if __name__ == "__main__": unittest.main()
