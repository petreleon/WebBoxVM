#!/usr/bin/env python3
"""Emit an unadmitted pinned-image Docker ptrace primitive observation."""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

CONTRACT = "webboxvm-graphics-github-pinned-image-ptrace-primitive-witness-v1"
PROBE_CONTRACT = "webboxvm-graphics-ptrace-capability-v3"
INPUT_CONTRACT = "webboxvm-graphics-ptrace-source-envelope-v1"
REPOSITORY = "petreleon/WebBoxVM"
REF = "refs/heads/codex/graphics-f01-baseline"
BASE = "todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/02-vulkan-docs-closure/03-bind-vulkan-docs/05-actual-closure-proof/02-proof-lineage-trace"
HELPER = f"{BASE}/lineage_github_docker_witness.py"
SOURCE = f"{BASE}/observer/lineage_ptrace_probe.c"
IMAGE = "khronosgroup/docker-images@sha256:f1ca671f3bdb10ad49e238b9bf28853088a21af49504498fc9084c9b4fea4762"
SOURCE_SHA256 = "76c5fd93ca9c56fd48d8f02a037e87fa3c0d72ff13ba360ffcae44203ff9da89"
SOURCE_BYTES = 6700
SHA = re.compile(r"[0-9a-f]{40}")
STAGES = frozenset((
    "setup fork child-traceme initial-wait initial-stop parent-setoptions parent-syscall-entry "
    "parent-syscall-entry-wait parent-syscall-entry-stop parent-syscall-entry-info parent-syscall-exit "
    "parent-syscall-exit-wait parent-syscall-exit-stop parent-syscall-exit-info parent-fork parent-fork-wait "
    "parent-fork-event parent-fork-pid parent-fork-parent-cont parent-fork-child-wait parent-fork-child-stop "
    "parent-exec parent-exec-wait parent-exec-event parent-exec-cont parent-exit-wait parent-exit-cont parent-exit "
    "parent-fork-exec-complete"
).split())


class WitnessError(ValueError):
    pass


def reject(message: str) -> None:
    raise WitnessError(message)
def required(env: dict[str, str], key: str) -> str:
    value = env.get(key, "")
    if not value: reject(f"missing GitHub anchor {key}")
    return value
def anchor(env: dict[str, str]) -> dict[str, str]:
    expected = {"GITHUB_ACTIONS": "true", "GITHUB_EVENT_NAME": "push", "GITHUB_REPOSITORY": REPOSITORY,
                "GITHUB_REF": REF, "RUNNER_ENVIRONMENT": "github-hosted", "RUNNER_OS": "Linux", "RUNNER_ARCH": "X64"}
    for key, value in expected.items():
        if required(env, key) != value: reject(f"unexpected GitHub anchor {key}")
    commit, workflow = required(env, "GITHUB_SHA"), required(env, "GITHUB_WORKFLOW_SHA")
    if not SHA.fullmatch(commit) or workflow != commit: reject("workflow and checked-out commit are not one immutable SHA")
    run_id, attempt = required(env, "GITHUB_RUN_ID"), required(env, "GITHUB_RUN_ATTEMPT")
    if not run_id.isdecimal() or not attempt.isdecimal(): reject("GitHub run identity is invalid")
    return {"commit": commit, "workflow_commit": workflow, "run_id": run_id, "attempt": attempt}
def git(root: Path, *args: str, text: bool = True) -> str | bytes:
    result = subprocess.run(["git", "-C", str(root), *args], text=text, capture_output=True, check=False)
    if result.returncode: reject(f"git anchor command failed: {args[0]}")
    return result.stdout.strip() if text else result.stdout
def inputs(root: Path, value: dict[str, str], helper: Path) -> tuple[str, str, bytes]:
    if root.is_symlink() or not root.is_dir() or helper.is_symlink() or not helper.is_file():
        reject("GitHub workspace or helper is unsafe")
    if git(root, "rev-parse", "HEAD") != value["commit"] or git(root, "status", "--porcelain"):
        reject("workspace is not the clean anchored commit")
    helper_blob = git(root, "rev-parse", "--verify", f"{value['commit']}:{HELPER}")
    source_blob = git(root, "rev-parse", "--verify", f"{value['commit']}:{SOURCE}")
    if not isinstance(helper_blob, str) or not SHA.fullmatch(helper_blob) or not isinstance(source_blob, str) or not SHA.fullmatch(source_blob):
        reject("helper or source blob identity is invalid")
    if git(root, "cat-file", "blob", helper_blob, text=False) != helper.read_bytes():
        reject("executed helper does not match its anchored Git blob")
    payload = git(root, "cat-file", "blob", source_blob, text=False)
    if not isinstance(payload, bytes) or input_record(payload) != {"contract": INPUT_CONTRACT, "sha256": SOURCE_SHA256, "bytes": SOURCE_BYTES}:
        reject("source blob is not the reviewed ptrace gate")
    return helper_blob, source_blob, payload

def detail(value: str) -> str:
    return " ".join(value.split())[:320] or "Docker did not provide a diagnostic"
def input_record(payload: bytes) -> dict[str, object]:
    return {"contract": INPUT_CONTRACT, "sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload)}
def source_input(payload: bytes) -> bytes:
    value = input_record(payload)
    return f"{value['contract']} {value['bytes']} {value['sha256']}\n".encode("ascii") + payload

def image_id(value: object) -> str:
    if not isinstance(value, list) or len(value) != 1 or not isinstance(value[0], dict): reject("Docker image inspection is malformed")
    row, expected = value[0], f"khronosgroup/docker-images@{IMAGE.partition('@')[2]}"
    digests = row.get("RepoDigests")
    if row.get("Os") != "linux" or row.get("Architecture") != "amd64" or not isinstance(digests, list) or expected not in digests:
        reject("Docker did not retain the pinned linux/amd64 image digest")
    result = row.get("Id")
    if not isinstance(result, str) or not result.startswith("sha256:"): reject("Docker image has no immutable config ID")
    return result

def runtime() -> tuple[str, dict[str, str]]:
    binary = shutil.which("docker")
    if not binary: reject("Docker CLI is unavailable")
    try:
        pulled = subprocess.run([binary, "pull", "--platform", "linux/amd64", IMAGE], text=True, capture_output=True, check=False)
        version = subprocess.run([binary, "version", "--format", "{{.Client.Version}}/{{.Server.Version}}"], text=True, capture_output=True, check=False)
        inspected = subprocess.run([binary, "image", "inspect", IMAGE], text=True, capture_output=True, check=False)
    except OSError as error:
        reject(f"Docker CLI could not run: {detail(str(error))}")
    if pulled.returncode: reject(f"Docker pull failed: {detail(pulled.stderr)}")
    if version.returncode or not version.stdout.strip(): reject(f"Docker version failed: {detail(version.stderr)}")
    if inspected.returncode: reject(f"Docker image inspection failed: {detail(inspected.stderr)}")
    try: result = image_id(json.loads(inspected.stdout))
    except json.JSONDecodeError as error: reject(f"Docker image inspection is not JSON: {error}")
    return binary, {"client_server_version": version.stdout.strip(), "image_id": result}

def command(binary: str, source: Path, work: Path) -> list[str]:
    script = f"""set -eu
IFS=' ' read -r contract bytes sha
test "$contract" = '{INPUT_CONTRACT}'; test "$bytes" = '{SOURCE_BYTES}'; test "$sha" = '{SOURCE_SHA256}'
dd bs=1 count="$bytes" of=/work/probe.c 2>/dev/null
test "$(wc -c < /work/probe.c | tr -d '[:space:]')" = "$bytes"
set -- $(sha256sum /work/probe.c); test "$1" = "$sha"
printf '{{\"contract\":\"%s\",\"sha256\":\"%s\",\"bytes\":%s}}\\n' "$contract" "$sha" "$bytes"
gcc -std=c11 -O2 -Wall -Werror /work/probe.c -o /work/probe
exec /work/probe"""
    return [binary, "run", "--rm", "-i", "--pull=never", "--network", "none", "--platform", "linux/amd64", "--user", "501:20",
            "--mount", f"type=bind,src={source},dst=/vulkan,readonly", "--mount", f"type=bind,src={work},dst=/work",
            "-w", "/work", "--entrypoint", "sh", IMAGE, "-c", script]

def decoded(result: subprocess.CompletedProcess[bytes], payload: bytes) -> dict[str, object]:
    try:
        rows = result.stdout.decode("utf-8").splitlines()
        envelope, value = (json.loads(row) for row in rows)
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        reject(f"container primitive did not emit two JSON records: {error}")
    expected = input_record(payload)
    if len(rows) != 2 or envelope != expected: reject("container did not receive the reviewed ptrace source")
    if result.returncode != 77 or not isinstance(value, dict) or set(value) != {"contract", "status", "stage", "errno"}:
        reject("container probe did not fail closed")
    if value["contract"] != PROBE_CONTRACT or value["status"] != "blocked" or value["stage"] not in STAGES:
        reject("container probe has an invalid contract or stage")
    if type(value["errno"]) is not int or value["errno"] < 0: reject("container probe has an invalid errno")
    outcome = "pinned-image-ptrace-observed" if value["stage"] == "parent-fork-exec-complete" and value["errno"] == 0 else "pinned-image-ptrace-blocked"
    return {"outcome": outcome, "input": expected, "probe": value}

def execute(payload: bytes, temp: Path) -> tuple[dict[str, str], dict[str, object]]:
    binary, docker = runtime()
    with tempfile.TemporaryDirectory(prefix="webboxvm-docker-ptrace-", dir=temp) as directory:
        root = Path(directory); source, work = root / "source", root / "work"
        source.mkdir(mode=0o555); work.mkdir(mode=0o777); os.chmod(source, 0o555); os.chmod(work, 0o777)
        try: result = subprocess.run(command(binary, source, work), input=source_input(payload), capture_output=True, check=False)
        except OSError as error: reject(f"Docker primitive could not run: {detail(str(error))}")
    return docker, decoded(result, payload)

def receipt(value: dict[str, str], helper_blob: str, source_blob: str, payload: bytes, docker: dict[str, str], result: dict[str, object]) -> dict[str, object]:
    if result["outcome"] not in {"pinned-image-ptrace-observed", "pinned-image-ptrace-blocked"}: reject("container primitive has no terminal observation")
    return {"contract": CONTRACT, "status": "observed-unadmitted", "outcome": result["outcome"], "anchor": {
            "repository": REPOSITORY, "ref": REF, "commit": value["commit"], "workflow_commit": value["workflow_commit"],
            "helper_path": HELPER, "helper_blob": helper_blob, "source_path": SOURCE, "source_blob": source_blob,
            "source_sha256": SOURCE_SHA256, "source_bytes": len(payload)}, "runner": {"environment": "github-hosted",
            "os": "Linux", "arch": "X64", "kernel": " ".join(os.uname())}, "container": {"image": IMAGE,
            "platform": "linux/amd64", "network": "none", "user": "501:20", "source_mount": "/vulkan:ro",
            "work_mount": "/work:rw", "purpose": "pinned-image-ptrace-primitive-only", "image_pull_network": "host-preflight-only"},
            "docker": docker, "run": {"id": value["run_id"], "attempt": value["attempt"]}, "input": result["input"],
            "probe": result["probe"]}

def main() -> None:
    if len(sys.argv) != 2: raise SystemExit("usage: lineage_github_docker_witness.py WORKSPACE")
    try:
        value = anchor(dict(os.environ)); root = Path(sys.argv[1]); helper = Path(__file__)
        helper_blob, source_blob, payload = inputs(root, value, helper)
        docker, result = execute(payload, Path(required(dict(os.environ), "RUNNER_TEMP")))
        print(json.dumps(receipt(value, helper_blob, source_blob, payload, docker, result), sort_keys=True, separators=(",", ":")))
    except WitnessError as error:
        print(f"FAIL: {error}", file=sys.stderr); raise SystemExit(2)


if __name__ == "__main__":
    main()
