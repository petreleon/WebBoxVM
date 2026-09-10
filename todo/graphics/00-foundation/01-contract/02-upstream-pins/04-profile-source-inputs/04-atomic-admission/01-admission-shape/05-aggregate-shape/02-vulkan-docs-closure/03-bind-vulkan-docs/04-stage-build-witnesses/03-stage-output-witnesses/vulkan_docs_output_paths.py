"""Canonical provider and cache selectors for staged Docs output witnesses."""

from __future__ import annotations

from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
INPUT = HERE.parent / "02-stage-core-inputs"
STAGE = HERE.parent / "01-stage-contract"
for directory in (INPUT, STAGE):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))

from vulkan_docs_input_paths import provider_root
from vulkan_docs_stage_model import RUN_IDS, StagingPlan, reject, require_live_plan
from vulkan_docs_stage_paths import relative


def _capture(plan: StagingPlan, run_id: str) -> dict[str, object]:
    plan = require_live_plan(plan)
    if run_id not in RUN_IDS:
        reject("Docs output witness has an unknown recorded run")
    captures = plan.value["captures"]
    capture = next((item for item in captures if item["run_id"] == run_id), None)
    if not isinstance(capture, dict):
        reject("Docs output witness is missing a capture")
    return capture


def provider_relative(plan: StagingPlan, run_id: str) -> str:
    capture = _capture(plan, run_id)
    artifact = capture.get("artifact")
    if not isinstance(artifact, str) or not artifact.startswith("runs/"):
        reject("Docs output provider capture is invalid")
    result = f"{artifact}/generated"
    relative(result)
    return result


def source_relative(plan: StagingPlan, run_id: str) -> str:
    _capture(plan, run_id)
    result = f"sources/{run_id}"
    relative(result)
    return result


def output_base(plan: StagingPlan, run_id: str) -> str:
    plan, capture = require_live_plan(plan), _capture(plan, run_id)
    outputs = plan.value["layout"].get("outputs")
    value = outputs.get(capture["run_id"]) if isinstance(outputs, dict) else None
    if not isinstance(value, str):
        reject("Docs output cache layout is invalid")
    result = f"{plan.value['layout']['namespace']}/{plan.digest}/{value}"
    relative(result)
    return result


def staged_relative(plan: StagingPlan, run_id: str, selector: str) -> str:
    result = f"{output_base(plan, run_id)}/{selector}"
    relative(result)
    return result


def receipt_relative(plan: StagingPlan) -> str:
    plan = require_live_plan(plan)
    result = f"{plan.value['layout']['namespace']}/{plan.digest}/outputs/{plan.digest}.staged-outputs.json"
    relative(result)
    return result


def worktree_relative(plan: StagingPlan) -> str:
    plan = require_live_plan(plan)
    result = f"{plan.value['layout']['namespace']}/{plan.digest}"
    relative(result)
    return result
