"""Manifest construction and exact plan binding for Docs output witness trees."""

from __future__ import annotations

from vulkan_docs_output_model import OutputMember, OutputTree, bound, tree_digest
from vulkan_docs_stage_model import MAX_OUTPUT_BYTES, MAX_OUTPUT_FILES, RUN_IDS, StagingPlan, reject, require_live_plan
from vulkan_docs_stage_parse import digest


def tree(run_id: str, values: object) -> OutputTree:
    if run_id not in RUN_IDS or not isinstance(values, tuple) or not values:
        reject("Docs output tree has an invalid recorded run or member list")
    if not all(isinstance(item, OutputMember) for item in values):
        reject("Docs output tree has an invalid member")
    selectors = tuple(item.selector for item in values)
    if selectors != tuple(sorted(selectors)) or len(set(selectors)) != len(selectors):
        reject("Docs output tree selectors are unordered or duplicated")
    total = sum(item.bytes for item in values)
    bound(len(values), "Docs output tree file count", MAX_OUTPUT_FILES)
    bound(total, "Docs output tree bytes", MAX_OUTPUT_BYTES)
    primary = next((item for item in values if item.selector == "out/html/vkspec.html"), None)
    if primary is None:
        reject("Docs output tree lacks the primary HTML")
    return OutputTree(run_id, values, total, tree_digest(values), primary)


def _capture(plan: StagingPlan, run_id: str) -> dict[str, object]:
    plan = require_live_plan(plan)
    captures = plan.value.get("captures")
    if not isinstance(captures, list) or tuple(item.get("run_id") for item in captures) != RUN_IDS:
        reject("Docs output tree plan has invalid capture ordering")
    capture = next(item for item in captures if item["run_id"] == run_id)
    if not isinstance(capture, dict):
        reject("Docs output tree capture is invalid")
    return capture


def validate(plan: StagingPlan, value: OutputTree) -> OutputTree:
    plan = require_live_plan(plan)
    capture = _capture(plan, value.run_id)
    witness = plan.value.get("comparison", {}).get("output_witness")
    outputs = plan.value.get("witness", {}).get("outputs")
    if not isinstance(witness, dict) or not isinstance(outputs, list) or len(outputs) != 1:
        reject("Docs output tree plan lacks a reviewed witness")
    primary = outputs[0]
    expected = (witness.get("file_count"), witness.get("bytes"), witness.get("tree_sha256"), witness.get("primary_html_sha256"))
    actual = (len(value.members), value.bytes, value.sha256, value.primary.sha256)
    if actual != expected or value.sha256 != capture.get("generated_tree_sha256"):
        reject("Docs output tree does not match its recorded witness")
    if (value.primary.selector, value.primary.bytes, value.primary.sha256) != (
            primary.get("selector"), primary.get("bytes"), primary.get("sha256")):
        reject("Docs output tree primary HTML does not match the build witness")
    if digest(value.sha256, "Docs output tree sha256") != value.sha256:
        reject("Docs output tree digest is invalid")
    return value


def manifests(plan: StagingPlan, values: object) -> list[dict[str, object]]:
    if not isinstance(values, tuple) or tuple(item.run_id for item in values if isinstance(item, OutputTree)) != RUN_IDS:
        reject("Docs output witnesses are not the two ordered recorded trees")
    if len(values) != len(RUN_IDS) or not all(isinstance(item, OutputTree) for item in values):
        reject("Docs output witnesses are invalid")
    return [validate(plan, item).manifest() for item in values]
