"""Fixed, self-hashed future completion-marker grammar for actual Docs staging."""

from __future__ import annotations

import copy

from vulkan_docs_stage_model import (
    INPUT_MANIFEST_FIELDS, MARKER_CONTRACT, MARKER_FIELDS, MAX_INPUTS, MAX_OUTPUT_BYTES, MAX_OUTPUT_FILES,
    OUTPUT_MANIFEST_FIELDS, RUN_IDS, StagingPlan, reject, require_live_plan,
)
from vulkan_docs_stage_parse import bounded, canonical, digest, exact_object, fixed_state
from vulkan_docs_stage_paths import relative


def _context(plan: StagingPlan) -> dict[str, object]:
    value = require_live_plan(plan).value
    return {
        "schema": 1, "contract": MARKER_CONTRACT, "status": "staging-only-unadmitted",
        "admitted": False, "cutover_ready": False, "profile": value["profile"], "role": value["role"],
        "required_input_id": value["required_input_id"], "plan_sha256": plan.digest,
        "build_witness_sha256": value["witness"]["witness_sha256"],
        "scope_identity_sha256": value["scope_receipt"]["scope_identity_sha256"],
        "comparison_sha256": value["comparison"]["comparison_sha256"],
    }


def _inputs(plan: StagingPlan, value: object) -> dict[str, object]:
    data = exact_object(value, INPUT_MANIFEST_FIELDS, "Docs staging input manifest")
    expected = plan.value["inputs"]
    raw, derived = len(expected["raw_records"]), len(expected["derived_records"])
    if (bounded(data.get("raw_count"), "Docs staging raw count", MAX_INPUTS),
            bounded(data.get("derived_count"), "Docs staging derived count", MAX_INPUTS),
            bounded(data.get("total_count"), "Docs staging total count", MAX_INPUTS)) != (raw, derived, raw + derived):
        reject("Docs staging input manifest does not bind the planned input counts")
    if digest(data.get("manifest_sha256"), "Docs staging input manifest sha256") != expected["input_manifest_sha256"]:
        reject("Docs staging input manifest does not bind the planned input identity")
    return copy.deepcopy(data)


def _outputs(plan: StagingPlan, value: object) -> list[dict[str, object]]:
    if not isinstance(value, list) or len(value) != len(RUN_IDS):
        reject("Docs staging output manifests need both recorded runs")
    witness = plan.value["comparison"]["output_witness"]
    result = []
    for item, capture, run_id in zip(value, plan.value["captures"], RUN_IDS):
        data = exact_object(item, OUTPUT_MANIFEST_FIELDS, "Docs staging output manifest")
        if data.get("run_id") != run_id:
            reject("Docs staging output manifests reorder or replace recorded runs")
        if (bounded(data.get("file_count"), "Docs staging output file count", MAX_OUTPUT_FILES),
                bounded(data.get("bytes"), "Docs staging output bytes", MAX_OUTPUT_BYTES),
                digest(data.get("tree_sha256"), "Docs staging output tree sha256"),
                digest(data.get("primary_html_sha256"), "Docs staging primary HTML sha256")) != (
                    witness["file_count"], witness["bytes"], witness["tree_sha256"], witness["primary_html_sha256"]):
            reject("Docs staging output manifest does not bind the full observed tree")
        if data["tree_sha256"] != capture["generated_tree_sha256"] or data["primary_html_sha256"] != capture["primary_html_sha256"]:
            reject("Docs staging output manifest does not bind its recorded run")
        if digest(data.get("manifest_sha256"), "Docs staging output manifest sha256") != data["tree_sha256"]:
            reject("Docs staging output manifest identity does not match its full tree")
        result.append(copy.deepcopy(data))
    return result


def value(plan: StagingPlan, inputs: object, outputs: object) -> dict[str, object]:
    result = {**_context(plan), "inputs": _inputs(plan, inputs), "output_witnesses": _outputs(plan, outputs)}
    if set(result) != MARKER_FIELDS - {"marker_sha256"}:
        reject("Docs staging marker cannot be sealed from an invalid schema")
    result["marker_sha256"] = canonical(result, "webboxvm-graphics-vulkan-docs-build-staging-marker-v1")
    return result


def parse(value: object, plan: StagingPlan) -> dict[str, object]:
    data = exact_object(value, MARKER_FIELDS, "Docs staging marker")
    fixed_state(data, "Docs staging marker")
    context = _context(plan)
    if any(data.get(name) != item for name, item in context.items()):
        reject("Docs staging marker does not bind the validated plan")
    actual = digest(data.get("marker_sha256"), "Docs staging marker sha256")
    payload = dict(data)
    payload.pop("marker_sha256", None)
    if actual != canonical(payload, "webboxvm-graphics-vulkan-docs-build-staging-marker-v1"):
        reject("Docs staging marker has a stale self identity")
    _inputs(plan, data.get("inputs"))
    _outputs(plan, data.get("output_witnesses"))
    return copy.deepcopy(data)


def exact(value_to_check: object, plan: StagingPlan, inputs: object, outputs: object) -> None:
    if parse(value_to_check, plan) != value(plan, inputs, outputs):
        reject("Docs staging marker does not exactly match the staged manifests")


def marker_relative(plan: StagingPlan) -> str:
    plan = require_live_plan(plan)
    value = f"{plan.value['layout']['namespace']}/{plan.digest}/{plan.value['layout']['markers']}/{plan.digest}.complete.json"
    relative(value)
    return value
