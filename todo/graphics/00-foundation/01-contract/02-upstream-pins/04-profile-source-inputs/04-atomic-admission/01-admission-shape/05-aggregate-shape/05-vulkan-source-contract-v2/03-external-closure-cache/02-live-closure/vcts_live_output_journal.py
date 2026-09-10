"""Lock and recover one private output transaction without guessing ownership."""

from __future__ import annotations

import fcntl
import os
import secrets
from dataclasses import dataclass
from pathlib import Path

import vcts_live_output_io as io

JOURNAL, PREFIX, JournalError, reject = io.JOURNAL, io.PREFIX, io.JournalError, io.reject

@dataclass
class Transaction:
    paths: tuple[Path, Path, Path]
    nonce: str
    _fd: int
    _identity: tuple[int, int]

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()
    def close(self) -> None:
        if self._fd >= 0:
            fd, self._fd = self._fd, -1
            try:
                fcntl.flock(fd, fcntl.LOCK_UN)
            finally:
                os.close(fd)
def _close(fd: int) -> None:
    if fd >= 0:
        try:
            os.close(fd)
        except OSError:
            pass
def _discard_new(parent: int, fd: int) -> None:
    try:
        original, named = os.fstat(fd), os.stat(JOURNAL, dir_fd=parent, follow_symlinks=False)
        if (original.st_dev, original.st_ino) != (named.st_dev, named.st_ino):
            reject("capture output journal changed before initialization cleanup")
        os.unlink(JOURNAL, dir_fd=parent)
        os.fsync(parent)
    except OSError as error:
        reject(io.failure("capture output journal initialization cleanup", error))


def _abandon(parent: int, fd: int) -> None:
    try:
        if fd >= 0:
            _discard_new(parent, fd)
    finally:
        _close(fd)


def _new(parent: int, paths: tuple[Path, Path, Path]) -> Transaction:
    fd, nonce = -1, secrets.token_hex(32)
    try:
        fd = os.open(JOURNAL, os.O_RDWR | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0), 0o600, dir_fd=parent)
        os.fchmod(fd, 0o600)
        io.lock(fd)
        io.write(fd, io.record_data(paths, parent, nonce))
        os.fsync(fd)
        os.fsync(parent)
        state = os.fstat(fd)
        return Transaction(paths, nonce, fd, (state.st_dev, state.st_ino))
    except OSError as error:
        _abandon(parent, fd)
        reject(io.failure("capture output journal create", error))
    except BaseException:
        _abandon(parent, fd)
        raise


def _open(parent: int, paths: tuple[Path, Path, Path]) -> Transaction:
    fd = -1
    try:
        fd = os.open(JOURNAL, os.O_RDWR | getattr(os, "O_NOFOLLOW", 0), dir_fd=parent)
        io.lock(fd)
        nonce, state = io.record(io.read(fd), paths, parent), os.fstat(fd)
        return Transaction(paths, nonce, fd, (state.st_dev, state.st_ino))
    except OSError as error:
        _close(fd)
        reject(io.failure("capture output journal open", error))
    except BaseException:
        _close(fd)
        raise


def _unlink(parent: int, name: str) -> None:
    try:
        os.unlink(name, dir_fd=parent)
    except OSError as error:
        reject(io.failure("capture output recovery", error))


def _recover(parent: int, transaction: Transaction) -> None:
    try:
        names = set(os.listdir(parent))
    except OSError as error:
        reject(io.failure("capture output scan", error))
    allowed = {JOURNAL, *(path.name for path in transaction.paths)}
    staged = [name for name in names if io.stage(name, transaction.nonce)]
    allowed.update(staged)
    if names - allowed:
        reject("capture output directory contains artifacts outside this transaction")
    if transaction.paths[2].name in names:
        for name in staged:
            value = io.state(parent, name)
            if value is None or not io.private(value):
                reject("committed capture staging artifact is not private regular content")
        for name in staged:
            _unlink(parent, name)
        try:
            os.fsync(parent)
        except OSError as error:
            reject(io.failure("capture committed staging cleanup", error))
        reject("capture receipt already exists; output bundle is immutable")
    rows = (*[path.name for path in transaction.paths[:2]], *staged)
    for name in rows:
        value = io.state(parent, name)
        if value is not None:
            if not io.private(value):
                reject("recoverable capture artifact is not private regular content")
            _unlink(parent, name)
    try:
        os.fsync(parent)
    except OSError as error:
        reject(io.failure("capture output recovery", error))


def preflight(paths: tuple[Path, Path, Path]) -> Transaction:
    """Lock a fresh output directory or safely recover this exact interrupted transaction."""
    parent, transaction = io.parent(paths), None
    try:
        try:
            names = set(os.listdir(parent))
        except OSError as error:
            reject(io.failure("capture output scan", error))
        if JOURNAL not in names:
            if names:
                reject("capture output directory is not fresh and has no recoverable transaction journal")
            transaction = _new(parent, paths)
        else:
            transaction = _open(parent, paths)
            _recover(parent, transaction)
        return transaction
    except BaseException:
        if transaction is not None:
            transaction.close()
        raise
    finally:
        _close(parent)


def ready(transaction: Transaction) -> int:
    """Open the locked transaction's parent only when it still names the same journal."""
    if not isinstance(transaction, Transaction) or transaction._fd < 0:
        reject("capture output transaction is closed")
    parent = io.parent(transaction.paths)
    try:
        state = os.stat(JOURNAL, dir_fd=parent, follow_symlinks=False)
        same = (state.st_dev, state.st_ino) == transaction._identity
        if not same or io.record(io.read(transaction._fd), transaction.paths, parent) != transaction.nonce:
            reject("capture output transaction journal changed")
        if set(os.listdir(parent)) != {JOURNAL}:
            reject("capture output transaction no longer has an empty output set")
        return parent
    except BaseException:
        _close(parent)
        raise


def stage_name(nonce: str) -> str:
    return f"{PREFIX}{nonce}-{secrets.token_hex(16)}"
