#!/usr/bin/env python3
"""Populate or rehash an external V2 cache from an already-supplied ledger."""

from __future__ import annotations

import importlib.util
import os
import sys
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import urlsplit

HERE = Path(__file__).resolve().parent
SCHEMA = HERE.parent.parent / "02-canonical-suite-schema"
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))
import vcts_cache_io as io
import vcts_cache_input as source
import vcts_cache_receipt as receipt


def load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load reviewed module: {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


identity = load("f025_v2_cache_identity", SCHEMA / "canonical_suite_identity.py")
ledger = load("f025_v2_cache_ledger", SCHEMA / "canonical_suite_ledger.py")
ROOT_LIMIT = 8 * 1024 * 1024


@dataclass(frozen=True)
class CacheReceipt:
    digest: str
    member_count: int
    total_bytes: int
    reused: bool


def reject(message: str) -> None:
    io.reject(message)


def repository_root(anchor: Path) -> Path:
    for candidate in (anchor.resolve(), *anchor.resolve().parents):
        if (candidate / ".git").exists():
            return candidate
    reject("repository root cannot be located")


def raw_url(commit: str, path: str) -> str:
    try:
        safe = ledger.safe_path(path)
    except ledger.LedgerError as error:
        reject(f"unsafe VCTS path: {error}")
    if not isinstance(commit, str) or not identity.HEX40.fullmatch(commit) or safe != path:
        reject("unsafe VCTS commit or path")
    result = f"https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/{commit}/{path}"
    parsed = urlsplit(result)
    if (parsed.scheme, parsed.netloc, parsed.query, parsed.fragment) != ("https", "raw.githubusercontent.com", "", ""):
        reject("raw VCTS URL is not immutable HTTPS")
    return result


def inputs(identity_path: Path, ledger_path: Path):
    try:
        root = identity.validate(identity_path)
        with source.ledger_snapshot(ledger_path, ledger.LIMITS["max_ledger_json_bytes"]) as (frozen, fd):
            os.lseek(fd, 0, os.SEEK_SET)
            checked = ledger.validate(frozen, identity_path)
            os.lseek(fd, 0, os.SEEK_SET)
            document = ledger.document(frozen)
    except (identity.IdentityError, ledger.LedgerError) as error:
        reject(f"reviewed closure input is invalid: {error}")
    if (document.get("ledger_sha256") != checked.digest or document.get("root_identity_sha256") != root.digest):
        reject("ledger changed after validation")
    return root, checked, document


def root_expected(root) -> dict[str, object]:
    value = {"sha256": identity.ROOT["sha256"], "bytes": identity.ROOT["bytes"]}
    if raw_url(root.peeled_commit, root.root_path) != identity.ROOT["immutable_url"]:
        reject("root selector URL is not the reviewed immutable URL")
    return value


def member_expected(row: dict[str, object]) -> dict[str, object]:
    return {key: row[key] for key in ("sha256", "blob_sha1", "bytes")}


def content_name(digest: str) -> str:
    return digest + ".source"


def verify_fd(cache_fd: int, root, document: dict[str, object]) -> None:
    root_fd = io.subdir(cache_fd, "root", False)
    try:
        members_fd = io.subdir(cache_fd, "members", False)
        try:
            expected = root_expected(root)
            if not io.existing(root_fd, content_name(str(expected["sha256"])), expected, ROOT_LIMIT):
                reject("cache selector root is missing")
            io.selector(root_fd, content_name(str(expected["sha256"])), root.direct_members, ROOT_LIMIT)
            for row in document["members"]:
                expected = member_expected(row)
                if not io.existing(members_fd, content_name(str(expected["sha256"])), expected,
                                   ledger.LIMITS["max_member_bytes"]):
                    reject("cache member is missing")
            actual = receipt.parse(io.read(cache_fd, receipt.name(document), 16 * 1024))
            expected_marker = receipt.marker(document, root.digest, identity.ROOT["sha256"], ledger.LIMITS)
            if actual != expected_marker:
                reject("cache receipt does not bind this exact closure")
        finally:
            os.close(members_fd)
    finally:
        os.close(root_fd)


def verify(identity_path: Path, ledger_path: Path, external_root: Path, repository: Path) -> CacheReceipt:
    root, checked, document = inputs(identity_path, ledger_path)
    with io.closure(external_root, repository, root.digest, checked.digest, False, False) as cache_fd:
        verify_fd(cache_fd, root, document)
    return CacheReceipt(checked.digest, checked.member_count, checked.total_bytes, True)


def populate(identity_path: Path, ledger_path: Path, external_root: Path, repository: Path,
             timeout: float = 60.0, opener=None) -> CacheReceipt:
    root, checked, document = inputs(identity_path, ledger_path)
    with io.closure(external_root, repository, root.digest, checked.digest, True, True) as cache_fd:
        try:
            io.read(cache_fd, receipt.name(document), 16 * 1024)
        except FileNotFoundError:
            pass
        else:
            verify_fd(cache_fd, root, document)
            return CacheReceipt(checked.digest, checked.member_count, checked.total_bytes, True)
        root_fd = io.subdir(cache_fd, "root", True)
        try:
            members_fd = io.subdir(cache_fd, "members", True)
            try:
                expected = root_expected(root)
                reused = io.store(root_fd, content_name(str(expected["sha256"])), expected, ROOT_LIMIT,
                                  io.chunks(identity.ROOT["immutable_url"], timeout, expected, opener))
                for row in document["members"]:
                    expected = member_expected(row)
                    reused &= io.store(members_fd, content_name(str(expected["sha256"])), expected,
                                       ledger.LIMITS["max_member_bytes"],
                                       io.chunks(raw_url(str(row["revision"]), str(row["path"])), timeout, expected, opener))
                io.publish(cache_fd, receipt.name(document),
                           receipt.payload(receipt.marker(document, root.digest, identity.ROOT["sha256"], ledger.LIMITS)))
                verify_fd(cache_fd, root, document)
                return CacheReceipt(checked.digest, checked.member_count, checked.total_bytes, reused)
            finally:
                os.close(members_fd)
        finally:
            os.close(root_fd)
