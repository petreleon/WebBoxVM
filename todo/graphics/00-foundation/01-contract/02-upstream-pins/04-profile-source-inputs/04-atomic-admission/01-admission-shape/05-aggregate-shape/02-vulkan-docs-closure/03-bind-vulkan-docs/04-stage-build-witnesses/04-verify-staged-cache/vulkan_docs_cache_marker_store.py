"""Single-session publication and reuse checks for the Docs staging marker."""

from __future__ import annotations

import hashlib
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
INPUT, OUTPUT, STAGE = HERE.parent / "02-stage-core-inputs", HERE.parent / "03-stage-output-witnesses", HERE.parent / "01-stage-contract"
for directory in (INPUT, OUTPUT, STAGE):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))

from vulkan_docs_input_bind import members, receipt_from_bytes as input_receipt
from vulkan_docs_input_cache import Cache, cache_session
from vulkan_docs_input_inventory import exact_inputs
from vulkan_docs_input_paths import receipt_relative as input_receipt_relative
from vulkan_docs_input_store import _rehash
from vulkan_docs_cache_marker_inventory import exact_closure
from vulkan_docs_output_bind import receipt_from_bytes as output_receipt
from vulkan_docs_output_inventory import exact_outputs, staged_trees
from vulkan_docs_output_model import decode, encode
from vulkan_docs_output_paths import receipt_relative as output_receipt_relative
from vulkan_docs_stage_marker import exact, marker_relative, parse, value
from vulkan_docs_stage_model import MAX_PLAN_BYTES, StagingPlan, reject, require_live_plan


def _verified(plan: StagingPlan, cache: Cache) -> tuple[dict[str, object], dict[str, object], tuple[object, ...], tuple[object, ...]]:
    planned = members(plan)
    payload = cache.read(input_receipt_relative(plan), "input-stage receipt", maximum_bytes=MAX_PLAN_BYTES)
    if payload is None:
        reject("Docs input-stage receipt is missing")
    inputs = input_receipt(payload, plan, planned)
    _rehash(plan, cache, planned)
    exact_inputs(plan, cache, planned, receipt=True)
    payload = cache.read(output_receipt_relative(plan), "output-stage receipt", maximum_bytes=MAX_PLAN_BYTES)
    if payload is None:
        reject("Docs output-stage receipt is missing")
    trees = staged_trees(plan, cache)
    exact_outputs(plan, cache, trees, receipt=True)
    outputs = output_receipt(payload, plan, inputs, trees)
    return inputs, outputs, planned, trees


def _read_marker(plan: StagingPlan, cache: Cache) -> dict[str, object]:
    payload = cache.read(marker_relative(plan), "staging marker", maximum_bytes=MAX_PLAN_BYTES)
    if payload is None:
        reject("Docs staging marker is missing")
    return decode(payload)


def _check_marker(plan: StagingPlan, cache: Cache, inputs: dict[str, object], outputs: dict[str, object]) -> dict[str, object]:
    marker = _read_marker(plan, cache)
    exact(marker, plan, inputs["inputs"], outputs["output_witnesses"])
    return parse(marker, plan)


def _publish(plan: StagingPlan) -> dict[str, object]:
    plan = require_live_plan(plan)
    with cache_session(plan) as cache:
        inputs, outputs, planned, trees = _verified(plan, cache)
        exact_closure(plan, cache, planned, trees, marker=False)
        payload = encode(value(plan, inputs["inputs"], outputs["output_witnesses"]))
        if len(payload) > MAX_PLAN_BYTES:
            reject("Docs staging marker exceeds its byte limit")
        target = marker_relative(plan)
        cache.atomic(target, payload, "staging marker", maximum_bytes=MAX_PLAN_BYTES)
        inputs, outputs, planned, trees = _verified(plan, cache)
        exact_closure(plan, cache, planned, trees, marker=True)
        verified = cache.read(target, "staging marker", expected_bytes=len(payload),
                              expected_sha256=hashlib.sha256(payload).hexdigest())
        if verified is None or verified != payload:
            reject("Docs staging marker changed after publication")
        result = _check_marker(plan, cache, inputs, outputs)
        exact_closure(plan, cache, planned, trees, marker=True)
        cache.confirm()
        return result


def _reuse(plan: StagingPlan) -> dict[str, object]:
    plan = require_live_plan(plan)
    with cache_session(plan) as cache:
        inputs, outputs, planned, trees = _verified(plan, cache)
        exact_closure(plan, cache, planned, trees, marker=True)
        result = _check_marker(plan, cache, inputs, outputs)
        exact_closure(plan, cache, planned, trees, marker=True)
        cache.confirm()
        return result
