#!/usr/bin/env python3
"""Run a confined ptrace capability gate without widening the pinned policy."""

from __future__ import annotations

import argparse
import json
import subprocess
from dataclasses import dataclass
from pathlib import Path

from lineage_model import reject
from lineage_probe_paths import WORK_PARENT, WORK_PREFIX, recheck, same_source, source_file, work_path, work_root

IMAGE = "khronosgroup/docker-images@sha256:f1ca671f3bdb10ad49e238b9bf28853088a21af49504498fc9084c9b4fea4762"
CONTRACT = "webboxvm-graphics-ptrace-capability-v3"
STAGES = frozenset(("setup", "fork", "child-traceme", "initial-wait", "initial-stop", "parent-setoptions",
                    "parent-syscall-entry", "parent-syscall-entry-wait", "parent-syscall-entry-stop",
                    "parent-syscall-entry-info", "parent-syscall-exit", "parent-syscall-exit-wait",
                    "parent-syscall-exit-stop", "parent-syscall-exit-info", "parent-fork", "parent-fork-wait",
                    "parent-fork-event", "parent-fork-pid", "parent-fork-parent-cont", "parent-fork-child-wait",
                    "parent-fork-child-stop", "parent-exec", "parent-exec-wait", "parent-exec-event",
                    "parent-exec-cont", "parent-exit-wait", "parent-exit-cont", "parent-exit", "parent-fork-exec-complete"))


@dataclass(frozen=True)
class Capability:
    available: bool
    stage: str
    error_number: int


def command(work: Path, source: Path | None = None) -> list[str]:
    work = work_path(work); source = source_file()[0] if source is None else source
    script = "set -eu\ngcc -std=c11 -O2 -Wall -Werror /lineage_ptrace_probe.c -o /work/lineage-ptrace-probe\nexec /work/lineage-ptrace-probe"
    return ["docker", "run", "--rm", "--network", "none", "--platform", "linux/amd64", "--user", "501:20",
            "--mount", f"type=bind,src={source},dst=/lineage_ptrace_probe.c,readonly",
            "--mount", f"type=bind,src={work},dst=/work", "-w", "/work", "--entrypoint", "sh", IMAGE, "-c", script]


def decode(returncode: int, output: str) -> Capability:
    try: value = json.loads(output.strip())
    except json.JSONDecodeError as error: reject(f"ptrace gate has invalid JSON output: {error}")
    if not isinstance(value, dict) or set(value) != {"contract", "status", "stage", "errno"} or value["contract"] != CONTRACT:
        reject("ptrace gate has an invalid output schema")
    if value["stage"] not in STAGES or type(value["errno"]) is not int or value["errno"] < 0:
        reject("ptrace gate has an invalid stage or errno")
    available = value["status"] == "available"
    if value["status"] not in {"available", "blocked"}: reject("ptrace gate has an invalid status")
    if available: reject("ptrace gate cannot establish an anchored available execution")
    if not available and returncode != 77: reject("ptrace gate did not fail closed")
    return Capability(available, value["stage"], value["errno"])


def probe(work: Path) -> Capability:
    root = work_root(work)
    try:
        source = source_file(); recheck(root)
        try: completed = subprocess.run(command(root.path, source[0]), text=True, capture_output=True, check=False)
        except OSError as error: reject(f"ptrace gate did not start: {error}")
        recheck(root); same_source(source)
        if completed.returncode not in {0, 77} and not completed.stdout.strip():
            reject(f"ptrace gate did not start: {completed.stderr.strip() or completed.returncode}")
        result = decode(completed.returncode, completed.stdout)
    finally:
        root.close()
    return result


def main() -> None:
    parser = argparse.ArgumentParser(); parser.add_argument("--work", type=Path, required=True)
    try: result = probe(parser.parse_args().work)
    except Exception as error:
        print(f"FAIL: {error}")
        raise SystemExit(2)
    print(f"CAPABILITY: BLOCKED {result.stage} errno={result.error_number}")
    raise SystemExit(77)


if __name__ == "__main__": main()
