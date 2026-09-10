"""Live-plan projection and self-hashed receipt binding for staged Docs inputs."""

from __future__ import annotations

import copy

from vulkan_docs_input_model import (
    DERIVED, INPUT_STAGE_CONTRACT, INPUT_STAGE_FIELDS, MAX_PROVIDER_BYTES, MAX_STAGED_BYTES, RAW, InputMember,
    decode, encode, member, reject,
)
from vulkan_docs_stage_model import INPUT_FIELDS, INPUT_MANIFEST_FIELDS, MAX_INPUTS, StagingPlan, require_live_plan
from vulkan_docs_stage_parse import canonical, digest, exact_object

RAW_COUNT, DERIVED_COUNT = 298, 1462
MANIFEST_DOMAIN = "webboxvm-graphics-vulkan-docs-observed-input-manifest-v1"


def members(plan: StagingPlan) -> tuple[InputMember, ...]:
    value = require_live_plan(plan).value
    inputs = value.get("inputs")
    if not isinstance(inputs, dict) or set(inputs) != INPUT_FIELDS:
        reject("Docs input staging plan has an invalid input schema")
    raw, derived = inputs.get("raw_records"), inputs.get("derived_records")
    if not isinstance(raw, list) or not isinstance(derived, list) or (len(raw), len(derived)) != (RAW_COUNT, DERIVED_COUNT):
        reject("Docs input staging plan has unexpected input counts")
    result = tuple(member(row, RAW) for row in raw) + tuple(member(row, DERIVED) for row in derived)
    keys = tuple((item.kind, item.selector) for item in result)
    if len(result) > MAX_INPUTS or len(set(keys)) != len(keys) or len({item.selector for item in result}) != len(result):
        reject("Docs input staging plan has duplicate selectors")
    if tuple(item.selector for item in result[:len(raw)]) != tuple(sorted(item.selector for item in result[:len(raw)])):
        reject("Docs input staging raw selectors are unordered")
    if tuple(item.selector for item in result[len(raw):]) != tuple(sorted(item.selector for item in result[len(raw):])):
        reject("Docs input staging derived selectors are unordered")
    manifest = canonical([item.normalized() for item in sorted(result, key=lambda item: (item.kind, item.selector))], MANIFEST_DOMAIN)
    if manifest != digest(inputs.get("input_manifest_sha256"), "Docs input staging manifest sha256"):
        reject("Docs input staging plan has a crossed input manifest")
    total_bytes = sum(item.bytes for item in result)
    if not 0 < total_bytes <= MAX_STAGED_BYTES or total_bytes * 2 > MAX_PROVIDER_BYTES:
        reject("Docs input staging plan exceeds its cumulative byte bound")
    return result


def _bound_members(plan: StagingPlan, planned: tuple[InputMember, ...]) -> tuple[InputMember, ...]:
    expected = members(plan)
    if planned != expected:
        reject("Docs input-stage receipt has unbound staged members")
    return expected


def input_manifest(plan: StagingPlan, planned: tuple[InputMember, ...]) -> dict[str, object]:
    planned = _bound_members(plan, planned)
    raw = sum(item.kind == RAW for item in planned)
    derived = len(planned) - raw
    value = require_live_plan(plan).value["inputs"]
    result = {
        "raw_count": raw, "derived_count": derived, "total_count": len(planned),
        "manifest_sha256": digest(value["input_manifest_sha256"], "Docs input staging manifest sha256"),
    }
    if set(result) != INPUT_MANIFEST_FIELDS or (raw, derived) != (RAW_COUNT, DERIVED_COUNT):
        reject("Docs input staging manifest has invalid counts")
    return result


def _context(plan: StagingPlan, planned: tuple[InputMember, ...]) -> dict[str, object]:
    value = require_live_plan(plan).value
    captures = value.get("captures")
    if not isinstance(captures, list) or len(captures) != 2:
        reject("Docs input staging plan has invalid capture references")
    return {
        "schema": 1, "contract": INPUT_STAGE_CONTRACT, "status": "staging-only-unadmitted",
        "admitted": False, "cutover_ready": False, "profile": value["profile"], "role": value["role"],
        "required_input_id": value["required_input_id"], "plan_sha256": plan.digest,
        "build_witness_sha256": value["witness"]["witness_sha256"],
        "scope_identity_sha256": value["scope_receipt"]["scope_identity_sha256"],
        "comparison_sha256": value["comparison"]["comparison_sha256"], "captures": copy.deepcopy(captures),
        "inputs": input_manifest(plan, planned),
    }


def receipt_value(plan: StagingPlan, planned: tuple[InputMember, ...]) -> dict[str, object]:
    result = _context(plan, planned)
    if set(result) != INPUT_STAGE_FIELDS - {"input_stage_sha256"}:
        reject("Docs input-stage receipt has an invalid schema")
    result["input_stage_sha256"] = canonical(result, INPUT_STAGE_CONTRACT)
    return result


def parse(value: object, plan: StagingPlan, planned: tuple[InputMember, ...]) -> dict[str, object]:
    data = exact_object(value, INPUT_STAGE_FIELDS, "Docs input-stage receipt")
    if encode(data) != encode(receipt_value(plan, planned)):
        reject("Docs input-stage receipt does not exactly bind the live plan")
    return copy.deepcopy(data)


def receipt_from_bytes(payload: bytes, plan: StagingPlan, planned: tuple[InputMember, ...]) -> dict[str, object]:
    return parse(decode(payload), plan, planned)
