"""Freeze a bounded supplied ledger before it is validated or consumed."""

from __future__ import annotations

import os
import stat
import tempfile
from contextlib import contextmanager
from pathlib import Path

from vcts_cache_fs import CHUNK, CLOEXEC, NOFOLLOW, reject


@contextmanager
def ledger_snapshot(path: Path, limit: int):
    if not isinstance(path, Path) or type(limit) is not int or limit < 1:
        reject("ledger snapshot arguments are invalid")
    if not NOFOLLOW:
        reject("O_NOFOLLOW is required for the supplied ledger")
    try:
        source = os.open(path, os.O_RDONLY | NOFOLLOW | CLOEXEC)
    except OSError as error:
        reject(f"ledger cannot be safely opened: {error.strerror}")
    try:
        before = os.fstat(source)
        if not stat.S_ISREG(before.st_mode) or before.st_size < 1 or before.st_size > limit:
            reject("ledger snapshot violates its byte limit")
        with tempfile.TemporaryFile() as frozen:
            total = 0
            while chunk := os.read(source, CHUNK):
                total += len(chunk)
                if total > limit:
                    reject("ledger changed beyond its byte limit while copying")
                frozen.write(chunk)
            after = os.fstat(source)
            if total != before.st_size or (after.st_ino, after.st_size) != (before.st_ino, before.st_size):
                reject("ledger changed while its snapshot was copied")
            frozen.flush()
            os.fsync(frozen.fileno())
            yield Path("/dev/fd") / str(frozen.fileno()), frozen.fileno()
    finally:
        os.close(source)
