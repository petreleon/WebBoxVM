"""Per-closure locks that remain bound to the descriptor-opened cache tree."""

from __future__ import annotations

import fcntl
import os
import stat
from contextlib import contextmanager
from pathlib import Path

from successor_cache_fs import same_parent
from successor_cache_model import CacheMiss, reject
from successor_cache_paths import NOFOLLOW, close, parent_fd


def _same_lock(parent: int, leaf: str, held: int) -> None:
    try:
        named, opened = os.stat(leaf, dir_fd=parent, follow_symlinks=False), os.fstat(held)
    except OSError as error:
        reject(f"successor closure lock changed during use: {error}")
    if (not stat.S_ISREG(named.st_mode)
            or (named.st_dev, named.st_ino) != (opened.st_dev, opened.st_ino)):
        reject("successor closure lock changed during use")


@contextmanager
def cache_lock(root: Path, value: str, exclusive: bool, create: bool):
    try:
        parent, leaf = parent_fd(root, value, create)
    except FileNotFoundError:
        raise CacheMiss("successor closure lock is missing") from None
    descriptor, acquired = -1, False
    try:
        flags = (os.O_RDWR if exclusive or create else os.O_RDONLY) | NOFOLLOW | getattr(os, "O_CLOEXEC", 0)
        if create:
            flags |= os.O_CREAT
        try:
            descriptor = os.open(leaf, flags, 0o600, dir_fd=parent)
        except OSError as error:
            reject(f"successor closure lock is a symlink or unsafe: {error}")
        try:
            if not stat.S_ISREG(os.fstat(descriptor).st_mode):
                reject("successor closure lock is not a regular file")
            fcntl.flock(descriptor, fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH)
            acquired = True
        except OSError as error:
            reject(f"successor closure lock cannot be acquired safely: {error}")
        def guard() -> None:
            same_parent(root, value, parent)
            _same_lock(parent, leaf, descriptor)
        guard()
        yield guard
        guard()
    finally:
        try:
            if descriptor >= 0:
                try:
                    if acquired:
                        try:
                            fcntl.flock(descriptor, fcntl.LOCK_UN)
                        except OSError as error:
                            reject(f"successor closure lock cannot be released safely: {error}")
                finally:
                    close(descriptor)
        finally:
            close(parent)
