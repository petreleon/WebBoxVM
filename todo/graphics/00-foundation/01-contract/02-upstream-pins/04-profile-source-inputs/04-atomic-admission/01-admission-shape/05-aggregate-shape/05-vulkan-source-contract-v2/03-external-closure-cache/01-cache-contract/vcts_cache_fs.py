"""Descriptor-anchored traversal and per-closure locking."""

from __future__ import annotations

import fcntl
import os
import stat
from contextlib import contextmanager
from pathlib import Path

CHUNK = 64 * 1024
HEX = set("0123456789abcdef")
NOFOLLOW = getattr(os, "O_NOFOLLOW", 0)
CLOEXEC = getattr(os, "O_CLOEXEC", 0)
DIRECTORY = getattr(os, "O_DIRECTORY", 0)


class CacheError(ValueError):
    """The cache cannot prove the requested immutable closure."""


def reject(message: str) -> None:
    raise CacheError(message)


def name(value: str) -> None:
    if not isinstance(value, str) or not value or "/" in value or value in (".", ".."):
        reject("cache path component is unsafe")


def digest(value: str) -> None:
    if not isinstance(value, str) or len(value) != 64 or set(value) - HEX:
        reject("cache digest is invalid")


def _dir_flags() -> int:
    if not NOFOLLOW:
        reject("O_NOFOLLOW is required for the external cache")
    return os.O_RDONLY | DIRECTORY | NOFOLLOW | CLOEXEC


def file(parent: int, value: str, write: bool = False, create: bool = False) -> int:
    name(value)
    flags = (os.O_RDWR if write else os.O_RDONLY) | NOFOLLOW | CLOEXEC
    if create:
        flags |= os.O_CREAT
    try:
        fd = os.open(value, flags, 0o600, dir_fd=parent)
    except FileNotFoundError:
        raise
    except OSError as error:
        reject(f"cache object cannot be opened: {error.strerror}")
    if not stat.S_ISREG(os.fstat(fd).st_mode):
        os.close(fd)
        reject("cache object is not a regular file")
    return fd


def _directory(fd: int) -> int:
    if not stat.S_ISDIR(os.fstat(fd).st_mode):
        os.close(fd)
        reject("cache object is not a directory")
    return fd


def _private(fd: int, created: bool = False) -> int:
    state = os.fstat(fd)
    if state.st_uid != os.geteuid() or state.st_mode & 0o022:
        os.close(fd)
        reject("cache directory must be current-user owned and non-writable by group or others")
    if created:
        os.fchmod(fd, 0o700)
        if stat.S_IMODE(os.fstat(fd).st_mode) != 0o700:
            os.close(fd)
            reject("new cache directory did not retain mode 0700")
    return fd


def child(parent: int, value: str, create: bool, private: bool = False) -> int:
    name(value)
    try:
        fd = _directory(os.open(value, _dir_flags(), dir_fd=parent))
        return _private(fd) if private else fd
    except FileNotFoundError:
        if not create:
            raise
        try:
            os.mkdir(value, 0o700, dir_fd=parent)
        except FileExistsError:
            pass
        except OSError as error:
            reject(f"cache directory cannot be created: {error.strerror}")
        try:
            fd = _directory(os.open(value, _dir_flags(), dir_fd=parent))
        except OSError as error:
            reject(f"cache directory cannot be opened: {error.strerror}")
        os.fsync(parent)
        return _private(fd, True) if private else fd
    except OSError as error:
        reject(f"cache directory cannot be opened: {error.strerror}")


def absolute(path: Path, repository: Path, create: bool) -> int:
    if not isinstance(path, Path) or not path.is_absolute() or path == Path(path.anchor):
        reject("cache root must be a non-root absolute path")
    if any(part in (".", "..") for part in path.parts):
        reject("cache root has an unsafe component")
    try:
        root, repo = str(path.resolve()), str(repository.resolve())
    except (OSError, RuntimeError) as error:
        reject(f"cache root cannot be resolved for overlap checking: {error}")
    if os.path.commonpath((root, repo)) in (root, repo):
        reject("cache root must not overlap the repository")
    fd = os.open(path.anchor, os.O_RDONLY | DIRECTORY | CLOEXEC)
    try:
        for part in path.parts[1:]:
            next_fd = child(fd, part, create)
            os.close(fd)
            fd = next_fd
        return _private(fd)
    except BaseException:
        try:
            os.close(fd)
        except OSError:
            pass
        raise


@contextmanager
def closure(root: Path, repository: Path, identity: str, ledger: str, create: bool, exclusive: bool):
    digest(identity)
    digest(ledger)
    fd, lock = None, None
    try:
        fd = absolute(root, repository, create)
        for part in ("webboxvm-graphics", "v2", "vulkan-cts-mustpass", identity, ledger):
            next_fd = child(fd, part, create, True)
            os.close(fd)
            fd = next_fd
        try:
            lock = file(fd, ".closure.lock", write=exclusive, create=create)
        except FileNotFoundError:
            reject("cache closure lock is missing")
        fcntl.flock(lock, fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH)
        yield fd
    except FileNotFoundError:
        reject("cache closure is missing")
    finally:
        if lock is not None:
            fcntl.flock(lock, fcntl.LOCK_UN)
            os.close(lock)
        if fd is not None:
            os.close(fd)


def subdir(closure_fd: int, value: str, create: bool) -> int:
    try:
        return child(closure_fd, value, create, True)
    except FileNotFoundError:
        reject(f"cache {value} directory is missing")
