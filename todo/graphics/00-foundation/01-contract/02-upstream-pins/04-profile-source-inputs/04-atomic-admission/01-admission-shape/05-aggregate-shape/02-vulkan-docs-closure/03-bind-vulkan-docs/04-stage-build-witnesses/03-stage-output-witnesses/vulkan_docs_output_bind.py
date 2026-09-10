"""Exact receipt binding for the two unadmitted Docs output witnesses."""

from __future__ import annotations

import copy
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
INPUT = HERE.parent / "02-stage-core-inputs"
if str(INPUT) not in sys.path:
    sys.path.insert(0, str(INPUT))

from vulkan_docs_input_bind import members as input_members
from vulkan_docs_input_bind import parse as parse_input
from vulkan_docs_output_model import OUTPUT_STAGE_CONTRACT, OUTPUT_STAGE_FIELDS, decode, encode
from vulkan_docs_output_tree import manifests
from vulkan_docs_stage_model import StagingPlan, reject, require_live_plan
from vulkan_docs_stage_parse import canonical, exact_object


def bound_input(plan: StagingPlan, input_receipt: object) -> dict[str, object]:
    """Return only an exact receipt for the current live input-stage plan."""
    plan = require_live_plan(plan)
    return parse_input(input_receipt, plan, input_members(plan))


def _context(plan: StagingPlan, input_receipt: object, trees: object) -> dict[str, object]:
    plan = require_live_plan(plan)
    inputs = bound_input(plan, input_receipt)
    value = plan.value
    captures = value.get("captures")
    if not isinstance(captures, list) or len(captures) != 2:
        reject("Docs output-stage receipt has invalid capture references")
    return {
        "schema": 1,
        "contract": OUTPUT_STAGE_CONTRACT,
        "status": "staging-only-unadmitted",
        "admitted": False,
        "cutover_ready": False,
        "profile": value["profile"],
        "role": value["role"],
        "required_input_id": value["required_input_id"],
        "plan_sha256": plan.digest,
        "build_witness_sha256": value["witness"]["witness_sha256"],
        "scope_identity_sha256": value["scope_receipt"]["scope_identity_sha256"],
        "comparison_sha256": value["comparison"]["comparison_sha256"],
        "captures": copy.deepcopy(captures),
        "inputs": copy.deepcopy(inputs["inputs"]),
        "input_stage_sha256": inputs["input_stage_sha256"],
        "output_witnesses": copy.deepcopy(manifests(plan, trees)),
    }


def receipt_value(plan: StagingPlan, input_receipt: object, trees: object) -> dict[str, object]:
    """Project the sealed plan, exact input receipt, and ordered output manifests."""
    result = _context(plan, input_receipt, trees)
    if set(result) != OUTPUT_STAGE_FIELDS - {"output_stage_sha256"}:
        reject("Docs output-stage receipt has an invalid schema")
    result["output_stage_sha256"] = canonical(result, OUTPUT_STAGE_CONTRACT)
    return result


def parse(value: object, plan: StagingPlan, input_receipt: object, trees: object) -> dict[str, object]:
    """Reject any receipt that is not the complete recomputed live projection."""
    data = exact_object(value, OUTPUT_STAGE_FIELDS, "Docs output-stage receipt")
    if encode(data) != encode(receipt_value(plan, input_receipt, trees)):
        reject("Docs output-stage receipt does not exactly bind staged witnesses")
    return copy.deepcopy(data)


def receipt_from_bytes(payload: bytes, plan: StagingPlan, input_receipt: object, trees: object) -> dict[str, object]:
    """Decode with duplicate-key rejection, then enforce the exact receipt projection."""
    return parse(decode(payload), plan, input_receipt, trees)
