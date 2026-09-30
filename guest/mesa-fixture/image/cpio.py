"""Deterministic, bounded Linux newc writer for a graphics guest fixture."""

import hashlib
from pathlib import PurePosixPath
import stat

MAX_INITRD = 0x0F000000


def align(output):
    output.write(b"\0" * (-output.tell() % 4))


def entry(output, number, name, mode, data, major=0, minor=0):
    encoded = name.encode("utf-8")
    if (not name or name != PurePosixPath(name).as_posix() or name.startswith("/")
            or "\0" in name or any(part in (".", "..") for part in name.split("/"))):
        raise ValueError("cpio names must stay inside the guest root")
    fields = (number, mode, 0, 0, 2 if stat.S_ISDIR(mode) else 1, 0,
              len(data), 0, 0, major, minor, len(encoded) + 1, 0)
    if any(value < 0 or value > 0xFFFFFFFF for value in fields):
        raise ValueError("cpio field does not fit newc")
    output.write(b"070701" + b"".join(f"{value:08x}".encode() for value in fields))
    output.write(encoded + b"\0")
    align(output)
    output.write(data)
    align(output)
    if output.tell() >= MAX_INITRD:
        raise ValueError("custom initrd must remain strictly below 240 MiB")


def write(path, nodes):
    with path.open("wb") as output:
        for number, name in enumerate(sorted(nodes), 1):
            node = nodes[name]
            entry(output, number, name, node["mode"], node["data"],
                  node.get("major", 0), node.get("minor", 0))
        entry(output, 0, "TRAILER!!!", 0, b"")
    return {"bytes": path.stat().st_size, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
