#!/usr/bin/env python3
"""Witness one pinned-image synthetic ptrace fixture run on GitHub-hosted x64."""

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

BASE = "todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/02-vulkan-docs-closure/03-bind-vulkan-docs/05-actual-closure-proof/02-proof-lineage-trace"
CONTRACT = "webboxvm-graphics-github-synthetic-ptrace-witness-v1"
INPUT_CONTRACT = "webboxvm-graphics-synthetic-ptrace-source-envelope-v1"
REPOSITORY, REF = "petreleon/WebBoxVM", "refs/heads/codex/graphics-f01-baseline"
HELPER = f"{BASE}/lineage_github_synthetic_witness.py"
IMAGE = "khronosgroup/docker-images@sha256:f1ca671f3bdb10ad49e238b9bf28853088a21af49504498fc9084c9b4fea4762"
SPECS = (
    ("lineage_ptrace_state.h", f"{BASE}/observer/lineage_ptrace_state.h", 1163, "fa5ea08f44c2446c0904468ab0545e54a8e13b5a23096180e586499fff2ea6c5"),
    ("lineage_ptrace_state.c", f"{BASE}/observer/lineage_ptrace_state.c", 9016, "7c3395a55c69a609f407cd1e4c59a0216fc7fb4233120b006cb4d6da1c180555"),
    ("lineage_ptrace_fixture.c", f"{BASE}/observer/lineage_ptrace_fixture.c", 1577, "9c0dfa754641e8ac7b4559c8e0b6d642f088979535230c4ad63f019313b27e7a"),
    ("lineage_ptrace_fixture_collector.c", f"{BASE}/observer/lineage_ptrace_fixture_collector.c", 6797, "6b62fb8551ae680e45ed0ee21f5dd66d88eb685d6e59426f5ccf0d6f49c40813"),
    ("lineage_ptrace_fixture.raw.hex", f"{BASE}/observer/lineage_ptrace_fixture.raw.hex", 7, "4447db2af9610585bd237a01f1f8adc303015710968fc7db64106a12d886153d"),
)
RAW_HEX, RAW = b"726177\n", b"raw"
ARGV = "797b8f932ac58c625cfb33c986ba1e4b5eb471598cb3847781a4eae40f8c21fb"
SHA = re.compile(r"[0-9a-f]{40}")
class WitnessError(ValueError): pass
def reject(message: str) -> None: raise WitnessError(message)
def required(env: dict[str, str], key: str) -> str:
    value = env.get(key, "")
    if not value: reject(f"missing GitHub anchor {key}")
    return value
def anchor(env: dict[str, str]) -> dict[str, str]:
    expected = {"GITHUB_ACTIONS": "true", "GITHUB_EVENT_NAME": "push", "GITHUB_REPOSITORY": REPOSITORY, "GITHUB_REF": REF,
                "RUNNER_ENVIRONMENT": "github-hosted", "RUNNER_OS": "Linux", "RUNNER_ARCH": "X64"}
    for key, value in expected.items():
        if required(env, key) != value: reject(f"unexpected GitHub anchor {key}")
    commit, workflow = required(env, "GITHUB_SHA"), required(env, "GITHUB_WORKFLOW_SHA")
    if not SHA.fullmatch(commit) or workflow != commit: reject("workflow and checked-out commit are not one immutable SHA")
    run_id, attempt = required(env, "GITHUB_RUN_ID"), required(env, "GITHUB_RUN_ATTEMPT")
    if not run_id.isdecimal() or not attempt.isdecimal(): reject("GitHub run identity is invalid")
    return {"commit": commit, "workflow_commit": workflow, "run_id": run_id, "attempt": attempt}
def git(root: Path, *args: str, binary: bool = False):
    result = subprocess.run(["git", "-C", str(root), *args], text=not binary, capture_output=True, check=False)
    if result.returncode: reject(f"git anchor command failed: {args[0]}")
    return result.stdout if binary else result.stdout.strip()
def input_record(payloads: tuple[bytes, ...]) -> list[dict[str, object]]:
    return [{"path": path, "sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload)}
            for (_, path, _, _), payload in zip(SPECS, payloads)]

def raw_value(payload: bytes) -> bytes:
    try: value = bytes.fromhex(payload.decode("ascii").strip())
    except (UnicodeDecodeError, ValueError): reject("raw fixture hex is malformed")
    if value != RAW: reject("raw fixture is not the anchored three-byte value")
    return value

def inputs(root: Path, value: dict[str, str], helper: Path) -> tuple[str, list[dict[str, object]], tuple[bytes, ...]]:
    if root.is_symlink() or not root.is_dir() or helper.is_symlink() or not helper.is_file():
        reject("GitHub workspace or helper is unsafe")
    if git(root, "rev-parse", "HEAD") != value["commit"] or git(root, "status", "--porcelain"): reject("workspace is not the clean anchored commit")
    helper_blob = git(root, "rev-parse", "--verify", f"{value['commit']}:{HELPER}")
    if not SHA.fullmatch(helper_blob) or git(root, "cat-file", "blob", helper_blob, binary=True) != helper.read_bytes():
        reject("executed helper does not match its anchored Git blob")
    blobs, payloads = [], []
    for _, path, size, digest in SPECS:
        location = root / path
        if location.is_symlink() or not location.is_file(): reject("synthetic fixture input is unsafe")
        blob = git(root, "rev-parse", "--verify", f"{value['commit']}:{path}")
        payload = git(root, "cat-file", "blob", blob, binary=True)
        if not SHA.fullmatch(blob) or len(payload) != size or hashlib.sha256(payload).hexdigest() != digest or payload != location.read_bytes():
            reject("synthetic fixture input is not its reviewed Git blob")
        blobs.append({"path": path, "blob": blob, "sha256": digest, "bytes": size}); payloads.append(payload)
    if payloads[-1] != RAW_HEX: reject("raw fixture hex source is not reviewed")
    raw_value(payloads[-1])
    return helper_blob, blobs, tuple(payloads)
def source_input(payloads: tuple[bytes, ...]) -> bytes:
    rows = [f"{INPUT_CONTRACT} {len(SPECS)}\n".encode("ascii")]
    for (name, _, size, digest), payload in zip(SPECS, payloads): rows.extend((f"{name} {size} {digest}\n".encode("ascii"), payload))
    return b"".join(rows)


def detail(value: str) -> str: return " ".join(value.split())[:320] or "Docker did not provide a diagnostic"
def image_id(value: object) -> str:
    expected = f"khronosgroup/docker-images@{IMAGE.partition('@')[2]}"
    if not isinstance(value, list) or len(value) != 1 or not isinstance(value[0], dict): reject("Docker image inspection is malformed")
    row, digests, result = value[0], value[0].get("RepoDigests"), value[0].get("Id")
    if row.get("Os") != "linux" or row.get("Architecture") != "amd64" or not isinstance(digests, list) or expected not in digests or not isinstance(result, str) or not result.startswith("sha256:"):
        reject("Docker did not retain the pinned linux/amd64 image digest")
    return result


def runtime() -> tuple[str, dict[str, str]]:
    binary = shutil.which("docker")
    if not binary: reject("Docker CLI is unavailable")
    try:
        pulled = subprocess.run([binary, "pull", "--platform", "linux/amd64", IMAGE], text=True, capture_output=True, check=False)
        version = subprocess.run([binary, "version", "--format", "{{.Client.Version}}/{{.Server.Version}}"], text=True, capture_output=True, check=False)
        inspected = subprocess.run([binary, "image", "inspect", IMAGE], text=True, capture_output=True, check=False)
    except OSError as error: reject(f"Docker CLI could not run: {detail(str(error))}")
    if pulled.returncode: reject(f"Docker pull failed: {detail(pulled.stderr)}")
    if version.returncode or not version.stdout.strip(): reject(f"Docker version failed: {detail(version.stderr)}")
    if inspected.returncode: reject(f"Docker image inspection failed: {detail(inspected.stderr)}")
    try: result = image_id(json.loads(inspected.stdout))
    except json.JSONDecodeError as error: reject(f"Docker image inspection is not JSON: {error}")
    return binary, {"client_server_version": version.stdout.strip(), "image_id": result}


def command(binary: str, raw: Path, work: Path) -> list[str]:
    checks = "\n".join("IFS=' ' read -r name bytes sha rest\ntest \"$name\" = '" + name + "'; test \"$bytes\" = '" + str(size) + "'; test \"$sha\" = '" + digest + "'; test -z \"$rest\"\ndd bs=1 count=\"$bytes\" of=\"/work/$name\" 2>/dev/null\ntest \"$(wc -c < \"/work/$name\" | tr -d '[:space:]')\" = \"$bytes\"\nset -- $(sha256sum \"/work/$name\"); test \"$1\" = \"$sha\"; test \"$2\" = \"/work/$name\"" for name, _, size, digest in SPECS)
    script = f"""set -eu
IFS=' ' read -r contract count rest
test \"$contract\" = '{INPUT_CONTRACT}'; test \"$count\" = '{len(SPECS)}'; test -z \"$rest\"
{checks}
dd bs=1 count=1 of=/work/trailer 2>/dev/null; test ! -s /work/trailer
test \"$(cat /work/{SPECS[-1][0]})\" = 726177
test \"$(sha256sum /vulkan/raw.adoc | cut -d ' ' -f1)\" = '{hashlib.sha256(RAW).hexdigest()}'
mkdir /work/temporary /work/generated; chmod 0777 /work/temporary /work/generated
printf '%s\\n' source-envelope-verified >&2
gcc -std=c11 -O2 -Wall -Werror /work/lineage_ptrace_state.c /work/lineage_ptrace_fixture_collector.c -o /work/collector
gcc -std=c11 -O2 -Wall -Werror /work/lineage_ptrace_fixture.c -o /work/fixture
exec /work/collector /work/fixture"""
    return [binary, "run", "--rm", "-i", "--pull=never", "--network", "none", "--platform", "linux/amd64", "--user", "501:20", "--mount", f"type=bind,src={raw},dst=/vulkan,readonly", "--mount", f"type=bind,src={work},dst=/work", "-w", "/work", "--entrypoint", "sh", IMAGE, "-c", script]


def unique(items):
    value = dict(items)
    if len(value) != len(items): reject("collector JSON repeats a key")
    return value


def decoded(result: subprocess.CompletedProcess[bytes]) -> dict[str, object]:
    if result.returncode != 0: reject(f"synthetic collector did not return zero: {result.returncode}")
    if result.stderr != b"source-envelope-verified\n" or not isinstance(result.stdout, bytes) or not result.stdout.endswith(b"\n"):
        reject("container did not attest the source envelope")
    try: rows = [json.loads(line.decode("utf-8"), object_pairs_hook=unique, parse_constant=lambda _: reject("collector JSON is nonfinite")) for line in result.stdout.splitlines()]
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error: reject(f"collector output is not JSONL: {error}")
    if len(rows) != 19 or any(not isinstance(row, dict) for row in rows): reject("collector output does not contain exactly nineteen object rows")
    parent, child = rows[0].get("pid"), rows[12].get("pid")
    if type(parent) is not int or parent < 1 or type(child) is not int or child < 1 or child == parent: reject("collector output has invalid fixture pids")
    expected = ({"kind": "process", "pid": parent, "parent": None, "argv_sha256": ARGV}, {"kind": "exec", "pid": parent, "argv_sha256": ARGV},
                {"kind": "open", "pid": parent, "fd": 3, "path": "/vulkan/raw.adoc", "mode": "read"}, {"kind": "read", "pid": parent, "fd": 3}, {"kind": "close", "pid": parent, "fd": 3},
                {"kind": "open", "pid": parent, "fd": 3, "path": "/work/temporary/out.tmp", "mode": "write"}, {"kind": "write", "pid": parent, "fd": 3}, {"kind": "close", "pid": parent, "fd": 3},
                {"kind": "rename", "pid": parent, "old": "/work/temporary/out.tmp", "new": "/work/generated/out.adoc"}, {"kind": "open", "pid": parent, "fd": 3, "path": "/work/generated/out.adoc", "mode": "read"}, {"kind": "read", "pid": parent, "fd": 3}, {"kind": "close", "pid": parent, "fd": 3},
                {"kind": "process", "pid": child, "parent": parent, "argv_sha256": ARGV}, {"kind": "exec", "pid": child, "argv_sha256": ARGV}, {"kind": "exit", "pid": child}, {"kind": "exit", "pid": parent},
                {"kind": "snapshot", "path": "/vulkan/raw.adoc", "hex": RAW.hex()}, {"kind": "snapshot", "path": "/work/generated/out.adoc", "hex": RAW.hex()}, {"kind": "terminal", "status": "observed-unadmitted"})
    if any(row != want for row, want in zip(rows, expected)): reject("collector output is not the one bounded fixture receipt")
    return {"records": rows, "rows": len(rows), "sha256": hashlib.sha256(result.stdout).hexdigest()}


def execute(payloads: tuple[bytes, ...], temp: Path) -> tuple[dict[str, str], dict[str, object]]:
    binary, docker = runtime()
    with tempfile.TemporaryDirectory(prefix="webboxvm-synthetic-ptrace-", dir=temp) as directory:
        root = Path(directory); raw, work = root / "vulkan", root / "work"; os.chmod(root, 0o777); raw.mkdir(mode=0o777); work.mkdir(mode=0o777); os.chmod(raw, 0o777); os.chmod(work, 0o777)
        raw_file = raw / "raw.adoc"; raw_file.write_bytes(raw_value(payloads[-1])); os.chmod(raw_file, 0o444)
        try: result = subprocess.run(command(binary, raw, work), input=source_input(payloads), capture_output=True, check=False, timeout=30)
        except (OSError, subprocess.TimeoutExpired) as error: reject(f"synthetic collector Docker could not run: {detail(str(error))}")
    return docker, decoded(result)


def receipt(value: dict[str, str], helper_blob: str, blobs: list[dict[str, object]], payloads: tuple[bytes, ...], docker: dict[str, str], collector: dict[str, object]) -> dict[str, object]:
    return {"contract": CONTRACT, "status": "observed-unadmitted", "scope": "synthetic-fixture-only-not-docs-lineage", "anchor": {"repository": REPOSITORY, "ref": REF, "commit": value["commit"], "workflow_commit": value["workflow_commit"], "helper_path": HELPER, "helper_blob": helper_blob, "inputs": blobs, "raw_sha256": hashlib.sha256(RAW).hexdigest(), "raw_bytes": len(RAW)}, "input": {"contract": INPUT_CONTRACT, "members": input_record(payloads)}, "runner": {"environment": "github-hosted", "os": "Linux", "arch": "X64", "kernel": " ".join(os.uname())}, "container": {"image": IMAGE, "platform": "linux/amd64", "network": "none", "user": "501:20", "stdin": "reviewed-source-envelope", "purpose": "synthetic-collector-fixture-only", "image_pull_network": "host-preflight-only"}, "docker": docker, "run": {"id": value["run_id"], "attempt": value["attempt"]}, "collector": collector}


def main() -> None:
    if len(sys.argv) != 2: raise SystemExit("usage: lineage_github_synthetic_witness.py WORKSPACE")
    try:
        value = anchor(dict(os.environ)); helper_blob, blobs, payloads = inputs(Path(sys.argv[1]), value, Path(__file__))
        print(json.dumps(receipt(value, helper_blob, blobs, payloads, *execute(payloads, Path(required(dict(os.environ), "RUNNER_TEMP")))), sort_keys=True, separators=(",", ":")))
    except WitnessError as error:
        print(f"FAIL: {error}", file=sys.stderr); raise SystemExit(2)


if __name__ == "__main__": main()
