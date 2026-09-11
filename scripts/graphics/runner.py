"""Execute local checks while preserving observations, not graphics claims."""

from __future__ import annotations

import datetime as dt
import platform
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Sequence

from catalog import Check
from prerequisites import missing
from result import artifact, sha256

COUNT = re.compile(r"(?m)^WEBBOXVM_GRAPHICS_OBSERVED_COUNT=(\d+)\s*$")


def git(root: Path, *arguments: str) -> subprocess.CompletedProcess[bytes] | None:
    try:
        return subprocess.run(["git", *arguments], cwd=root, capture_output=True, check=False)
    except OSError:
        return None


def snapshot(root: Path) -> tuple[dict[str, str | None], dict[str, str] | None]:
    revision = git(root, "rev-parse", "HEAD")
    diff = git(root, "diff", "--no-ext-diff", "--binary", "HEAD")
    if revision is None or diff is None or revision.returncode or diff.returncode:
        return ({"revision": None, "dirty_diff_sha256": None}, {
            "kind": "repository", "value": str(root), "reason": "git revision or diff is unavailable",
        })
    return ({
        "revision": revision.stdout.decode("utf-8", "replace").strip(),
        "dirty_diff_sha256": sha256(diff.stdout),
    }, None)


def version(command: Sequence[str], root: Path) -> dict[str, Any]:
    try:
        result = subprocess.run(command, cwd=root, text=True, errors="replace", capture_output=True, check=False)
    except OSError as error:
        return {"command": list(command), "exit_status": None, "error": str(error)}
    return {
        "command": list(command), "exit_status": result.returncode,
        "stdout": result.stdout, "stderr": result.stderr,
    }


def output(stdout: str = "", stderr: str = "") -> dict[str, str]:
    return {
        "stdout": stdout, "stderr": stderr,
        "stdout_sha256": sha256(stdout.encode()), "stderr_sha256": sha256(stderr.encode()),
    }


def observed(stdout: str, stderr: str) -> int | None:
    markers = COUNT.findall(f"{stdout}\n{stderr}")
    return int(markers[0]) if len(markers) == 1 else None


def run_check(check: Check, root: Path, repository_blocker: dict[str, str] | None) -> dict[str, Any]:
    began = time.monotonic()
    blockers = ([] if repository_blocker is None else [repository_blocker]) + missing(root, check.prerequisites)
    record: dict[str, Any] = {
        "name": check.name, "command": list(check.command), "tool_versions": [version(tool, root) for tool in check.tools],
        "prerequisites": blockers, "expected_count": check.expected_count, "observed_count": 0,
        "exit_status": None, "artifacts": [], "output": output(), "errors": [],
    }
    if blockers:
        record["result"] = "BLOCKED"
    else:
        try:
            child = subprocess.run(check.command, cwd=root, text=True, errors="replace", capture_output=True, check=False)
        except OSError as error:
            record["result"] = "FAIL"
            record["errors"].append(f"cannot execute child: {error}")
        else:
            record["exit_status"] = child.returncode
            record["output"] = output(child.stdout, child.stderr)
            record["observed_count"] = observed(child.stdout, child.stderr)
            if child.returncode:
                record["errors"].append(f"child exited {child.returncode}")
            if record["observed_count"] is None:
                record["errors"].append("child did not emit exactly one observed-count marker")
            elif record["observed_count"] != check.expected_count:
                record["errors"].append(
                    f"expected {check.expected_count} observed cases, got {record['observed_count']}"
                )
            record["result"] = "PASS" if not record["errors"] else "FAIL"
    record["artifacts"] = [artifact(root, item) for item in check.artifacts]
    missing_artifacts = [item["path"] for item in record["artifacts"] if item.get("error")]
    if missing_artifacts and record["result"] != "BLOCKED":
        record["result"] = "FAIL"
        record["errors"].append("missing artifacts: " + ", ".join(missing_artifacts))
    record["duration_ms"] = round((time.monotonic() - began) * 1000, 3)
    return record


def base_result(root: Path, catalog: Path, selection: Sequence[str]) -> tuple[dict[str, Any], dict[str, str] | None]:
    identity, blocker = snapshot(root)
    git_version = version(("git", "--version"), root)
    return ({
        "schema": 1, "result": "INVALID", "started_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "root": str(root), "catalog": str(catalog), "selection": list(selection), **identity,
        "runner_tool_versions": {"python": sys.version, "platform": platform.platform(), "git": git_version},
        "checks": [], "errors": [],
    }, blocker)


def execute(root: Path, catalog: Path, checks: Sequence[Check], selection: Sequence[str]) -> tuple[dict[str, Any], int]:
    record, blocker = base_result(root, catalog, selection)
    record["checks"] = [run_check(check, root, blocker) for check in checks]
    results = [item["result"] for item in record["checks"]]
    record["result"] = "FAIL" if "FAIL" in results else "BLOCKED" if "BLOCKED" in results else "PASS"
    failures = [item for item in record["checks"] if item["result"] == "FAIL"]
    if failures:
        child_codes = [item["exit_status"] for item in failures if item["exit_status"] not in (None, 0)]
        return record, child_codes[0] if child_codes else 1
    return record, 3 if "BLOCKED" in results else 0


def invalid(root: Path, catalog: Path, selection: Sequence[str], error: Exception) -> dict[str, Any]:
    record, _ = base_result(root, catalog, selection)
    record["errors"].append(str(error))
    return record
