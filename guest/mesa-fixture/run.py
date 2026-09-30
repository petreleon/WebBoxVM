#!/usr/bin/env python3
"""Run bounded stock-Mesa guest checks and preserve every run's raw UART."""

import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time

from verify_image import digest, verify

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
DEFAULT = REPO / ".artifacts/graphics/i01-mesa-image"
REQUIRED = {"tools": ("I01_GUEST_TOOLS_PASS",),
            "startup": ("I01_MESA_GLES_STARTUP_PASS", "I01_MESA_VULKAN_STARTUP_PASS")}


def execute(image, runner, mode, result_path, timeout=960):
    manifest = verify(image)
    if runner.is_symlink() or not runner.is_file():
        raise ValueError("build the regular guest_command runner first")
    if result_path.is_symlink():
        raise ValueError("result path must not be a symlink")
    result_path.parent.mkdir(parents=True, exist_ok=True)
    directory = Path(tempfile.mkdtemp(prefix="guest-", dir=result_path.parent))
    uart = directory / "uart.log"
    script = HERE / "checks" / (mode + ".sh")
    runner_sha256, script_sha256 = digest(runner), digest(script)
    command = [str(runner), "--kernel", str(image / "Image"), "--initrd", str(image / "initrd.cpio"),
               "--command-file", str(script), "--uart-log", str(uart)]
    for line in REQUIRED[mode]:
        command += ["--require", line]
    started = time.monotonic()
    timed_out = False
    with (directory / "stdout.log").open("wb") as stdout, (directory / "stderr.log").open("wb") as stderr:
        process = subprocess.Popen(command, cwd=REPO, stdout=stdout, stderr=stderr, start_new_session=True)
        try:
            code = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGKILL)
            code = process.wait()
    artifacts = []
    for name in ("stdout.log", "stderr.log", "uart.log"):
        path = directory / name
        artifacts.append({"path": str(path), "bytes": path.stat().st_size, "sha256": digest(path)}
                         if path.is_file() else {"path": str(path), "missing": True})
    errors = []
    try:
        if verify(image) != manifest or digest(runner) != runner_sha256 or digest(script) != script_sha256:
            errors.append("fixture/runner/script changed during execution")
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
        errors.append("fixture inputs changed: " + str(error))
    passed = code == 0 and not timed_out and uart.is_file() and not errors
    value = {"schema": 1, "kind": "webboxvm-i01-guest-check", "mode": mode,
             "result": "PASS" if passed else "FAIL", "exit_status": code, "timeout": timed_out,
             "command": command, "duration_seconds": time.monotonic() - started,
             "manifest_sha256": manifest["manifest_sha256"], "runner_sha256": runner_sha256,
             "script_sha256": script_sha256, "errors": errors,
             "required_lines": list(REQUIRED[mode]), "artifacts": artifacts,
             "conformance_claimed": False, "performance_claimed": False}
    temporary = directory / "result.json"
    temporary.write_text(json.dumps(value, indent=2) + "\n")
    latest = directory / "latest.json"
    latest.write_bytes(temporary.read_bytes())
    latest.replace(result_path)
    print(json.dumps(value, indent=2))
    return 0 if passed else code if code > 0 else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--image", type=Path, default=DEFAULT / "image")
    parser.add_argument("--runner", type=Path, default=REPO / "target/release/examples/guest_command")
    parser.add_argument("--mode", choices=REQUIRED, default="startup")
    parser.add_argument("--result", type=Path, default=DEFAULT / "guest-result.json")
    args = parser.parse_args()
    try:
        code = execute(args.image.absolute(), args.runner.absolute(), args.mode, args.result.absolute())
    except (ValueError, OSError, KeyError, TypeError, json.JSONDecodeError) as error:
        parser.exit(2, f"FAIL: {error}\n")
    raise SystemExit(code)


if __name__ == "__main__":
    main()
