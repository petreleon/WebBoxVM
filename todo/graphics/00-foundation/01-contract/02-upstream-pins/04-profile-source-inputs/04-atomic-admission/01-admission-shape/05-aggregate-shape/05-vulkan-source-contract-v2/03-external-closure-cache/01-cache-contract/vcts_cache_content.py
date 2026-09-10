"""Bounded content verification and link-only cache publication."""

from __future__ import annotations

import hashlib
import os
import secrets
import stat

from vcts_cache_fs import CHUNK, CLOEXEC, NOFOLLOW, file, name, reject


def _stage(parent: int) -> tuple[str, int]:
    for _ in range(16):
        value = ".stage-" + secrets.token_hex(16)
        try:
            return value, os.open(value, os.O_WRONLY | os.O_CREAT | os.O_EXCL | NOFOLLOW | CLOEXEC,
                                  0o600, dir_fd=parent)
        except FileExistsError:
            continue
    reject("cannot allocate a private cache staging file")


def _write(fd: int, chunk: bytes) -> None:
    view = memoryview(chunk)
    while view:
        count = os.write(fd, view)
        if count < 1:
            reject("cache staging write made no progress")
        view = view[count:]


def _observed(fd: int, expected: dict[str, object], limit: int) -> tuple[str, str, int]:
    state, size = os.fstat(fd), os.fstat(fd).st_size
    if (not stat.S_ISREG(state.st_mode) or type(expected.get("bytes")) is not int
            or size != expected["bytes"] or size < 1 or size > limit):
        reject("cache content has an invalid byte count")
    os.lseek(fd, 0, os.SEEK_SET)
    sha256, blob = hashlib.sha256(), hashlib.sha1(f"blob {size}\0".encode())
    while chunk := os.read(fd, CHUNK):
        sha256.update(chunk)
        blob.update(chunk)
    if os.fstat(fd).st_size != size:
        reject("cache content changed while hashing")
    result = sha256.hexdigest(), blob.hexdigest(), size
    if result[0] != expected.get("sha256") or expected.get("blob_sha1") not in (None, result[1]):
        reject("cache content does not match its declared identity")
    return result


def existing(parent: int, value: str, expected: dict[str, object], limit: int) -> bool:
    try:
        fd = file(parent, value)
    except FileNotFoundError:
        return False
    try:
        _observed(fd, expected, limit)
        return True
    finally:
        os.close(fd)


def store(parent: int, value: str, expected: dict[str, object], limit: int, chunks) -> bool:
    name(value)
    if existing(parent, value, expected, limit):
        return True
    stage, fd = _stage(parent)
    try:
        sha256, blob, total = hashlib.sha256(), hashlib.sha1(f"blob {expected['bytes']}\0".encode()), 0
        for chunk in chunks:
            if not isinstance(chunk, bytes) or len(chunk) > CHUNK:
                reject("cache response is not bounded bytes")
            total += len(chunk)
            if total > limit or total > expected["bytes"]:
                reject("cache response exceeds its declared byte limit")
            _write(fd, chunk)
            sha256.update(chunk)
            blob.update(chunk)
        os.fsync(fd)
        os.close(fd)
        fd = -1
        if (total != expected["bytes"] or sha256.hexdigest() != expected.get("sha256")
                or expected.get("blob_sha1") not in (None, blob.hexdigest())):
            reject("cache response does not match its declared identity")
        try:
            os.link(stage, value, src_dir_fd=parent, dst_dir_fd=parent, follow_symlinks=False)
        except FileExistsError:
            if not existing(parent, value, expected, limit):
                reject("cache publication race has no valid winner")
            return True
        os.fsync(parent)
        return False
    finally:
        closer = getattr(chunks, "close", None)
        if closer is not None:
            closer()
        if fd >= 0:
            os.close(fd)
        try:
            os.unlink(stage, dir_fd=parent)
            os.fsync(parent)
        except FileNotFoundError:
            pass


def publish(parent: int, value: str, payload: bytes) -> bool:
    if not isinstance(payload, bytes) or not payload or len(payload) > CHUNK:
        reject("cache receipt payload is invalid")
    expected = {"sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload)}
    return store(parent, value, expected, len(payload), (payload,))


def read(parent: int, value: str, limit: int) -> bytes:
    fd = file(parent, value)
    try:
        size = os.fstat(fd).st_size
        if size < 1 or size > limit:
            reject("cache metadata size is invalid")
        result = bytearray()
        while chunk := os.read(fd, min(CHUNK, size + 1 - len(result))):
            result.extend(chunk)
            if len(result) > size:
                reject("cache metadata changed while reading")
        if len(result) != size:
            reject("cache metadata changed while reading")
        return bytes(result)
    finally:
        os.close(fd)


def selector(parent: int, value: str, expected: tuple[str, ...], limit: int) -> None:
    fd = file(parent, value)
    try:
        if os.fstat(fd).st_size > limit:
            reject("selector exceeds its byte limit")
        prefix, pending, index = "external/vulkancts/mustpass/main/", b"", 0
        while chunk := os.read(fd, CHUNK):
            if b"\r" in chunk:
                reject("selector must not use CRLF")
            pending += chunk
            while b"\n" in pending:
                line, pending = pending.split(b"\n", 1)
                if len(line) > 4096:
                    reject("selector line is too long")
                try:
                    path = prefix + line.decode("ascii")
                except UnicodeDecodeError:
                    reject("selector is not ASCII")
                if index >= len(expected) or not line or path != expected[index]:
                    reject("selector does not preserve the pinned member order")
                index += 1
        if pending or index != len(expected):
            reject("selector must end in one terminal LF with every pinned member")
    finally:
        os.close(fd)
