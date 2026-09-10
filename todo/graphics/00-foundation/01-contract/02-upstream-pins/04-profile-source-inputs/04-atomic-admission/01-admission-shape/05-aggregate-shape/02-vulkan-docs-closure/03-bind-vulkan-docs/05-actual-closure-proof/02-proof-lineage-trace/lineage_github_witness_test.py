#!/usr/bin/env python3
"""Checks for the narrow GitHub-hosted ptrace witness anchor."""

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from lineage_github_witness import Anchor, PROBE_CONTRACT, SOURCE, WitnessError, anchor, exact_source, probe, receipt


def environment(**changed: str) -> dict[str, str]:
    value = {"GITHUB_ACTIONS": "true", "GITHUB_EVENT_NAME": "push", "GITHUB_REPOSITORY": "petreleon/WebBoxVM",
             "GITHUB_REF": "refs/heads/codex/graphics-f01-baseline", "RUNNER_ENVIRONMENT": "github-hosted",
             "RUNNER_OS": "Linux", "RUNNER_ARCH": "X64", "GITHUB_SHA": "a" * 40,
             "GITHUB_WORKFLOW_SHA": "a" * 40, "GITHUB_RUN_ID": "17", "GITHUB_RUN_ATTEMPT": "1"}
    value.update(changed)
    return value


def command(root: Path, *args: str) -> str:
    return subprocess.run(["git", "-C", str(root), *args], text=True, capture_output=True, check=True).stdout.strip()


class GitHubWitnessTest(unittest.TestCase):
    def test_anchor_rejects_local_wrong_ref_and_detached_workflow(self):
        value = anchor(environment())
        self.assertEqual((value.commit, value.run_id), ("a" * 40, "17"))
        for changed in ({"GITHUB_ACTIONS": "false"}, {"GITHUB_REF": "refs/heads/main"},
                        {"GITHUB_WORKFLOW_SHA": "b" * 40}, {"RUNNER_ARCH": "ARM64"}):
            with self.assertRaises(WitnessError): anchor(environment(**changed))

    def test_exact_source_is_clean_head_blob_not_worktree_content(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); source = root / SOURCE; source.parent.mkdir(parents=True)
            command(root, "init", "-q"); command(root, "config", "user.email", "test@example.invalid")
            command(root, "config", "user.name", "test"); source.write_bytes(b"anchored source\n")
            command(root, "add", "."); command(root, "commit", "-qm", "anchor")
            commit = command(root, "rev-parse", "HEAD")
            blob, payload = exact_source(root, Anchor(commit, commit, "1", "1"))
            self.assertEqual(payload, b"anchored source\n")
            self.assertEqual(blob, command(root, "rev-parse", f"{commit}:{SOURCE}"))
            source.write_bytes(b"changed worktree\n")
            with self.assertRaises(WitnessError): exact_source(root, Anchor(commit, commit, "1", "1"))

    def test_receipt_keeps_success_as_unadmitted_observation(self):
        result = {"contract": PROBE_CONTRACT, "status": "blocked", "stage": "parent-fork-exec-complete", "errno": 0}
        value = receipt(Anchor("a" * 40, "a" * 40, "17", "1"), "b" * 40, b"source", result)
        self.assertEqual(value["status"], "observed-unadmitted")
        self.assertEqual(value["probe"], result)
        self.assertEqual(value["anchor"]["source_path"], SOURCE)

    def test_compiler_failure_has_a_bounded_diagnostic(self):
        failure = subprocess.CompletedProcess([], 1, "", "first\nsecond\nthird\nfourth")
        with patch("lineage_github_witness.subprocess.run", return_value=failure):
            with self.assertRaisesRegex(WitnessError, "second third fourth"):
                probe(b"int main(void) { return 0; }")


if __name__ == "__main__":
    unittest.main()
