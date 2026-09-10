#!/usr/bin/env python3
"""Checks for the hosted synthetic ptrace fixture witness."""

from __future__ import annotations

import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from lineage_github_synthetic_witness import (
    ARGV, BASE, HELPER, IMAGE, INPUT_CONTRACT, RAW, RAW_HEX, SPECS, WitnessError, anchor, command, decoded, execute,
    image_id, input_record, inputs, receipt, runtime, source_input,
)


def environment(**changed: str) -> dict[str, str]:
    value = {"GITHUB_ACTIONS": "true", "GITHUB_EVENT_NAME": "push", "GITHUB_REPOSITORY": "petreleon/WebBoxVM",
             "GITHUB_REF": "refs/heads/codex/graphics-f01-baseline", "RUNNER_ENVIRONMENT": "github-hosted",
             "RUNNER_OS": "Linux", "RUNNER_ARCH": "X64", "GITHUB_SHA": "a" * 40, "GITHUB_WORKFLOW_SHA": "a" * 40,
             "GITHUB_RUN_ID": "17", "GITHUB_RUN_ATTEMPT": "1"}
    value.update(changed)
    return value


def wire(parent: int = 10, child: int = 11) -> list[dict[str, object]]:
    return [
        {"kind": "process", "pid": parent, "parent": None, "argv_sha256": ARGV}, {"kind": "exec", "pid": parent, "argv_sha256": ARGV},
        {"kind": "open", "pid": parent, "fd": 3, "path": "/vulkan/raw.adoc", "mode": "read"}, {"kind": "read", "pid": parent, "fd": 3}, {"kind": "close", "pid": parent, "fd": 3},
        {"kind": "open", "pid": parent, "fd": 3, "path": "/work/temporary/out.tmp", "mode": "write"}, {"kind": "write", "pid": parent, "fd": 3}, {"kind": "close", "pid": parent, "fd": 3},
        {"kind": "rename", "pid": parent, "old": "/work/temporary/out.tmp", "new": "/work/generated/out.adoc"}, {"kind": "open", "pid": parent, "fd": 3, "path": "/work/generated/out.adoc", "mode": "read"}, {"kind": "read", "pid": parent, "fd": 3}, {"kind": "close", "pid": parent, "fd": 3},
        {"kind": "process", "pid": child, "parent": parent, "argv_sha256": ARGV}, {"kind": "exec", "pid": child, "argv_sha256": ARGV}, {"kind": "exit", "pid": child}, {"kind": "exit", "pid": parent},
        {"kind": "snapshot", "path": "/vulkan/raw.adoc", "hex": RAW.hex()}, {"kind": "snapshot", "path": "/work/generated/out.adoc", "hex": RAW.hex()}, {"kind": "terminal", "status": "observed-unadmitted"},
    ]


def completed(rows: list[dict[str, object]] | None = None, code: int = 0, stderr: bytes = b"source-envelope-verified\n") -> subprocess.CompletedProcess[bytes]:
    output = b"\n".join(json.dumps(row, separators=(",", ":")).encode() for row in (wire() if rows is None else rows)) + b"\n"
    return subprocess.CompletedProcess([], code, output, stderr)


def inspected(digests: object = None) -> list[dict[str, object]]:
    digest = f"khronosgroup/docker-images@{IMAGE.partition('@')[2]}"
    return [{"Os": "linux", "Architecture": "amd64", "RepoDigests": [digest] if digests is None else digests, "Id": "sha256:config"}]


def git(root: Path, *args: str) -> str:
    return subprocess.run(["git", "-C", str(root), *args], text=True, capture_output=True, check=True).stdout.strip()


def payloads() -> tuple[bytes, ...]:
    return tuple((Path(__file__).parent / "observer" / name).read_bytes() for name, _, _, _ in SPECS)


class GitHubSyntheticWitnessTest(unittest.TestCase):
    def test_anchor_rejects_local_wrong_ref_and_detached_workflow(self):
        self.assertEqual(anchor(environment())["run_id"], "17")
        for changed in ({"GITHUB_ACTIONS": "false"}, {"GITHUB_REF": "refs/heads/main"}, {"GITHUB_WORKFLOW_SHA": "b" * 40}, {"RUNNER_ARCH": "ARM64"}):
            with self.assertRaises(WitnessError): anchor(environment(**changed))

    def test_fixed_inputs_include_the_independently_anchored_three_byte_raw_fixture(self):
        for (_, path, size, digest), payload in zip(SPECS, payloads()):
            self.assertEqual((len(payload), hashlib.sha256(payload).hexdigest()), (size, digest), path)
        self.assertEqual((SPECS[-1][0], payloads()[-1], bytes.fromhex(payloads()[-1].decode().strip())), ("lineage_ptrace_fixture.raw.hex", RAW_HEX, RAW))

    def test_stdin_envelope_and_command_keep_the_pinned_unprivileged_policy(self):
        sent, value = payloads(), command("docker", Path("/tmp/vulkan"), Path("/tmp/work")); script = value[-1]
        envelope = source_input(sent); self.assertTrue(envelope.startswith(f"{INPUT_CONTRACT} {len(SPECS)}\n".encode()))
        self.assertEqual(value.count("--pull=never"), 1); self.assertIn("-i", value)
        self.assertEqual((value[value.index("--network") + 1], value[value.index("--platform") + 1], value[value.index("--user") + 1]), ("none", "linux/amd64", "501:20"))
        self.assertIn("type=bind,src=/tmp/vulkan,dst=/vulkan,readonly", value); self.assertIn("type=bind,src=/tmp/work,dst=/work", value)
        self.assertFalse(any(item == "--privileged" or item.startswith(("--cap-add", "--security-opt")) for item in value))
        for name, _, size, digest in SPECS:
            self.assertIn(f"{name} {size} {digest}\n".encode(), envelope); self.assertIn(name, script); self.assertIn(digest, script)
        self.assertLess(script.index("read -r"), script.index("gcc ")); self.assertIn("source-envelope-verified", script); self.assertIn("/work/trailer", script)
        self.assertIn("exec /work/collector /work/fixture", script); self.assertNotIn("> /vulkan/raw.adoc", script)

    def test_decoded_requires_zero_the_exact_nineteen_rows_and_envelope_attestation(self):
        value = decoded(completed())
        self.assertEqual((value["rows"], value["records"][-1]), (19, {"kind": "terminal", "status": "observed-unadmitted"}))
        altered = wire(); altered[16]["hex"] = "626164"
        blocked = subprocess.CompletedProcess([], 77, b'{"kind":"terminal","status":"blocked","stage":"syscall-entry"}\n', b"source-envelope-verified\n")
        with self.assertRaisesRegex(WitnessError, "syscall-entry"): decoded(blocked)
        for result in (completed(altered), completed(code=77), completed(stderr=b""), completed(wire()[:-1])):
            with self.assertRaises(WitnessError): decoded(result)
        duplicate = completed().stdout.splitlines(); duplicate[0] = b'{"kind":"process","kind":"process","pid":10,"parent":null,"argv_sha256":"' + ARGV.encode() + b'"}'
        with self.assertRaises(WitnessError): decoded(subprocess.CompletedProcess([], 0, b"\n".join(duplicate) + b"\n", b"source-envelope-verified\n"))

    def test_inputs_anchor_self_and_every_transmitted_blob(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "workspace"; root.mkdir(); anchored = root / HELPER; anchored.parent.mkdir(parents=True); anchored.write_bytes(b"anchored helper\n")
            helper = Path(directory) / "runner-temp-helper.py"; helper.write_bytes(b"anchored helper\n")
            test_payloads = (b"header\n", b"state\n", b"fixture\n", b"collector\n", RAW_HEX)
            specs = tuple((name, f"{BASE}/observer/{name}", len(payload), hashlib.sha256(payload).hexdigest()) for (name, _, _, _), payload in zip(SPECS, test_payloads))
            for (_, path, _, _), payload in zip(specs, test_payloads): location = root / path; location.parent.mkdir(parents=True, exist_ok=True); location.write_bytes(payload)
            git(root, "init", "-q"); git(root, "config", "user.email", "test@example.invalid"); git(root, "config", "user.name", "test"); git(root, "add", "."); git(root, "commit", "-qm", "anchor")
            commit = git(root, "rev-parse", "HEAD"); value = {"commit": commit, "workflow_commit": commit, "run_id": "1", "attempt": "1"}
            with patch("lineage_github_synthetic_witness.SPECS", specs):
                helper_blob, blobs, received = inputs(root, value, helper)
                self.assertEqual((len(blobs), received, helper_blob), (5, test_payloads, git(root, "rev-parse", f"{commit}:{HELPER}")))
                helper.write_bytes(b"modified helper\n")
                with self.assertRaises(WitnessError): inputs(root, value, helper)

    def test_runtime_and_execute_fail_closed_without_a_valid_terminal_receipt(self):
        self.assertEqual(image_id(inspected()), "sha256:config")
        with self.assertRaises(WitnessError): image_id(inspected(["evil/digest"]))
        with patch("lineage_github_synthetic_witness.shutil.which", return_value=None):
            with self.assertRaises(WitnessError): runtime()
        sent = payloads()
        with tempfile.TemporaryDirectory() as directory, patch("lineage_github_synthetic_witness.runtime", return_value=("docker", {})), patch("lineage_github_synthetic_witness.subprocess.run", return_value=completed()) as run:
            self.assertEqual(execute(sent, Path(directory))[1]["rows"], 19)
        self.assertEqual(run.call_args.kwargs["input"], source_input(sent))
        with tempfile.TemporaryDirectory() as directory, patch("lineage_github_synthetic_witness.runtime", return_value=("docker", {})), patch("lineage_github_synthetic_witness.subprocess.run", return_value=completed(code=1)):
            with self.assertRaises(WitnessError): execute(sent, Path(directory))

    def test_receipt_remains_fixture_only_and_unadmitted(self):
        sent, collector, value = payloads(), decoded(completed()), anchor(environment())
        record = receipt(value, "b" * 40, [{"blob": "c" * 40}], sent, {"image_id": "sha256:x"}, collector)
        self.assertEqual((record["status"], record["scope"], record["collector"]["rows"]), ("observed-unadmitted", "synthetic-fixture-only-not-docs-lineage", 19))
        self.assertEqual(record["container"]["stdin"], "reviewed-source-envelope")


if __name__ == "__main__": unittest.main()
