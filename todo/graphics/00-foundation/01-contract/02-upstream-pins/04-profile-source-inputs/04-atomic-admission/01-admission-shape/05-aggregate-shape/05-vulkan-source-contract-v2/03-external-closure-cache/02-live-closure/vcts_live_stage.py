#!/usr/bin/env python3
"""Stream one immutable VCTS tree plan into a descriptor-anchored local stage."""

from __future__ import annotations

import hashlib
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE = HERE.parent / "01-cache-contract"
for location in (HERE, CACHE):
    if str(location) not in sys.path:
        sys.path.insert(0, str(location))
import vcts_cache_content as content
import vcts_cache_contract as cache
import vcts_cache_fs as fs
import vcts_cache_input as snapshot
import vcts_cache_transport as transport
import vcts_tree_plan as plan
import vcts_tree_plan_input as plan_input
from vcts_live_cleanup import discard_payloads
from vcts_live_replay import LiveStage, Payload

StageError = fs.CacheError
ROOT_LIMIT = 8 * 1024 * 1024


def reject(message: str) -> None:
    fs.reject(message)


def _write(fd: int, data: bytes) -> None:
    view = memoryview(data)
    while view:
        count = os.write(fd, view)
        if count < 1:
            reject("live stage write made no progress")
        view = view[count:]


def _capture(fd: int, name: str, url: str, path: str, revision: str, expected: dict[str, object], chunks,
             members=()) -> Payload:
    temporary, output = ".part-" + name, -1
    try:
        output = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | fs.NOFOLLOW | fs.CLOEXEC, 0o600, dir_fd=fd)
        total, sha256 = 0, hashlib.sha256()
        blob = hashlib.sha1(f"blob {expected['bytes']}\0".encode())
        for chunk in chunks:
            if not isinstance(chunk, bytes) or len(chunk) > fs.CHUNK:
                reject("live raw response is not bounded bytes")
            total += len(chunk)
            if total > expected["bytes"]:
                reject("live raw response exceeds its planned byte count")
            _write(output, chunk)
            sha256.update(chunk)
            blob.update(chunk)
        os.fsync(output)
        try:
            os.close(output)
        finally:
            output = -1
        if (total != expected["bytes"] or sha256.hexdigest() != expected.get("sha256", sha256.hexdigest())
                or blob.hexdigest() != expected["blob_sha1"]):
            reject("live raw response does not match the immutable plan")
        os.link(temporary, name, src_dir_fd=fd, dst_dir_fd=fd, follow_symlinks=False)
        os.unlink(temporary, dir_fd=fd)
        os.fsync(fd)
        if members:
            content.selector(fd, name, members, ROOT_LIMIT)
        return Payload(name, url, path, revision, blob.hexdigest(), sha256.hexdigest(), total)
    finally:
        closer = getattr(chunks, "close", None)
        try:
            if output >= 0:
                try:
                    os.close(output)
                finally:
                    output = -1
        finally:
            try:
                os.unlink(temporary, dir_fd=fd)
            except FileNotFoundError:
                pass
            finally:
                if closer is not None:
                    closer()


def _directory(root: Path, repository: Path, identity_digest: str, plan_digest: str) -> tuple[int, int, Path]:
    fs.digest(identity_digest)
    fs.digest(plan_digest)
    parent, result, copied, created = fs.absolute(root, repository, True), -1, -1, False
    try:
        for name in ("webboxvm-graphics", "v2", "vcts-live-stage", identity_digest):
            child = fs.child(parent, name, True, True)
            os.close(parent)
            parent = child
        try:
            os.mkdir(plan_digest, 0o700, dir_fd=parent)
        except FileExistsError:
            reject("live stage already exists; use a new disposable external root")
        created = True
        result = fs.child(parent, plan_digest, False, True)
        os.fsync(parent)
        copied = os.dup(parent)
        returned, result, copied = (result, copied), -1, -1
        return (*returned, root / "webboxvm-graphics" / "v2" / "vcts-live-stage" / identity_digest / plan_digest)
    except BaseException as failure:
        for descriptor in (result, copied):
            if descriptor >= 0:
                try:
                    os.close(descriptor)
                except OSError:
                    pass
        if created:
            try:
                os.rmdir(plan_digest, dir_fd=parent)
                os.fsync(parent)
            except OSError as cleanup:
                reject(f"live stage setup failed ({failure}); cleanup failed: {cleanup}")
        raise
    finally:
        os.close(parent)


def _inputs(identity_path: Path, plan_path: Path):
    with snapshot.ledger_snapshot(plan_path, plan_input.MAX_PLAN_BYTES) as (frozen, fd):
        os.lseek(fd, 0, os.SEEK_SET)
        checked = plan.validate(frozen, identity_path)
        os.lseek(fd, 0, os.SEEK_SET)
        document, root = plan.document(frozen), plan.checked_identity(identity_path)
    if (document["root"]["path"], document["root"]["bytes"]) != (root.root_path, plan.identity.ROOT["bytes"]):
        reject("tree plan root does not bind the reviewed V2 selector")
    return checked, document, root


def capture(identity_path: Path, plan_path: Path, stage_root: Path, repository: Path, timeout=60.0, opener=None) -> LiveStage:
    """Capture the plan's root and 98 members once; caller owns the returned stage."""
    checked, document, root = _inputs(identity_path, plan_path)
    try:
        fd, parent_fd, directory = _directory(stage_root, repository, root.digest, checked.digest)
    except OSError as error:
        reject(f"live stage setup failed (errno {error.errno}, path {error.filename or '<unknown>'}): {error.strerror}")
    names: list[str] = []
    try:
        root_expected = {"bytes": plan.identity.ROOT["bytes"], "sha256": plan.identity.ROOT["sha256"],
                         "blob_sha1": document["root"]["blob_sha1"]}
        names.extend(("root.source", ".part-root.source"))
        rows = [_capture(fd, names[-2], plan.identity.ROOT["immutable_url"], root.root_path, root.peeled_commit,
                         root_expected, transport.chunks(plan.identity.ROOT["immutable_url"], timeout, root_expected, opener),
                         root.direct_members)]
        for index, member in enumerate(document["members"]):
            expected = {key: member[key] for key in ("bytes", "blob_sha1")}
            url = cache.raw_url(root.peeled_commit, member["path"])
            names.extend((f"member-{index:03d}.source", f".part-member-{index:03d}.source"))
            rows.append(_capture(fd, names[-2], url, member["path"], root.peeled_commit, expected,
                                 transport.chunks(url, timeout, expected, opener)))
        if len({row.url for row in rows}) != 99:
            reject("live stage does not have exactly 99 distinct planned raw URLs")
        return LiveStage(directory, rows[0], tuple(rows[1:]), fd, parent_fd, checked.digest)
    except BaseException as failure:
        try:
            discard_payloads(fd, parent_fd, checked.digest, tuple(names))
        except StageError as cleanup:
            reject(f"live stage capture failed ({failure}); cleanup also failed: {cleanup}")
        if isinstance(failure, OSError):
            reject(f"live stage capture failed (errno {failure.errno}, path {failure.filename or '<unknown>'}): {failure.strerror}")
        raise
