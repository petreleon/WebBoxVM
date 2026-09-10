"""Paired-provider staging and complete rehashing of the unadmitted Docs inputs."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path

from vulkan_docs_input_bind import members, receipt_from_bytes, receipt_value
from vulkan_docs_input_cache import Cache, cache_session
from vulkan_docs_input_inventory import exact_inputs
from vulkan_docs_input_model import InputMember, encode, reject
from vulkan_docs_input_paths import receipt_relative, staged_relative, worktree_relative
from vulkan_docs_input_provider import Providers, providers
from vulkan_docs_stage_marker import marker_relative
from vulkan_docs_stage_model import MAX_PLAN_BYTES, StagingPlan, require_live_plan
from vulkan_docs_stage_paths import DIR_FLAGS, _private, close


def _absent(cache: Cache, value: str, label: str) -> None:
    if cache.read(value, label, maximum_bytes=MAX_PLAN_BYTES, optional=True) is not None:
        reject(f"Docs input staging refuses existing {label}")


def _empty(cache: Cache, value: str) -> None:
    try:
        parent, leaf = cache._parent(value, False)
    except FileNotFoundError:
        return
    descriptor = -1
    try:
        try:
            descriptor = os.open(leaf, DIR_FLAGS, dir_fd=parent)
        except FileNotFoundError:
            return
        _private(descriptor, "input-cache worktree")
        if os.listdir(descriptor):
            reject("Docs input staging refuses an existing worktree")
    except OSError as error:
        reject(f"Docs input-cache worktree is unsafe: {error}")
    finally:
        close(descriptor)
        close(parent)


def _rehash(plan: StagingPlan, cache: Cache, planned: tuple[InputMember, ...]) -> None:
    for item in planned:
        payload = cache.read(
            staged_relative(plan, item), f"Docs staged {item.kind} {item.selector}",
            expected_bytes=item.bytes, expected_sha256=item.sha256,
        )
        if payload is None:
            reject("Docs staged input is missing")


def _preflight(plan: StagingPlan, cache: Cache, planned: tuple[InputMember, ...]) -> None:
    _absent(cache, receipt_relative(plan), "input-stage receipt")
    _absent(cache, marker_relative(plan), "completion-looking marker")
    _empty(cache, worktree_relative(plan))
    for item in planned:
        target = staged_relative(plan, item)
        payload = cache.read(
            target, f"Docs existing {item.kind} {item.selector}",
            expected_bytes=item.bytes, expected_sha256=item.sha256, optional=True,
        )
        if payload is not None:
            reject(f"Docs input staging refuses existing staged {item.kind} {item.selector}")


def _stage_member(plan: StagingPlan, cache: Cache, sources: Providers, captures: list[object], item: InputMember) -> None:
    left, right = (sources.read(capture, item) for capture in captures)
    if left != right:
        reject(f"Docs input providers diverge for {item.selector}")
    target = staged_relative(plan, item)
    cache.atomic(target, left, f"Docs staged {item.kind} {item.selector}", maximum_bytes=item.bytes)
    payload = cache.read(
        target, f"Docs staged {item.kind} {item.selector}", expected_bytes=item.bytes, expected_sha256=item.sha256,
    )
    if payload != left:
        reject(f"Docs staged {item.kind} {item.selector} changed after publication")


def _stage(plan: StagingPlan, artifact_root: Path) -> dict[str, object]:
    plan = require_live_plan(plan)
    planned = members(plan)
    captures = plan.value["captures"]
    if not isinstance(captures, list) or len(captures) != 2:
        reject("Docs input staging plan has invalid capture references")
    with cache_session(plan, create=True) as cache:
        _preflight(plan, cache, planned)
        with providers(plan, artifact_root) as sources:
            for item in planned:
                _stage_member(plan, cache, sources, captures, item)
        _rehash(plan, cache, planned)
        exact_inputs(plan, cache, planned, receipt=False)
        receipt = receipt_value(plan, planned)
        payload = encode(receipt)
        if len(payload) > MAX_PLAN_BYTES:
            reject("Docs input-stage receipt exceeds its byte limit")
        receipt_path = receipt_relative(plan)
        cache.atomic(receipt_path, payload, "input-stage receipt", maximum_bytes=MAX_PLAN_BYTES)
        _rehash(plan, cache, planned)
        exact_inputs(plan, cache, planned, receipt=True)
        verified = cache.read(receipt_path, "input-stage receipt", expected_bytes=len(payload), expected_sha256=hashlib.sha256(payload).hexdigest())
        if verified is None:
            reject("Docs input-stage receipt is missing")
        result = receipt_from_bytes(verified, plan, planned)
        cache.confirm()
        return result


def _verify(plan: StagingPlan) -> dict[str, object]:
    plan = require_live_plan(plan)
    planned = members(plan)
    with cache_session(plan) as cache:
        payload = cache.read(receipt_relative(plan), "input-stage receipt", maximum_bytes=MAX_PLAN_BYTES)
        if payload is None:
            reject("Docs input-stage receipt is missing")
        receipt = receipt_from_bytes(payload, plan, planned)
        _rehash(plan, cache, planned)
        exact_inputs(plan, cache, planned, receipt=True)
        cache.confirm()
        return receipt
