"""Immutable outcomes and errors for the isolated successor closure cache."""

from __future__ import annotations

from dataclasses import dataclass, field


class CacheError(ValueError):
    """A fixture-only successor cache is incomplete, unsafe, or stale."""


class CacheMiss(CacheError):
    """No complete marker exists; individual member files are not a cache hit."""


class MemberMiss(CacheError):
    """One declared member is absent, rather than merely unverified."""


def reject(message: str) -> None:
    raise CacheError(message)


@dataclass(frozen=True)
class CacheReceipt:
    closure_digest: str
    member_ids: tuple[str, ...]
    reused: bool
    state: str = field(default="fixture-cache-verified", init=False)
    admitted: bool = field(default=False, init=False)
    cutover_ready: bool = field(default=False, init=False)
