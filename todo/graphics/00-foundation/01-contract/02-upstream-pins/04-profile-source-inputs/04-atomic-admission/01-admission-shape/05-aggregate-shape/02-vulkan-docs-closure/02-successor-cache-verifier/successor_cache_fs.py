"""Descriptor-anchored file, marker, and lock operations for the cache."""

from __future__ import annotations

import os
import stat
import uuid
from pathlib import Path

from successor_cache_model import reject
from successor_cache_paths import NOFOLLOW, close, parent_fd, relative

FILE_FLAGS = os.O_RDONLY | NOFOLLOW | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NONBLOCK", 0)


def _regular(descriptor: int, name: str, label: str) -> None:
    try:
        target = os.open(name, FILE_FLAGS, dir_fd=descriptor)
    except OSError as error:
        reject(f"{label} is a symlink or unsafe: {error}")
    try:
        try:
            regular = stat.S_ISREG(os.fstat(target).st_mode)
        except (OSError, ValueError) as error:
            reject(f"{label} cannot be inspected safely: {error}")
        if not regular:
            reject(f"{label} is not a regular file")
    finally:
        close(target)


def read_file(
    root: Path, value: str, label: str, *, expected_bytes: int | None = None,
    maximum_bytes: int | None = None, optional: bool = False,
) -> bytes | None:
    try:
        parent, leaf = parent_fd(root, value, False)
    except FileNotFoundError:
        if optional:
            return None
        reject(f"{label} is missing")
    try:
        try:
            target = os.open(leaf, FILE_FLAGS, dir_fd=parent)
        except FileNotFoundError:
            if optional:
                return None
            reject(f"{label} is missing")
        except (OSError, ValueError) as error:
            reject(f"{label} is a symlink or unsafe: {error}")
        try:
            info = os.fstat(target)
            if not stat.S_ISREG(info.st_mode):
                reject(f"{label} is not a regular file")
            if expected_bytes is not None and info.st_size != expected_bytes:
                reject(f"{label} has a byte count mismatch")
            if maximum_bytes is not None and info.st_size > maximum_bytes:
                reject(f"{label} exceeds its byte limit")
            limit = expected_bytes if expected_bytes is not None else maximum_bytes
            if maximum_bytes is not None and (limit is None or maximum_bytes < limit):
                limit = maximum_bytes
            if limit is None:
                reject(f"{label} has no read limit")
            remaining, chunks = limit + 1, []
            while remaining and (chunk := os.read(target, min(1024 * 1024, remaining))):
                chunks.append(chunk)
                remaining -= len(chunk)
            if not remaining:
                reject(f"{label} has a byte count mismatch" if expected_bytes is not None
                       else f"{label} exceeds its byte limit")
            payload = b"".join(chunks)
            if expected_bytes is not None and len(payload) != expected_bytes:
                reject(f"{label} has a byte count mismatch")
            return payload
        except OSError as error:
            reject(f"{label} cannot be read safely: {error}")
        finally:
            close(target)
    finally:
        close(parent)


def same_parent(root: Path, value: str, held: int) -> None:
    try:
        current, _ = parent_fd(root, value, False)
    except FileNotFoundError:
        reject("successor cache parent changed during write")
    try:
        try:
            before, after = os.fstat(held), os.fstat(current)
        except OSError as error:
            reject(f"successor cache parent cannot be checked safely: {error}")
        if (before.st_dev, before.st_ino) != (after.st_dev, after.st_ino):
            reject("successor cache parent changed during write")
    finally:
        close(current)


def _write(descriptor: int, payload: bytes) -> None:
    remaining = memoryview(payload)
    while remaining:
        count = os.write(descriptor, remaining)
        if count <= 0:
            reject("successor cache write made no progress")
        remaining = remaining[count:]
    os.fsync(descriptor)


def atomic_file(root: Path, value: str, payload: bytes, label: str) -> Path:
    parent, leaf = parent_fd(root, value, True)
    temporary = f".successor-{uuid.uuid4().hex}.tmp"
    descriptor, linked = -1, False
    try:
        try:
            descriptor = os.open(
                temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | NOFOLLOW, 0o600, dir_fd=parent,
            )
            _write(descriptor, payload)
        except (OSError, ValueError) as error:
            reject(f"{label} cannot be staged safely: {error}")
        finally:
            if descriptor >= 0:
                close(descriptor)
                descriptor = -1
        try:
            os.link(temporary, leaf, src_dir_fd=parent, dst_dir_fd=parent, follow_symlinks=False)
            linked = True
        except FileExistsError:
            reject(f"successor cache refuses to overwrite existing {label}")
        except (OSError, ValueError) as error:
            reject(f"{label} cannot be published safely: {error}")
        try:
            os.unlink(temporary, dir_fd=parent)
        except (OSError, ValueError) as error:
            reject(f"{label} cannot finalize staging safely: {error}")
        temporary = ""
        try:
            os.fsync(parent)
        except (OSError, ValueError) as error:
            reject(f"{label} cannot be synchronized safely: {error}")
        _regular(parent, leaf, label)
        same_parent(root, value, parent)
        return root.joinpath(*relative(value))
    finally:
        if descriptor >= 0:
            close(descriptor)
        if temporary:
            try:
                os.unlink(temporary, dir_fd=parent)
            except OSError:
                pass
        if linked:
            try:
                os.fsync(parent)
            except OSError:
                pass
        close(parent)


def remove_file(root: Path, value: str) -> None:
    try:
        parent, leaf = parent_fd(root, value, False)
    except FileNotFoundError:
        return
    try:
        try:
            _regular(parent, leaf, "successor cache completion marker")
            os.unlink(leaf, dir_fd=parent)
            os.fsync(parent)
        except FileNotFoundError:
            return
        except (OSError, ValueError) as error:
            reject(f"successor cache completion marker cannot be removed safely: {error}")
    finally:
        close(parent)
