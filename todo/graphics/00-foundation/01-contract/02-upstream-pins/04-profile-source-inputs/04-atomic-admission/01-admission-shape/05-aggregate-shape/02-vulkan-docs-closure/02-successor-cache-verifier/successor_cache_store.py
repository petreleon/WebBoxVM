"""Validated successor-member paths and descriptor-anchored staging."""

from __future__ import annotations

import hashlib
from pathlib import Path

from successor_cache_fs import atomic_file, read_file
from successor_cache_model import MemberMiss, reject
from successor_cache_paths import relative


def cache_root(value: Path, repository: Path) -> Path:
    if not isinstance(value, Path) or not isinstance(repository, Path) or not value.is_absolute():
        reject("successor cache root must be an explicit absolute path")
    root = _no_symlink_components(value)
    try:
        repo = repository.resolve(strict=False)
    except (OSError, ValueError, RuntimeError) as error:
        reject(f"successor cache root cannot be resolved safely: {error}")
    if root == Path(root.anchor):
        reject("successor cache root cannot be a filesystem root")
    if _inside_repository(root, repo):
        reject("successor cache root must stay outside the repository")
    return root


def _no_symlink_components(value: Path) -> Path:
    if "\x00" in str(value) or any(part in (".", "..") for part in value.parts):
        reject("successor cache root has an unsafe path component")
    current = Path(value.anchor)
    for part in value.parts[1:]:
        current /= part
        try:
            symlink = current.is_symlink()
        except (OSError, ValueError, RuntimeError) as error:
            reject(f"successor cache root cannot be resolved safely: {error}")
        if symlink:
            reject("successor cache root is a symlink" if current == value
                   else "successor cache root has a symlink component")
    return current


def _inside_repository(root: Path, repository: Path) -> bool:
    for candidate in (root, *root.parents):
        try:
            if candidate.exists() and candidate.samefile(repository):
                return True
        except (OSError, ValueError, RuntimeError) as error:
            reject(f"successor cache root cannot be resolved safely: {error}")
    return False


def target(root: Path, value: str) -> Path:
    """Return a lexical path only; all cache I/O uses descriptor operations."""
    return root.joinpath(*relative(value))


def _token(value: object, label: str) -> str:
    if not isinstance(value, str) or not value or "/" in value or value in (".", ".."):
        reject(f"successor {label} is unsafe")
    return value


def member_path(member, logical_id: str) -> str:
    identifier = _token(getattr(member, "identifier", None), "member identifier")
    digest = _token(getattr(member, "digest", None), "member digest")
    if len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
        reject("successor member digest is unsafe")
    logical = _token(logical_id, "logical id")
    raw = hasattr(member, "immutable_url")
    kind, suffix = ("raw", "source") if raw else ("generated", "derived")
    expected = f"webboxvm-graphics/successor/{logical}/{kind}/{identifier}/{digest}.{suffix}"
    if getattr(member, "cache_path", None) != expected:
        reject("successor member cache path escapes its validated namespace")
    return expected


def verify(member, payload: bytes) -> None:
    identifier = getattr(member, "identifier", "member")
    if not isinstance(payload, bytes):
        reject(f"successor member {identifier} is not bytes")
    if len(payload) != member.byte_count:
        reject(f"successor member {identifier} has a byte count mismatch")
    if hashlib.sha256(payload).hexdigest() != member.digest:
        reject(f"successor member {identifier} has a sha256 mismatch")


def rehash_member(root: Path, member, logical_id: str) -> bytes:
    path = member_path(member, logical_id)
    payload = read_file(
        root, path, f"successor member {member.identifier}",
        expected_bytes=member.byte_count, optional=True,
    )
    if payload is None:
        raise MemberMiss(f"successor member {member.identifier} is missing")
    verify(member, payload)
    return payload


def stage_member(root: Path, member, logical_id: str, payload: bytes | None = None) -> tuple[Path, bool]:
    path = member_path(member, logical_id)
    existing = read_file(
        root, path, f"successor member {member.identifier}",
        expected_bytes=member.byte_count, optional=True,
    )
    if existing is not None:
        verify(member, existing)
        return target(root, path), True
    if payload is None:
        reject(f"successor member {member.identifier} needs a payload")
    verify(member, payload)
    return atomic_file(root, path, payload, f"successor member {member.identifier}"), False
