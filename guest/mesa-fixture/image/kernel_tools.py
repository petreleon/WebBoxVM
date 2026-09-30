"""Isolated pinned depmod execution with retained first-command records."""

import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time
import uuid

BUILDER = "sha256:16b7e49d020541143f5a5fd9a26e8a277edcbad3c55726f0bdbe45880bbf9c2d"
RECIPE = Path(__file__).resolve().parents[1]
BUILDER_MANIFEST = RECIPE.parents[1] / ".artifacts/graphics/i01-mesa-image/build/manifests/builder.json"


def authorized_builder(builder):
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", builder):
        raise ValueError("depmod requires an immutable builder identity")
    if BUILDER_MANIFEST.is_symlink():
        raise ValueError("builder provenance must not be redirected")
    record = json.loads(BUILDER_MANIFEST.read_text())
    lock = json.loads((RECIPE / "build/lock.json").read_text())
    if (record.get("image_id") != builder or record.get("native_architecture") != "arm64"
            or record.get("base_image") != lock["base_image"] or record.get("snapshot") != lock["snapshot"]):
        raise ValueError("depmod builder differs from authorized recipe provenance")
    return record


class Commands:
    def __init__(self, logs):
        self.logs, self.rows = Path(logs), []
        if self.logs.exists() or self.logs.is_symlink():
            raise ValueError("preserve prior command logs; choose a new log directory")
        self.logs.mkdir(parents=True)

    def run(self, argv, name, payload=None):
        if not re.fullmatch(r"[a-z0-9-]+", name):
            raise ValueError("unsafe command log name")
        start = time.monotonic()
        owned = None
        if argv[:2] == ["docker", "run"]:
            owned = "webboxvm-i01-kernel-" + uuid.uuid4().hex
            cidfile = str((self.logs / (name + ".cid")).resolve())
            argv = argv[:2] + ["--name", owned, "--cidfile", cidfile] + argv[2:]
        try:
            result = subprocess.run(argv, input=payload, capture_output=True, timeout=120, check=False)
            status, stdout, stderr = result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired as error:
            status, stdout, stderr = 124, error.stdout or b"", error.stderr or b""
        except OSError as error:
            status, stdout, stderr = 127, b"", str(error).encode()
        row = {"argv": argv, "exit_status": status, "seconds": time.monotonic() - start}
        if payload is not None:
            row["input"] = {"bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest()}
        if status == 124 and owned:
            cleanup_argv = ["docker", "rm", "--force", owned]
            try:
                cleanup = subprocess.run(cleanup_argv, capture_output=True, timeout=30, check=False)
                cleanup_status, cleanup_stdout, cleanup_stderr = cleanup.returncode, cleanup.stdout, cleanup.stderr
            except (OSError, subprocess.TimeoutExpired) as error:
                cleanup_status, cleanup_stdout, cleanup_stderr = 125, b"", str(error).encode()
            row["cleanup"] = {"argv": cleanup_argv, "exit_status": cleanup_status,
                "stdout": cleanup_stdout.decode(errors="replace"), "stderr": cleanup_stderr.decode(errors="replace")}
        for stream, raw in (("stdout", stdout), ("stderr", stderr)):
            path = self.logs / (name + "." + stream)
            with path.open("xb") as output:
                output.write(raw)
            row[stream] = {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
        self.rows.append(row)
        with (self.logs / (name + ".json")).open("x") as output:
            json.dump(row, output, indent=2, sort_keys=True)
            output.write("\n")
        if status:
            raise ValueError(f"{name}: first failure exit {status}; inspect {self.logs}")
        return stdout


def container(builder, executable):
    argv = ["docker", "run", "--interactive", "--rm", "--pull=never", "--platform", "linux/arm64",
            "--network=none", "--read-only", "--cap-drop=ALL", "--security-opt=no-new-privileges",
            "--user", f"{os.getuid()}:{os.getgid()}", "--tmpfs",
            f"/work:rw,noexec,nosuid,nodev,size=512m,uid={os.getuid()},gid={os.getgid()},mode=0700"]
    return argv + ["--entrypoint", executable, builder]


def verify_builder(commands, builder):
    provenance = authorized_builder(builder)
    template = '{"id":{{json .Id}},"architecture":{{json .Architecture}}}'
    identity = json.loads(commands.run(["docker", "image", "inspect", builder, "--format", template], "01-image"))
    if identity != {"id": builder, "architecture": "arm64"}:
        raise ValueError("depmod builder identity or architecture mismatch")
    versions = commands.run(container(builder, "/usr/bin/dpkg-query") +
        ["--show", "--showformat=${Package}\t${Version}\t${Architecture}\n", "kmod", "libkmod2", "xz-utils"], "02-tools")
    rows = [line.split("\t") for line in versions.decode().splitlines()]
    expected = [["kmod", "34.2-2", "arm64"], ["libkmod2", "34.2-2", "arm64"],
                ["xz-utils", "5.8.1-1+deb13u1", "arm64"]]
    if rows != expected:
        raise ValueError("depmod tool package versions differ from their pins")
    return {**identity, "packages": rows, "provenance": provenance}


def depmod(commands, builder, root, release, name):
    from kernel_transport import metadata, module_tar
    if not re.fullmatch(r"[A-Za-z0-9.+_-]+", release):
        raise ValueError("unsafe kernel release")
    script = (f"set -eu; cd /work; tar -xf -; /usr/sbin/depmod -b /work -a {release}; "
              f"tar -cf - usr/lib/modules/{release}/modules.*")
    raw = commands.run(container(builder, "/bin/sh") + ["-c", script], name, module_tar(root, release))
    metadata(raw, root, release)
