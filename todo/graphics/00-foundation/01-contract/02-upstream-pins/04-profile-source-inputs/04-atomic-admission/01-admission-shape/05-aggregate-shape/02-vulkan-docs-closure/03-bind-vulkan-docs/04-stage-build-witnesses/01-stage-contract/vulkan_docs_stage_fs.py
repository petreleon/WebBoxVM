"""Descriptor-anchored bounded reads and atomic no-overwrite writes for staging."""

from __future__ import annotations

import os
import stat
import uuid
import hashlib
from pathlib import Path

from vulkan_docs_stage_model import MAX_OUTPUT_BYTES, StagingPlan, bounded_payload, reject, require_live_plan
from vulkan_docs_stage_parse import digest
from vulkan_docs_stage_paths import NOFOLLOW, _parent_fd, close, relative

FILE_FLAGS = os.O_RDONLY | NOFOLLOW | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NONBLOCK", 0)

def _same_file(left, right, label: str) -> None:
    if (left.st_dev, left.st_ino, left.st_size) != (right.st_dev, right.st_ino, right.st_size):
        reject(f"{label} changed during the operation")


def _same_read(left, right, label: str) -> None:
    _same_file(left, right, label)
    if left.st_ctime_ns != right.st_ctime_ns:
        reject(f"{label} changed during the operation")


def _published(descriptor: int, name: str, source, label: str) -> None:
    try:
        target = os.open(name, FILE_FLAGS, dir_fd=descriptor)
    except OSError as error:
        reject(f"{label} is a symlink or unsafe: {error}")
    try:
        info = os.fstat(target)
        if not stat.S_ISREG(info.st_mode):
            reject(f"{label} is not a regular file")
        if info.st_nlink != 1:
            reject(f"{label} has a hard-link alias")
        _same_file(source, info, label)
    except OSError as error:
        reject(f"{label} cannot be inspected safely: {error}")
    finally:
        close(target)


def _read_file(
    root: Path, value: str, label: str, *, expected_bytes: int | None = None,
    maximum_bytes: int | None = None, expected_sha256: str | None = None, optional: bool = False,
) -> bytes | None:
    expected_sha256 = None if expected_sha256 is None else digest(expected_sha256, f"{label} expected sha256")
    try:
        parent, leaf = _parent_fd(root, value, False)
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
            before = os.fstat(target)
            if not stat.S_ISREG(before.st_mode):
                reject(f"{label} is not a regular file")
            if before.st_nlink != 1:
                reject(f"{label} has a hard-link alias")
            if expected_bytes is not None and before.st_size != expected_bytes:
                reject(f"{label} has a byte count mismatch")
            if maximum_bytes is not None and before.st_size > maximum_bytes:
                reject(f"{label} exceeds its byte limit")
            limit = expected_bytes if expected_bytes is not None else maximum_bytes
            if limit is None:
                reject(f"{label} has no read limit")
            remaining, chunks = limit + 1, []
            while remaining and (chunk := os.read(target, min(1024 * 1024, remaining))):
                chunks.append(chunk)
                remaining -= len(chunk)
            if not remaining:
                reject(f"{label} has a byte count mismatch" if expected_bytes is not None else f"{label} exceeds its byte limit")
            payload = b"".join(chunks)
            _same_read(before, os.fstat(target), label)
            if os.fstat(target).st_nlink != 1:
                reject(f"{label} has a hard-link alias")
            if expected_bytes is not None and len(payload) != expected_bytes:
                reject(f"{label} has a byte count mismatch")
            if expected_sha256 is not None and hashlib.sha256(payload).hexdigest() != expected_sha256:
                reject(f"{label} has a sha256 mismatch")
            return payload
        except OSError as error:
            reject(f"{label} cannot be read safely: {error}")
        finally:
            close(target)
    finally:
        close(parent)


def _same_parent(root: Path, value: str, held: int) -> None:
    try:
        current, _ = _parent_fd(root, value, False)
    except FileNotFoundError:
        reject("Docs staging parent changed during write")
    try:
        _same_file(os.fstat(held), os.fstat(current), "Docs staging parent")
    except OSError as error:
        reject(f"Docs staging parent cannot be inspected safely: {error}")
    finally:
        close(current)


def _write(descriptor: int, payload: bytes) -> None:
    remaining = memoryview(payload)
    while remaining:
        count = os.write(descriptor, remaining)
        if count <= 0:
            reject("Docs staging write made no progress")
        remaining = remaining[count:]
    os.fsync(descriptor)


def _atomic_file(root: Path, value: str, payload: object, label: str, *, maximum_bytes: object = MAX_OUTPUT_BYTES) -> Path:
    payload = bounded_payload(payload, maximum_bytes)
    parent, leaf = _parent_fd(root, value, True)
    temporary, descriptor, linked, temporary_info = f".stage-{uuid.uuid4().hex}.tmp", -1, False, None
    try:
        try:
            descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | NOFOLLOW, 0o600, dir_fd=parent)
            _write(descriptor, payload)
            temporary_info = os.fstat(descriptor)
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
            reject(f"Docs staging refuses to overwrite existing {label}")
        except (OSError, ValueError) as error:
            reject(f"{label} cannot be published safely: {error}")
        os.unlink(temporary, dir_fd=parent)
        temporary = ""
        os.fsync(parent)
        if temporary_info is None:
            reject(f"{label} cannot be inspected safely")
        _published(parent, leaf, temporary_info, label)
        _same_parent(root, value, parent)
        if _read_file(root, value, label, expected_bytes=len(payload)) != payload:
            reject(f"{label} changed during publish")
        return root.joinpath(*relative(value))
    except (OSError, ValueError) as error:
        reject(f"{label} cannot finalize staging safely: {error}")
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


def read_file(plan: StagingPlan, value: str, label: str, **kwargs) -> bytes | None:
    return _read_file(require_live_plan(plan).external_root, value, label, **kwargs)


def atomic_file(plan: StagingPlan, value: str, payload: object, label: str, *, maximum_bytes: object = None) -> Path:
    return _atomic_file(require_live_plan(plan).external_root, value, payload, label, maximum_bytes=maximum_bytes)
