#!/usr/bin/env python3
"""Extract the pinned kernel, compile clients, and atomically publish one fixture."""

import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
ARTIFACTS = REPO / ".artifacts/graphics/i01-mesa-image"


def stage(command, logs, name):
    started = time.monotonic()
    path = logs / (name + ".log")
    with path.open("xb") as stream:
        try:
            code = subprocess.run(command, stdout=stream, stderr=subprocess.STDOUT, timeout=600).returncode
        except subprocess.TimeoutExpired:
            code = 124
    row = {"command": command, "exit_status": code, "seconds": time.monotonic() - started,
           "log": str(path)}
    (logs / (name + ".json")).write_text(json.dumps(row, indent=2) + "\n")
    if code:
        raise ValueError(f"{name}: first failure exit {code}; inspect {path}")


def finish(attempt, output):
    if not re.fullmatch(r"[0-9]{2}", attempt):
        raise ValueError("fixture attempt must have two digits")
    if output.exists() or output.is_symlink():
        raise ValueError("preserve prior image; choose a fresh output directory")
    build = ARTIFACTS / "build"
    mesa = json.loads((build / "manifests/build.json").read_text())
    lock = json.loads((HERE / "build/lock.json").read_text())
    if (mesa.get("source") != lock["mesa"] or mesa.get("meson_args") != lock["meson_args"]
            or mesa.get("source_before_compile") != "PASS" or mesa.get("source_after_compile") != "PASS"
            or mesa.get("source_patched") is not False or mesa.get("network") != "none"):
        raise ValueError("Mesa build lacks the pinned unmodified source/flag proof")
    manifest = json.loads((build / "manifests/builder.json").read_text())
    builder = manifest["image_id"]
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", builder) or manifest["native_architecture"] != "arm64":
        raise ValueError("builder manifest lacks an immutable ARM64 identity")
    work = ARTIFACTS / ("fixture-" + attempt)
    if work.exists() or work.is_symlink():
        raise ValueError("preserve prior fixture work; choose a fresh attempt")
    logs = work / "logs"
    logs.mkdir(parents=True)
    stage([sys.executable, str(HERE / "build/verify.py")], logs, "01-inputs")
    stage([sys.executable, str(HERE / "verify_outputs.py"), "--build", str(build)], logs, "01-runtime")
    package = build / "kernel-package/linux-image-6.12.94+deb13-arm64_6.12.94-1_arm64.deb"
    stage([sys.executable, str(HERE / "image/kernel.py"), "--package", str(package),
           "--output", str(work / "kernel"), "--logs", str(work / "kernel-logs"),
           "--builder", builder], logs, "02-kernel")
    stage(["docker", "run", "--platform", "linux/arm64", "--pull=never", "--network=none",
           "--cpus=2", "--memory=1g", "--name", "webboxvm-i01-probes-" + attempt,
           "-v", str(ARTIFACTS) + ":/fixture", "-v", str(HERE / "probes") + ":/probes:ro",
           "-v", str(build / "mesa-destdir/opt/mesa-f02") + ":/opt/mesa-f02:ro", builder,
           "/usr/bin/timeout", "--kill-after=5", "120", "python3", "/probes/compile.py",
           "--mesa-manifest", "/fixture/build/manifests/build.json", "--output",
           "/fixture/fixture-" + attempt + "/probes"], logs, "03-probes")
    stage([sys.executable, str(HERE / "verify_outputs.py"), "--build", str(build)], logs, "03-runtime-recheck")
    stage([sys.executable, str(HERE / "image/assemble.py"), "--rootfs", str(build / "runtime-rootfs"),
           "--modules", str(work / "kernel/modules"), "--kernel", str(work / "kernel/Image"),
           "--probes", str(work / "probes/rootfs"), "--output", str(output)], logs, "04-image")
    print(json.dumps({"result": "PASS", "scope": "fixture production only", "image": str(output),
                      "work": str(work), "guest_runtime_validated": False}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--attempt", default="01")
    parser.add_argument("--output", type=Path, default=ARTIFACTS / "image")
    args = parser.parse_args()
    try:
        finish(args.attempt, args.output.absolute())
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
        parser.exit(2, f"FAIL: {error}\n")


if __name__ == "__main__":
    main()
