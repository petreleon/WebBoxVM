"""Canonical provider and cache selectors for one Docs input staging pass."""

from __future__ import annotations

from pathlib import Path

from vulkan_docs_input_model import DERIVED, RAW, InputMember, reject
from vulkan_docs_stage_model import StagingPlan, require_live_plan
from vulkan_docs_stage_paths import relative


def provider_root(value: Path) -> Path:
    if not isinstance(value, Path) or not value.is_absolute() or value.anchor != "/":
        reject("Docs input provider root must be an explicit absolute path")
    if any(part in (".", "..") for part in value.parts) or value == Path(value.anchor):
        reject("Docs input provider root has an unsafe component")
    root = Path(value.anchor)
    for part in value.parts[1:]:
        root /= part
        try:
            if root.is_symlink():
                reject("Docs input provider root has a symlink component")
        except OSError as error:
            reject(f"Docs input provider root cannot be inspected safely: {error}")
    if not root.is_dir():
        reject("Docs input provider root is not a directory")
    return root


def provider_relative(capture: object, member: InputMember) -> str:
    if not isinstance(capture, dict):
        reject("Docs input provider capture is invalid")
    run_id, artifact = capture.get("run_id"), capture.get("artifact")
    if not isinstance(run_id, str) or not isinstance(artifact, str) or not artifact.startswith("runs/"):
        reject("Docs input provider capture is invalid")
    relative(artifact)
    value = f"sources/{run_id}/{member.selector}" if member.kind == RAW else f"{artifact}/{member.selector}"
    if member.kind not in (RAW, DERIVED):
        reject("Docs input provider has an unknown kind")
    relative(value)
    return value


def staged_relative(plan: StagingPlan, member: InputMember) -> str:
    value = require_live_plan(plan).value
    layout = value["layout"]
    location = layout["raw"] if member.kind == RAW else layout["derived"]
    result = f"{layout['namespace']}/{plan.digest}/{location}/{member.staged_selector}"
    relative(result)
    return result


def worktree_relative(plan: StagingPlan) -> str:
    value = require_live_plan(plan).value
    result = f"{value['layout']['namespace']}/{plan.digest}"
    relative(result)
    return result


def receipt_relative(plan: StagingPlan) -> str:
    value = require_live_plan(plan).value
    result = f"{value['layout']['namespace']}/{plan.digest}/inputs/{plan.digest}.staged-inputs.json"
    relative(result)
    return result
