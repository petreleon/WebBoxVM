"""Stage and verify a fixture-only successor closure cache."""

from __future__ import annotations

import hashlib
import tempfile
from pathlib import Path
from typing import Callable

from successor_cache_identity import IDENTITY, PROJECT_ROOT, plan as _plan, trusted_repository
from successor_cache_lock import cache_lock
from successor_cache_fs import remove_file
from successor_cache_marker import canonical, exact, lock_relative, marker_relative, publish
from successor_cache_model import CacheError, CacheMiss, CacheReceipt, MemberMiss, reject
from successor_cache_store import cache_root, rehash_member, stage_member

__all__ = ("stage_fixture", "verify_fixture", "CacheError", "CacheMiss", "CacheReceipt")


def _receipt(value, reused: bool) -> CacheReceipt:
    return CacheReceipt(value.closure_digest, value.member_ids, reused)


def _verify(root: Path, value) -> CacheReceipt:
    exact(root, value)
    for member in value.members:
        rehash_member(root, member, value.logical_id)
    return _receipt(value, True)


def _output_members(value, generation) -> tuple:
    members = {item.identifier: item for item in value.members if not hasattr(item, "immutable_url")}
    try:
        return tuple(members[identifier] for identifier in generation.output_ids)
    except KeyError:
        reject("successor generation has an unknown output member")


def _run(callback, generation, producers, outputs) -> tuple[tuple[str, bytes], ...]:
    try:
        with tempfile.TemporaryDirectory(prefix="successor-generation-") as scratch:
            result = callback(generation, producers, Path(scratch))
    except Exception as error:
        reject(f"successor generation runner failed: {error}")
    if not isinstance(result, tuple) or len(result) != len(outputs):
        reject("successor generation runner returned an invalid output tuple")
    rows = []
    for pair, member in zip(result, outputs):
        if not isinstance(pair, tuple) or len(pair) != 2 or pair[0] != member.identifier:
            reject("successor generation runner reordered or renamed an output")
        if not isinstance(pair[1], bytes):
            reject("successor generation runner returned a non-byte output")
        if len(pair[1]) != member.byte_count or hashlib.sha256(pair[1]).hexdigest() != member.digest:
            reject("successor generation runner output does not match its identity")
        rows.append(pair)
    tree = [
        {"id": item.identifier, "selector": item.selector, "sha256": item.digest,
         "bytes": item.byte_count}
        for item in outputs
    ]
    if canonical(tree) != generation.output_tree_digest:
        reject("successor generation output tree does not match its identity")
    return tuple(rows)


def _stage_raw(root: Path, value, member, reader) -> bytes:
    try:
        return rehash_member(root, member, value.logical_id)
    except MemberMiss:
        try:
            payload = reader(member)
        except Exception as error:
            reject(f"successor raw reader failed: {error}")
        stage_member(root, member, value.logical_id, payload)
        return rehash_member(root, member, value.logical_id)


def _stage_ordered(root: Path, value, reader, runner) -> None:
    by_generation = {item.identifier: item for item in value.generations}
    payloads, complete = {}, set()
    for member in value.members:
        if hasattr(member, "immutable_url"):
            payloads[member.identifier] = _stage_raw(root, value, member, reader)
            continue
        if member.generation_id in complete:
            continue
        generation = by_generation.get(member.generation_id)
        if generation is None:
            reject("successor member has an unknown generation")
        outputs = _output_members(value, generation)
        if not outputs or member.identifier not in {item.identifier for item in outputs}:
            reject("successor member is absent from its generation outputs")
        output_ids = {item.identifier for item in outputs}
        producer_ids = tuple(dict.fromkeys(
            identifier for output in outputs for identifier in output.producers
            if identifier not in output_ids
        ))
        if any(identifier not in payloads for identifier in producer_ids):
            continue
        producers = tuple((identifier, payloads[identifier]) for identifier in producer_ids)
        first = _run(runner, generation, producers, outputs)
        second = _run(runner, generation, producers, outputs)
        if first != second:
            reject("successor generation clean runs diverged")
        for output, (_, payload) in zip(outputs, first):
            stage_member(root, output, value.logical_id, payload)
            payloads[output.identifier] = rehash_member(root, output, value.logical_id)
        complete.add(generation.identifier)
    if complete != set(by_generation):
        reject("successor generation dependencies are not closure ordered")


def stage_fixture(
    fixture_path: Path,
    external_root: Path,
    repository: Path,
    read_raw: Callable[[object], bytes],
    run_generation_callback: Callable[
        [object, tuple[tuple[str, bytes], ...], Path], tuple[tuple[str, bytes], ...]
    ],
    predecessor_paths: object = IDENTITY.PREDECESSOR_PATHS,
) -> CacheReceipt:
    value = _plan(fixture_path, predecessor_paths)
    root = cache_root(external_root, trusted_repository(repository))
    if not callable(read_raw) or not callable(run_generation_callback):
        reject("successor cache provider is not callable")
    with cache_lock(root, lock_relative(value), exclusive=True, create=True) as guard:
        try:
            return _verify(root, value)
        except CacheMiss:
            pass
        _stage_ordered(root, value, read_raw, run_generation_callback)
        for member in value.members:
            rehash_member(root, member, value.logical_id)
        guard()
        publish(root, value)
        try:
            _verify(root, value)
            guard()
        except CacheError:
            remove_file(root, marker_relative(value))
            raise
        return _receipt(value, False)


def verify_fixture(
    fixture_path: Path,
    external_root: Path,
    repository: Path,
    predecessor_paths: object = IDENTITY.PREDECESSOR_PATHS,
) -> CacheReceipt:
    value = _plan(fixture_path, predecessor_paths)
    root = cache_root(external_root, trusted_repository(repository))
    try:
        with cache_lock(root, lock_relative(value), exclusive=False, create=False):
            return _verify(root, value)
    except CacheMiss:
        return _verify(root, value)
