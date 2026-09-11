"""Bounded, component-wise no-follow reads for immutable roadmap records."""

import os
import stat
from pathlib import Path


class ReaderError(ValueError):
    """A record could not be read without following an unsafe path."""


NOFOLLOW = getattr(os, "O_NOFOLLOW", 0)
DIR_FLAGS = os.O_RDONLY | os.O_DIRECTORY | NOFOLLOW | getattr(os, "O_CLOEXEC", 0)
FILE_FLAGS = os.O_RDONLY | os.O_NONBLOCK | NOFOLLOW | getattr(os, "O_CLOEXEC", 0)


def stamp(info: os.stat_result) -> tuple[int, int, int, int, int]:
    return info.st_dev, info.st_ino, info.st_size, info.st_ctime_ns, info.st_nlink


def opened(path: Path, label: str) -> int:
    if not NOFOLLOW:
        raise ReaderError(f"{label} requires no-follow file descriptors")
    absolute = Path(os.path.abspath(path))
    try:
        current = os.open(absolute.anchor, DIR_FLAGS)
        try:
            for part in absolute.parts[1:-1]:
                successor = os.open(part, DIR_FLAGS, dir_fd=current)
                os.close(current)
                current = successor
            named = os.stat(absolute.name, dir_fd=current, follow_symlinks=False)
            descriptor = os.open(absolute.name, FILE_FLAGS, dir_fd=current)
            if stamp(named) != stamp(os.fstat(descriptor)):
                os.close(descriptor)
                raise ReaderError(f"{label} changed before safe open")
            return descriptor
        finally:
            os.close(current)
    except OSError as error:
        raise ReaderError(f"{label} cannot be safely opened: {error}") from error


def bounded_bytes(path: Path, label: str, limit: int) -> bytes:
    descriptor = opened(path, label)
    try:
        info = os.fstat(descriptor)
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
            raise ReaderError(f"{label} is not a regular file")
        value = b""
        while len(value) <= limit and len(value) < info.st_size:
            chunk = os.read(descriptor, limit + 1 - len(value))
            if not chunk:
                break
            value += chunk
        if stamp(info) != stamp(os.fstat(descriptor)):
            raise ReaderError(f"{label} changed during read")
    except OSError as error:
        raise ReaderError(f"{label} cannot be read: {error}") from error
    finally:
        os.close(descriptor)
    if len(value) > limit or len(value) != info.st_size:
        raise ReaderError(f"{label} exceeds its bounded size or changed during read")
    return value
