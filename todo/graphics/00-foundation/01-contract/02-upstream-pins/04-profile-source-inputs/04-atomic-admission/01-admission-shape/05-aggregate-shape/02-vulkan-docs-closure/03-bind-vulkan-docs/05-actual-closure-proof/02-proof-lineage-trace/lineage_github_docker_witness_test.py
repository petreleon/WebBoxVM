#!/usr/bin/env python3
"""Checks for the pinned-image Docker ptrace primitive witness."""

from __future__ import annotations

import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from lineage_github_docker_witness import (
    IMAGE, INPUT_CONTRACT, PROBE_CONTRACT, SOURCE_BYTES, SOURCE_SHA256, WitnessError, anchor, command, decoded,
    execute, image_id, input_record, receipt, runtime, source_input,
)


def environment(**changed: str) -> dict[str, str]:
    value = {"GITHUB_ACTIONS": "true", "GITHUB_EVENT_NAME": "push", "GITHUB_REPOSITORY": "petreleon/WebBoxVM",
             "GITHUB_REF": "refs/heads/codex/graphics-f01-baseline", "RUNNER_ENVIRONMENT": "github-hosted",
             "RUNNER_OS": "Linux", "RUNNER_ARCH": "X64", "GITHUB_SHA": "a" * 40,
             "GITHUB_WORKFLOW_SHA": "a" * 40, "GITHUB_RUN_ID": "17", "GITHUB_RUN_ATTEMPT": "1"}
    value.update(changed)
    return value


def completed(stage: str, number: int, payload: bytes, code: int = 77, envelope: object = None) -> subprocess.CompletedProcess[bytes]:
    source = input_record(payload) if envelope is None else envelope
    probe = {"contract": PROBE_CONTRACT, "status": "blocked", "stage": stage, "errno": number}
    output = b"\n".join(json.dumps(item, separators=(",", ":")).encode() for item in (source, probe))
    return subprocess.CompletedProcess([], code, output, b"")


def inspected(digests: object = None) -> list[dict[str, object]]:
    digest = f"khronosgroup/docker-images@{IMAGE.partition('@')[2]}"
    return [{"Os": "linux", "Architecture": "amd64", "RepoDigests": [digest] if digests is None else digests, "Id": "sha256:config"}]


class DockerPrimitiveWitnessTest(unittest.TestCase):
    def test_anchor_rejects_local_wrong_ref_and_detached_workflow(self):
        self.assertEqual(anchor(environment())["run_id"], "17")
        for changed in ({"GITHUB_ACTIONS": "false"}, {"GITHUB_REF": "refs/heads/main"},
                        {"GITHUB_WORKFLOW_SHA": "b" * 40}, {"RUNNER_ARCH": "ARM64"}):
            with self.assertRaises(WitnessError): anchor(environment(**changed))

    def test_source_constants_and_envelope_are_exact(self):
        payload = (Path(__file__).parent / "observer/lineage_ptrace_probe.c").read_bytes()
        self.assertEqual((len(payload), hashlib.sha256(payload).hexdigest()), (SOURCE_BYTES, SOURCE_SHA256))
        self.assertEqual(source_input(payload), f"{INPUT_CONTRACT} {SOURCE_BYTES} {SOURCE_SHA256}\n".encode() + payload)

    def test_command_keeps_the_no_privilege_pinned_image_policy(self):
        value = command("docker", Path("/tmp/source"), Path("/tmp/work")); script = value[-1]
        self.assertEqual(value.count("--pull=never"), 1)
        self.assertEqual((value[value.index("--network") + 1], value[value.index("--platform") + 1]), ("none", "linux/amd64"))
        self.assertEqual(value[value.index("--user") + 1], "501:20")
        self.assertIn("type=bind,src=/tmp/source,dst=/vulkan,readonly", value)
        self.assertIn("type=bind,src=/tmp/work,dst=/work", value)
        self.assertEqual(value[value.index("--entrypoint") + 2], IMAGE)
        self.assertFalse(any(item == "--privileged" or item.startswith(("--cap-add", "--security-opt")) for item in value))
        for marker in (INPUT_CONTRACT, SOURCE_SHA256, str(SOURCE_BYTES), "sha256sum", "gcc "):
            self.assertIn(marker, script)
        self.assertLess(script.index("read -r"), script.index("gcc "))
        self.assertLess(script.index("sha256sum"), script.index("gcc "))

    def test_decode_rejects_payload_tampering_and_never_marks_available(self):
        payload = b"source"
        observed = decoded(completed("parent-fork-exec-complete", 0, payload), payload)
        blocked = decoded(completed("child-traceme", 38, payload), payload)
        self.assertEqual((observed["outcome"], blocked["outcome"]), ("pinned-image-ptrace-observed", "pinned-image-ptrace-blocked"))
        forged = dict(input_record(payload)); forged["sha256"] = "0" * 64
        with self.assertRaises(WitnessError): decoded(completed("child-traceme", 38, payload, envelope=forged), payload)
        with self.assertRaises(WitnessError): decoded(completed("parent-fork-exec-complete", 0, payload, code=0), payload)

    def test_image_identity_is_exact_and_runtime_failures_are_not_observations(self):
        self.assertEqual(image_id(inspected()), "sha256:config")
        spoof = f"evil/khronosgroup/docker-images@{IMAGE.partition('@')[2]}"
        with self.assertRaises(WitnessError): image_id(inspected([spoof]))
        with patch("lineage_github_docker_witness.shutil.which", return_value=None):
            with self.assertRaises(WitnessError): runtime()
        failed = subprocess.CompletedProcess([], 1, "", "pull denied")
        with patch("lineage_github_docker_witness.shutil.which", return_value="docker"), \
             patch("lineage_github_docker_witness.subprocess.run", return_value=failed):
            with self.assertRaises(WitnessError): runtime()

    def test_container_launch_failure_cannot_create_a_receipt(self):
        payload = b"source"; failed = subprocess.CompletedProcess([], 125, b"", b"daemon refused")
        with tempfile.TemporaryDirectory() as directory, patch("lineage_github_docker_witness.runtime", return_value=("docker", {})), \
             patch("lineage_github_docker_witness.subprocess.run", return_value=failed) as run:
            with self.assertRaises(WitnessError): execute(payload, Path(directory))
        self.assertEqual(run.call_args.kwargs["input"], source_input(payload))

    def test_receipt_is_unadmitted_and_rejects_non_observation(self):
        value = anchor(environment()); payload = b"source"; result = decoded(completed("parent-fork-exec-complete", 0, payload), payload)
        observed = receipt(value, "b" * 40, "c" * 40, payload, {"image_id": "sha256:x"}, result)
        self.assertEqual((observed["status"], observed["outcome"]), ("observed-unadmitted", "pinned-image-ptrace-observed"))
        self.assertEqual(observed["container"]["purpose"], "pinned-image-ptrace-primitive-only")
        with self.assertRaises(WitnessError): receipt(value, "b" * 40, "c" * 40, payload, {}, {"outcome": "runtime-unavailable"})


if __name__ == "__main__":
    unittest.main()
