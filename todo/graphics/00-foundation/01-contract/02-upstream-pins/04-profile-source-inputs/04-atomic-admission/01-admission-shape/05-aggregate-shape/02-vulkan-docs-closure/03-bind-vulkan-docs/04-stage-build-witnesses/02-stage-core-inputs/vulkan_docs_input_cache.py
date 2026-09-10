"""One held-descriptor cache transaction for unadmitted Docs input staging."""

from __future__ import annotations

from contextlib import contextmanager
import hashlib
import os
import stat
import uuid

from vulkan_docs_input_model import reject
from vulkan_docs_stage_fs import FILE_FLAGS, _published, _same_read, _write
from vulkan_docs_stage_model import StagingPlan, bounded_payload, require_live_plan
from vulkan_docs_stage_parse import digest
from vulkan_docs_stage_paths import _directory_fd, _private, child, close, relative

class Cache:
    def __init__(self, plan: StagingPlan, descriptor: int) -> None:
        self.plan, self.descriptor = require_live_plan(plan), descriptor

    def _parent(self, value: str, create: bool) -> tuple[int, str]:
        parts, descriptor = relative(value), os.dup(self.descriptor)
        try:
            for part in parts[:-1]:
                next_descriptor = child(descriptor, part, create)
                try:
                    _private(next_descriptor, "input-cache directory")
                except BaseException:
                    close(next_descriptor)
                    raise
                close(descriptor)
                descriptor = next_descriptor
            return descriptor, parts[-1]
        except BaseException:
            close(descriptor)
            raise

    def read(
        self, value: str, label: str, *, expected_bytes: int | None = None, maximum_bytes: int | None = None,
        expected_sha256: str | None = None, optional: bool = False,
    ) -> bytes | None:
        expected_sha256 = None if expected_sha256 is None else digest(expected_sha256, f"{label} expected sha256")
        try:
            parent, leaf = self._parent(value, False)
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
                if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1:
                    reject(f"{label} is not an unaliased regular file")
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
                payload, after = b"".join(chunks), os.fstat(target)
                _same_read(before, after, label)
                if after.st_nlink != 1:
                    reject(f"{label} has a hard-link alias")
                if expected_bytes is not None and len(payload) != expected_bytes:
                    reject(f"{label} has a byte count mismatch")
                if expected_sha256 is not None and hashlib.sha256(payload).hexdigest() != expected_sha256:
                    reject(f"{label} has a sha256 mismatch")
                self._same_parent(value, parent)
                return payload
            except OSError as error:
                reject(f"{label} cannot be read safely: {error}")
            finally:
                close(target)
        finally:
            close(parent)

    def _same_parent(self, value: str, held: int) -> None:
        try:
            current, unused = self._parent(value, False)
        except FileNotFoundError:
            reject("Docs input-cache parent changed during publication")
        try:
            left, right = os.fstat(held), os.fstat(current)
            if (left.st_dev, left.st_ino) != (right.st_dev, right.st_ino):
                reject("Docs input-cache parent changed during publication")
        except OSError as error:
            reject(f"Docs input-cache parent cannot be inspected safely: {error}")
        finally:
            close(current)

    def atomic(self, value: str, payload: object, label: str, *, maximum_bytes: object) -> None:
        payload = bounded_payload(payload, maximum_bytes)
        parent, leaf = self._parent(value, True)
        temporary, descriptor, linked, info = f".stage-{uuid.uuid4().hex}.tmp", -1, False, None
        try:
            try:
                descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0), 0o600, dir_fd=parent)
                _write(descriptor, payload)
                info = os.fstat(descriptor)
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
                reject(f"Docs input staging refuses to overwrite existing {label}")
            except (OSError, ValueError) as error:
                reject(f"{label} cannot be published safely: {error}")
            os.unlink(temporary, dir_fd=parent)
            temporary = ""
            os.fsync(parent)
            if info is None:
                reject(f"{label} cannot be inspected safely")
            _published(parent, leaf, info, label)
            self._same_parent(value, parent)
            if self.read(value, label, expected_bytes=len(payload)) != payload:
                reject(f"{label} changed during publish")
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

    def confirm(self) -> None:
        _private(self.descriptor, "input-cache root")
        try:
            current = _directory_fd(self.plan.external_root, False)
        except OSError as error:
            reject(f"Docs input-cache root cannot be reopened safely: {error}")
        try:
            _private(current, "input-cache root")
            held, named = os.fstat(self.descriptor), os.fstat(current)
            if (held.st_dev, held.st_ino) != (named.st_dev, named.st_ino):
                reject("Docs input-cache root changed during staging")
        except OSError as error:
            reject(f"Docs input-cache root cannot be inspected safely: {error}")
        finally:
            close(current)

@contextmanager
def cache_session(plan: StagingPlan, *, create: bool = False):
    plan = require_live_plan(plan)
    try:
        descriptor = _directory_fd(plan.external_root, create)
    except OSError as error:
        reject(f"Docs input-cache root cannot be opened safely: {error}")
    try:
        yield Cache(plan, descriptor)
    finally:
        close(descriptor)
