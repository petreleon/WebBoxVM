"""Publish a locked capture transaction; its self-hashed receipt is the commit marker."""

from __future__ import annotations

import os

import vcts_live_output_journal as journal

PREFIX = journal.PREFIX
PublishError = journal.JournalError
reject = journal.reject
preflight = journal.preflight


def _failure(action: str, error: OSError) -> str:
    return f"{action} failed (errno {error.errno}, path {error.filename or '<unknown>'}): {error.strerror or error}"


def _stage(fd: int, nonce: str, payload: bytes) -> str:
    if not isinstance(payload, bytes) or not payload:
        reject("capture output payload is invalid")
    for _ in range(16):
        name, temporary, complete, created = journal.stage_name(nonce), -1, False, False
        try:
            temporary = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0), 0o600, dir_fd=fd)
            created = True
            view = memoryview(payload)
            while view:
                written = os.write(temporary, view)
                if written < 1:
                    reject("capture output staging made no progress")
                view = view[written:]
            os.fsync(temporary)
            try:
                os.close(temporary)
            finally:
                temporary = -1
            complete = True
            return name
        except FileExistsError:
            continue
        except OSError as error:
            reject(_failure("capture output staging", error))
        finally:
            if temporary >= 0:
                try:
                    os.close(temporary)
                except OSError:
                    pass
            if created and not complete:
                try:
                    os.unlink(name, dir_fd=fd)
                except FileNotFoundError:
                    pass
                except OSError as error:
                    reject(_failure("capture output staging cleanup", error))
    reject("capture output staging names are exhausted")


def _same(fd: int, name: str, expected: tuple[int, int]) -> bool:
    try:
        state = os.stat(name, dir_fd=fd, follow_symlinks=False)
    except OSError:
        return False
    return (state.st_dev, state.st_ino) == expected


def _remove(fd: int, names: list[str]) -> OSError | None:
    for name in names:
        try:
            os.unlink(name, dir_fd=fd)
        except FileNotFoundError:
            pass
        except OSError as error:
            return error
    return None


def publish(transaction: journal.Transaction, payloads: tuple[bytes, bytes, bytes]) -> None:
    """Expose plan and ledger first, then link the immutable receipt last."""
    if len(payloads) != 3:
        reject("capture publication must have exactly three payloads")
    fd, staged, committed, complete = journal.ready(transaction), [], [], False
    try:
        staged = [_stage(fd, transaction.nonce, payload) for payload in payloads]
        for temporary, path in zip(staged[:2], transaction.paths[:2]):
            state = os.stat(temporary, dir_fd=fd, follow_symlinks=False)
            os.link(temporary, path.name, src_dir_fd=fd, dst_dir_fd=fd, follow_symlinks=False)
            committed.append((path.name, (state.st_dev, state.st_ino)))
        cleanup = _remove(fd, staged[:2])
        if cleanup is not None:
            raise cleanup
        staged = staged[2:]
        os.fsync(fd)
        os.link(staged[0], transaction.paths[2].name, src_dir_fd=fd, dst_dir_fd=fd, follow_symlinks=False)
        complete = True
        cleanup = _remove(fd, staged)
        if cleanup is None:
            staged = []
        os.fsync(fd)
        if cleanup is not None:
            reject(_failure("capture receipt staging cleanup", cleanup))
    except OSError as error:
        if not complete:
            for name, expected in reversed(committed):
                if _same(fd, name, expected):
                    _remove(fd, [name])
            reject(_failure("capture output publication", error))
        reject(_failure("capture receipt durable commit", error))
    finally:
        if not complete:
            _remove(fd, staged)
        try:
            os.close(fd)
        except OSError:
            pass
