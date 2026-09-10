"""Live pinned-prerequisite binding and deterministic plan construction for Docs staging."""

from __future__ import annotations

import copy
import sys
from pathlib import Path

from vulkan_docs_stage_model import (
    CAPTURE_FIELDS, INPUT_FIELDS, LAYOUT_FIELDS, MAX_INPUTS, NAMESPACE, PLAN_CONTRACT, PLAN_FIELDS, RUN_IDS,
    StagingPlan, _live_plan, reject,
)
from vulkan_docs_stage_parse import canonical, digest, exact_object, fixed_state
from vulkan_docs_stage_paths import external_root

HERE = Path(__file__).resolve().parent
BIND = HERE.parent.parent
IDENTITY = BIND / "02-actual-closure-identity"
SCOPE = BIND / "03-capture-core-closure/02-bind-core-input-scope"
COMPARE = BIND / "03-capture-core-closure/03-compare-fresh-captures"
for directory in (IDENTITY, SCOPE, COMPARE):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))

from vulkan_docs_identity_contract import witness_value  # noqa: E402
from vulkan_docs_scope_bind import bind_capture  # noqa: E402
from vulkan_docs_scope_contract import compact as scope_compact  # noqa: E402
from vulkan_docs_scope_contract import receipt as scope_receipt  # noqa: E402
from vulkan_docs_compare_bind import capture_ref, counts, identities  # noqa: E402
from vulkan_docs_compare_contract import receipt as comparison_receipt  # noqa: E402

WITNESS = IDENTITY / "vulkan_docs_build_witness.json"
SCOPE_RECEIPT = SCOPE / "vulkan_docs_core_input_scope.json"
COMPARISON_RECEIPT = COMPARE / "vulkan_docs_core_input_comparison.json"


def project_root() -> Path:
    for candidate in HERE.parents:
        if (candidate / ".git").exists() and not candidate.is_symlink():
            return candidate.resolve(strict=True)
    raise RuntimeError("cannot locate the WebBoxVM repository")


PROJECT_ROOT = project_root()


def _capture(scope) -> dict[str, object]:
    value = {
        **capture_ref(scope), "source_tree_sha256": scope.capture.source_tree_digest,
        "generated_tree_sha256": scope.capture.generated_tree_digest,
        "primary_html_sha256": scope.capture.primary_html_digest,
        "producer_argv_sha256": scope.capture.producer_digest,
    }
    if set(value) != CAPTURE_FIELDS:
        reject("Docs staging capture has an invalid schema")
    for name in CAPTURE_FIELDS - {"run_id", "artifact"}:
        digest(value[name], f"Docs staging capture {name}")
    return value


def _bind(observation: Path, artifact_root: Path, raw_witness: dict[str, object]):
    try:
        build_witness = witness_value(raw_witness)
        scopes = tuple(bind_capture(observation, artifact_root, run_id) for run_id in RUN_IDS)
        checked_scope = scope_receipt(SCOPE_RECEIPT, observation, artifact_root, RUN_IDS[0])
        comparison = comparison_receipt(COMPARISON_RECEIPT, observation, artifact_root)
    except Exception as error:
        reject(f"Docs staging prerequisite binding failed: {error}")
    if checked_scope.value != scopes[0].value or checked_scope.scope_digest != scopes[0].scope_digest:
        reject("Docs staging scope receipt does not bind the selected live capture")
    if any(scope.value["build_witness_sha256"] != build_witness.digest for scope in scopes):
        reject("Docs staging scopes do not bind the reviewed build witness")
    return build_witness, scopes, comparison, scope_compact(checked_scope)


def _match(scopes, comparison) -> None:
    captures = [_capture(scope) for scope in scopes]
    comparison_captures = comparison.value.get("captures")
    if comparison_captures != [capture_ref(scope) for scope in scopes]:
        reject("Docs staging captures do not match the independent comparison")
    if tuple(item["run_id"] for item in captures) != RUN_IDS:
        reject("Docs staging captures are not the two reviewed runs")
    common = identities(scopes[0])
    if any(identities(scope) != common or counts(scope) != counts(scopes[0]) for scope in scopes[1:]):
        reject("Docs staging captures do not share one semantic scope")
    if comparison.value.get("identities") != common or comparison.value.get("counts") != counts(scopes[0]):
        reject("Docs staging comparison has stale semantic identities")
    if scopes[0].value["raw_records"] != scopes[1].value["raw_records"]:
        reject("Docs staging captures have divergent raw records")
    if scopes[0].value["derived_records"] != scopes[1].value["derived_records"]:
        reject("Docs staging captures have divergent derived records")


def _layout() -> dict[str, object]:
    return {
        "namespace": NAMESPACE, "raw": "inputs/raw", "derived": "inputs/derived",
        "outputs": {run_id: f"outputs/{run_id}" for run_id in RUN_IDS}, "markers": "markers",
    }


def _value(witness_value, scope_value, scopes, comparison) -> dict[str, object]:
    _match(scopes, comparison)
    records = {
        "raw_records": copy.deepcopy(scopes[0].value["raw_records"]),
        "derived_records": copy.deepcopy(scopes[0].value["derived_records"]),
        "input_manifest_sha256": scopes[0].capture.input_manifest_digest,
    }
    if set(records) != INPUT_FIELDS or len(records["raw_records"]) + len(records["derived_records"]) > MAX_INPUTS:
        reject("Docs staging inputs exceed their explicit bound")
    result = {
        "schema": 1, "contract": PLAN_CONTRACT, "status": "staging-only-unadmitted",
        "admitted": False, "cutover_ready": False, "profile": "vulkan-1.4-core", "role": "api-limit-format-spec",
        "required_input_id": "vulkan-14-spec", "witness": copy.deepcopy(witness_value),
        "scope_receipt": copy.deepcopy(scope_value), "comparison": copy.deepcopy(comparison.value),
        "captures": [_capture(scope) for scope in scopes], "inputs": records, "layout": _layout(),
    }
    if set(result) != PLAN_FIELDS - {"plan_sha256"}:
        reject("Docs staging plan cannot be sealed from an invalid schema")
    result["plan_sha256"] = canonical(result, "webboxvm-graphics-vulkan-docs-build-staging-plan-v1")
    return result


def plan_value(value: object, root: Path) -> StagingPlan:
    data = exact_object(value, PLAN_FIELDS, "Docs staging plan")
    if tuple(data.get(name) for name in ("schema", "contract", "profile", "role", "required_input_id")) != (
            1, PLAN_CONTRACT, "vulkan-1.4-core", "api-limit-format-spec", "vulkan-14-spec"):
        reject("Docs staging plan has an unrelated schema")
    fixed_state(data, "Docs staging plan")
    actual = digest(data.get("plan_sha256"), "Docs staging plan sha256")
    payload = dict(data)
    payload.pop("plan_sha256", None)
    if actual != canonical(payload, "webboxvm-graphics-vulkan-docs-build-staging-plan-v1"):
        reject("Docs staging plan has a stale self identity")
    if not isinstance(data.get("captures"), list) or len(data["captures"]) != len(RUN_IDS):
        reject("Docs staging plan has an invalid capture list")
    if any(not isinstance(item, dict) or set(item) != CAPTURE_FIELDS for item in data["captures"]):
        reject("Docs staging plan has an invalid capture schema")
    if tuple(item.get("run_id") for item in data["captures"]) != RUN_IDS:
        reject("Docs staging plan reorders or replaces captures")
    if not isinstance(data.get("inputs"), dict) or set(data["inputs"]) != INPUT_FIELDS:
        reject("Docs staging plan has an invalid input schema")
    if not isinstance(data.get("layout"), dict) or set(data["layout"]) != LAYOUT_FIELDS or data["layout"] != _layout():
        reject("Docs staging plan has an invalid cache layout")
    return StagingPlan(copy.deepcopy(data), actual, root)


def build_plan(observation: Path, artifact_root: Path, cache_root: Path) -> StagingPlan:
    root = external_root(cache_root, PROJECT_ROOT)
    witness_value = _read_witness()
    build_witness, scopes, comparison, scope_value = _bind(observation, artifact_root, witness_value)
    result = _live_plan(plan_value(_value(witness_value, scope_value, scopes, comparison), root))
    if result.value["witness"]["witness_sha256"] != build_witness.digest:
        reject("Docs staging plan does not retain the reviewed witness")
    return result


def _read_witness() -> dict[str, object]:
    from vulkan_docs_stage_parse import document
    return document(WITNESS)


def verify_value(value: object, expected: StagingPlan) -> StagingPlan:
    result = plan_value(value, expected.external_root)
    if result.value != expected.value:
        reject("Docs staging plan does not bind the selected pinned prerequisites")
    return expected
