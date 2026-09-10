#!/usr/bin/env python3
"""Emit a narrow GitHub-hosted ptrace witness from immutable Git blobs."""

from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

CONTRACT = "webboxvm-graphics-github-hosted-ptrace-witness-v1"
PROBE_CONTRACT = "webboxvm-graphics-ptrace-capability-v3"
REPOSITORY = "petreleon/WebBoxVM"
REF = "refs/heads/codex/graphics-f01-baseline"
SOURCE = "todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/02-vulkan-docs-closure/03-bind-vulkan-docs/05-actual-closure-proof/02-proof-lineage-trace/observer/lineage_ptrace_probe.c"
SHA = re.compile(r"[0-9a-f]{40}")


class WitnessError(ValueError):
    pass


@dataclass(frozen=True)
class Anchor:
    commit: str
    workflow_commit: str
    run_id: str
    attempt: str


def reject(message: str) -> None:
    raise WitnessError(message)


def required(env: dict[str, str], key: str) -> str:
    value = env.get(key, "")
    if not value:
        reject(f"missing GitHub anchor {key}")
    return value


def anchor(env: dict[str, str]) -> Anchor:
    expected = {"GITHUB_ACTIONS": "true", "GITHUB_EVENT_NAME": "push", "GITHUB_REPOSITORY": REPOSITORY,
                "GITHUB_REF": REF, "RUNNER_ENVIRONMENT": "github-hosted", "RUNNER_OS": "Linux", "RUNNER_ARCH": "X64"}
    for key, value in expected.items():
        if required(env, key) != value:
            reject(f"unexpected GitHub anchor {key}")
    commit, workflow = required(env, "GITHUB_SHA"), required(env, "GITHUB_WORKFLOW_SHA")
    if not SHA.fullmatch(commit) or workflow != commit:
        reject("workflow and checked-out commit are not one immutable SHA")
    run_id, attempt = required(env, "GITHUB_RUN_ID"), required(env, "GITHUB_RUN_ATTEMPT")
    if not run_id.isdecimal() or not attempt.isdecimal():
        reject("GitHub run identity is invalid")
    return Anchor(commit, workflow, run_id, attempt)


def git_text(root: Path, *args: str) -> str:
    result = subprocess.run(["git", "-C", str(root), *args], text=True, capture_output=True, check=False)
    if result.returncode:
        reject(f"git anchor command failed: {args[0]}")
    return result.stdout.strip()


def git_blob(root: Path, blob: str) -> bytes:
    result = subprocess.run(["git", "-C", str(root), "cat-file", "blob", blob], capture_output=True, check=False)
    if result.returncode:
        reject("Git object disappeared before probe compilation")
    return result.stdout


def exact_source(root: Path, value: Anchor) -> tuple[str, bytes]:
    if root.is_symlink() or not root.is_dir():
        reject("GitHub workspace is not a direct directory")
    if git_text(root, "rev-parse", "HEAD") != value.commit or git_text(root, "status", "--porcelain"):
        reject("workspace is not the clean anchored commit")
    blob = git_text(root, "rev-parse", "--verify", f"{value.commit}:{SOURCE}")
    if not SHA.fullmatch(blob):
        reject("source blob identity is invalid")
    return blob, git_blob(root, blob)


def probe(payload: bytes) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="webboxvm-ptrace-") as directory:
        root = Path(directory); source, binary = root / "probe.c", root / "probe"
        source.write_bytes(payload)
        build = subprocess.run(["gcc", "-std=c11", "-O2", "-Wall", "-Werror", str(source), "-o", str(binary)],
                               text=True, capture_output=True, check=False)
        if build.returncode:
            reject("compiler rejected the anchored probe blob")
        result = subprocess.run([str(binary)], text=True, capture_output=True, check=False)
    try:
        value = json.loads(result.stdout.strip())
    except json.JSONDecodeError as error:
        reject(f"probe did not emit JSON: {error}")
    if result.returncode != 77 or not isinstance(value, dict):
        reject("probe did not fail closed")
    if value != {"contract": PROBE_CONTRACT, "status": "blocked", "stage": "parent-fork-exec-complete", "errno": 0}:
        reject("probe did not complete every ptrace event")
    return value


def receipt(value: Anchor, blob: str, payload: bytes, result: dict[str, object]) -> dict[str, object]:
    return {"contract": CONTRACT, "status": "observed-unadmitted", "anchor": {"repository": REPOSITORY,
            "ref": REF, "commit": value.commit, "workflow_commit": value.workflow_commit, "source_path": SOURCE,
            "source_blob": blob, "source_sha256": hashlib.sha256(payload).hexdigest(), "source_bytes": len(payload)},
            "runner": {"environment": "github-hosted", "os": "Linux", "arch": "X64"},
            "run": {"id": value.run_id, "attempt": value.attempt}, "probe": result}


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: lineage_github_witness.py WORKSPACE")
    try:
        value = anchor(dict(os.environ)); blob, payload = exact_source(Path(sys.argv[1]), value)
        print(json.dumps(receipt(value, blob, payload, probe(payload)), sort_keys=True, separators=(",", ":")))
    except WitnessError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
