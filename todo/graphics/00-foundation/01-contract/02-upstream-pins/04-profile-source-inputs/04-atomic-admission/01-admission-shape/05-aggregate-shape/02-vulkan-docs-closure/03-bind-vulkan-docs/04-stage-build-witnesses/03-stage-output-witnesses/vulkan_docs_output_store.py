"""Separate, rehashed staging of the two recorded Docs output trees."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
INPUT = HERE.parent / "02-stage-core-inputs"
STAGE = HERE.parent / "01-stage-contract"
for directory in (INPUT, STAGE):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))

from vulkan_docs_input_cache import Cache, cache_session
from vulkan_docs_output_bind import receipt_from_bytes, receipt_value
from vulkan_docs_output_inventory import exact_outputs, staged_trees
from vulkan_docs_output_model import OutputTree
from vulkan_docs_output_paths import output_base, receipt_relative, staged_relative, worktree_relative
from vulkan_docs_output_provider import providers
from vulkan_docs_output_tree import manifests, validate
from vulkan_docs_stage_marker import marker_relative
from vulkan_docs_stage_model import MAX_PLAN_BYTES, RUN_IDS, StagingPlan, reject, require_live_plan
from vulkan_docs_stage_paths import DIR_FLAGS, _private, close


def _absent(cache: Cache, value: str, label: str) -> None:
    if cache.read(value, label, maximum_bytes=MAX_PLAN_BYTES, optional=True) is not None:
        reject(f"Docs output staging refuses existing {label}")


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
        _private(descriptor, "output-cache worktree")
        if os.listdir(descriptor):
            reject("Docs output staging refuses an existing output worktree")
    except OSError as error:
        reject(f"Docs output-cache worktree is unsafe: {error}")
    finally:
        close(descriptor)
        close(parent)


def _preflight(plan: StagingPlan, cache: Cache) -> None:
    _absent(cache, receipt_relative(plan), "output-stage receipt")
    _absent(cache, marker_relative(plan), "completion-looking marker")
    _empty(cache, f"{worktree_relative(plan)}/outputs")
    for run_id in RUN_IDS:
        _empty(cache, output_base(plan, run_id))


def _publish_tree(plan: StagingPlan, cache: Cache, source, capture: dict[str, object], value: OutputTree) -> None:
    for item in value.members:
        payload = source.read(capture, item)
        target = staged_relative(plan, value.run_id, item.selector)
        cache.atomic(target, payload, f"Docs staged {value.run_id} output {item.selector}", maximum_bytes=item.bytes)
        stored = cache.read(target, f"Docs staged {value.run_id} output {item.selector}", expected_bytes=item.bytes,
                            expected_sha256=item.sha256)
        if stored != payload:
            reject("Docs staged output changed after publication")


def _stage(plan: StagingPlan, artifact_root: Path, input_receipt: dict[str, object]) -> dict[str, object]:
    plan = require_live_plan(plan)
    with cache_session(plan, create=True) as cache:
        _preflight(plan, cache)
        with providers(plan, artifact_root) as source:
            original = tuple(validate(plan, source.scan(capture)) for capture in plan.value["captures"])
            if tuple(item.run_id for item in original) != RUN_IDS:
                reject("Docs output providers did not preserve recorded-run ordering")
            for capture, value in zip(plan.value["captures"], original):
                _publish_tree(plan, cache, source, capture, value)
                if source.scan(capture) != value:
                    reject("Docs output provider tree changed during staging")
            source.confirm()
        staged = staged_trees(plan, cache)
        if staged != original:
            reject("Docs staged output tree does not rehash to its provider witness")
        exact_outputs(plan, cache, original, receipt=False)
        receipt, payload = receipt_value(plan, input_receipt, original), None
        payload = receipt_value_bytes(receipt)
        target = receipt_relative(plan)
        cache.atomic(target, payload, "output-stage receipt", maximum_bytes=MAX_PLAN_BYTES)
        staged = staged_trees(plan, cache)
        exact_outputs(plan, cache, staged, receipt=True)
        verified = cache.read(target, "output-stage receipt", expected_bytes=len(payload),
                              expected_sha256=hashlib.sha256(payload).hexdigest())
        if verified is None:
            reject("Docs output-stage receipt is missing")
        result = receipt_from_bytes(verified, plan, input_receipt, staged)
        cache.confirm()
        return result


def receipt_value_bytes(value: dict[str, object]) -> bytes:
    from vulkan_docs_output_model import encode
    payload = encode(value)
    if len(payload) > MAX_PLAN_BYTES:
        reject("Docs output-stage receipt exceeds its byte limit")
    return payload


def _verify(plan: StagingPlan, input_receipt: dict[str, object]) -> dict[str, object]:
    plan = require_live_plan(plan)
    with cache_session(plan) as cache:
        payload = cache.read(receipt_relative(plan), "output-stage receipt", maximum_bytes=MAX_PLAN_BYTES)
        if payload is None:
            reject("Docs output-stage receipt is missing")
        staged = staged_trees(plan, cache)
        manifests(plan, staged)
        exact_outputs(plan, cache, staged, receipt=True)
        result = receipt_from_bytes(payload, plan, input_receipt, staged)
        cache.confirm()
        return result
