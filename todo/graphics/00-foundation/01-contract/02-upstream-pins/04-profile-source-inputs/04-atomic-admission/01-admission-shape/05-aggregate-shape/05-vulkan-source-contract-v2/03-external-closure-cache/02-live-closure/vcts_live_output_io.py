"""Small descriptor-anchored primitives for private VCTS output transactions."""

from __future__ import annotations

import fcntl
import hashlib
import os
import stat
from pathlib import Path

JOURNAL = ".vcts-live-output-journal"
PREFIX = ".vcts-live-output-stage-"
VERSION = b"vcts-live-output-journal-v1"
HEX = set("0123456789abcdef")


class JournalError(ValueError):
    """Output transaction state is unsafe, occupied, or cannot be recovered."""


def reject(message: str) -> None:
    raise JournalError(message)


def failure(action: str, error: OSError) -> str:
    return f"{action} failed (errno {error.errno}, path {error.filename or '<unknown>'}): {error.strerror or error}"


def binding(paths: tuple[Path, Path, Path]) -> str:
    return hashlib.sha256(b"\0".join(os.fsencode(path.name) for path in paths)).hexdigest()


def parent(paths: tuple[Path, Path, Path]) -> int:
    if len(set(paths)) != 3 or any(not isinstance(path, Path) or not path.is_absolute() for path in paths):
        reject("capture output paths must be three distinct absolute paths")
    base, names = paths[0].parent, [path.name for path in paths]
    if (any(path.parent != base or not name or name == JOURNAL or name.startswith(PREFIX) for path, name in zip(paths, names))
            or any(part in (".", "..") for path in paths for part in path.parts)):
        reject("capture output paths must share one safe parent and ordinary names")
    try:
        if base.resolve(strict=True) != base:
            reject("capture output parent must not be a symlink or alias")
        fd = os.open(base, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0))
        state = os.fstat(fd)
    except OSError as error:
        reject(failure("capture output parent open", error))
    if not stat.S_ISDIR(state.st_mode) or stat.S_ISLNK(state.st_mode) or state.st_uid != os.geteuid() or stat.S_IMODE(state.st_mode) != 0o700:
        os.close(fd)
        reject("capture output parent must be a current-user 0700 directory")
    return fd


def state(fd: int, name: str):
    try:
        return os.lstat(name, dir_fd=fd)
    except FileNotFoundError:
        return None
    except OSError as error:
        reject(failure("capture output inspection", error))


def private(value) -> bool:
    return stat.S_ISREG(value.st_mode) and value.st_uid == os.geteuid() and not value.st_mode & 0o077


def read(fd: int) -> bytes:
    try:
        value = os.fstat(fd)
        if not private(value) or stat.S_IMODE(value.st_mode) != 0o600 or value.st_size > 256:
            reject("capture output journal is not a private bounded regular file")
        os.lseek(fd, 0, os.SEEK_SET)
        data = os.read(fd, 257)
        if len(data) != value.st_size:
            reject("capture output journal changed while being read")
        return data
    except OSError as error:
        reject(failure("capture output journal read", error))


def record_data(paths: tuple[Path, Path, Path], parent_fd: int, nonce: str) -> bytes:
    value = os.fstat(parent_fd)
    return b"\n".join((VERSION, nonce.encode("ascii"), binding(paths).encode("ascii"),
                          f"{value.st_dev}:{value.st_ino}".encode("ascii"), b""))


def record(data: bytes, paths: tuple[Path, Path, Path], parent_fd: int) -> str:
    fields = data.split(b"\n")
    expected = record_data(paths, parent_fd, "x" * 64).split(b"\n")
    if len(fields) != len(expected) or fields[0] != expected[0] or fields[2:] != expected[2:]:
        reject("capture output journal does not bind this directory and output names")
    try:
        nonce = fields[1].decode("ascii")
    except UnicodeDecodeError:
        reject("capture output journal nonce is invalid")
    if len(nonce) != 64 or set(nonce) - HEX:
        reject("capture output journal nonce is invalid")
    return nonce


def lock(fd: int) -> None:
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError as error:
        reject(failure("capture output transaction lock", error))


def write(fd: int, data: bytes) -> None:
    view = memoryview(data)
    while view:
        count = os.write(fd, view)
        if count < 1:
            reject("capture output journal write made no progress")
        view = view[count:]


def stage(name: str, nonce: str) -> bool:
    head, tail = f"{PREFIX}{nonce}-", name.removeprefix(f"{PREFIX}{nonce}-")
    return name.startswith(head) and len(tail) == 32 and not (set(tail) - HEX)
