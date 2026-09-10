#!/usr/bin/env python3
"""Run only the bounded fixture collector in the previously observed Docker policy."""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

from lineage_model import reject
from lineage_synthetic_collect import collect

IMAGE = "khronosgroup/docker-images@sha256:f1ca671f3bdb10ad49e238b9bf28853088a21af49504498fc9084c9b4fea4762"
HERE = Path(__file__).resolve().parent
SOURCE = HERE / "observer"
SOURCES = ("lineage_ptrace_state.h", "lineage_ptrace_state.c", "lineage_ptrace_fixture.c", "lineage_ptrace_fixture_collector.c")


def safe_dir(value: Path, label: str) -> Path:
    try:
        if value.is_symlink() or not value.is_dir(): reject(f"synthetic collector {label} is not a safe directory")
        return value.resolve(strict=True)
    except OSError as error: reject(f"synthetic collector {label} cannot resolve: {error}")


def sources(value: Path) -> Path:
    root = safe_dir(value, "source")
    if any((root / name).is_symlink() or not (root / name).is_file() for name in SOURCES): reject("synthetic collector source is incomplete")
    return root


def command(binary: str, source: Path, vulkan: Path, work: Path) -> list[str]:
    source, vulkan, work = sources(source), safe_dir(vulkan, "raw root"), safe_dir(work, "work root")
    if any((work / name).is_symlink() or not (work / name).is_dir() for name in ("temporary", "generated")):
        reject("synthetic collector work root lacks controlled directories")
    script = "set -eu\ngcc -std=c11 -O2 -Wall -Werror /source/lineage_ptrace_state.c /source/lineage_ptrace_fixture_collector.c -o /work/collector\ngcc -std=c11 -O2 -Wall -Werror /source/lineage_ptrace_fixture.c -o /work/fixture\nexec /work/collector /work/fixture"
    return [binary, "run", "--rm", "--pull=never", "--network", "none", "--platform", "linux/amd64", "--user", "501:20",
            "--mount", f"type=bind,src={source},dst=/source,readonly", "--mount", f"type=bind,src={vulkan},dst=/vulkan,readonly",
            "--mount", f"type=bind,src={work},dst=/work", "-w", "/work", "--entrypoint", "sh", IMAGE, "-c", script]


def run(vulkan: Path, work: Path, source: Path = SOURCE):
    binary = shutil.which("docker")
    if not binary: reject("synthetic collector Docker CLI is unavailable")
    try: result = subprocess.run(command(binary, source, vulkan, work), capture_output=True, check=False, timeout=20)
    except (OSError, subprocess.TimeoutExpired) as error: reject(f"synthetic collector Docker cannot run: {error}")
    if result.returncode: reject(f"synthetic collector Docker did not complete: {result.returncode}")
    return collect(result.stdout)
