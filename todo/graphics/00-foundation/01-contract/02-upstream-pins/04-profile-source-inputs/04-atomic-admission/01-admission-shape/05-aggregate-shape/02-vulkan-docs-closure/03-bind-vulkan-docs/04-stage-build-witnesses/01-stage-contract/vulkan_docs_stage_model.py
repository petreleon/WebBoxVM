"""Fixed vocabulary and unadmitted result types for actual Docs staging."""

from __future__ import annotations

from dataclasses import dataclass, field
import hashlib
import json
from pathlib import Path

STATUS = "staging-only-unadmitted"
PLAN_CONTRACT = "vulkan-docs-build-staging-plan-v1"
MARKER_CONTRACT = "vulkan-docs-build-staging-marker-v1"
NAMESPACE = "webboxvm-graphics/vulkan-docs/staging/v1"
RUN_IDS = ("observer-a", "observer-b")
MAX_PLAN_BYTES = 1024 * 1024
MAX_INPUT_BYTES = 8 * 1024 * 1024
MAX_INPUTS = 8192
MAX_OUTPUT_FILES = 4096
MAX_OUTPUT_BYTES = 32 * 1024 * 1024

PLAN_FIELDS = frozenset((
    "schema", "contract", "status", "admitted", "cutover_ready", "profile", "role", "required_input_id",
    "witness", "scope_receipt", "comparison", "captures", "inputs", "layout", "plan_sha256",
))
CAPTURE_FIELDS = frozenset((
    "run_id", "artifact", "run_sha256", "normalized_sha256", "io_trace_sha256", "include_trace_sha256",
    "source_tree_sha256", "generated_tree_sha256", "primary_html_sha256", "producer_argv_sha256",
))
INPUT_FIELDS = frozenset(("raw_records", "derived_records", "input_manifest_sha256"))
LAYOUT_FIELDS = frozenset(("namespace", "raw", "derived", "outputs", "markers"))
MARKER_FIELDS = frozenset((
    "schema", "contract", "status", "admitted", "cutover_ready", "profile", "role", "required_input_id",
    "plan_sha256", "build_witness_sha256", "scope_identity_sha256", "comparison_sha256", "inputs",
    "output_witnesses", "marker_sha256",
))
INPUT_MANIFEST_FIELDS = frozenset(("raw_count", "derived_count", "total_count", "manifest_sha256"))
OUTPUT_MANIFEST_FIELDS = frozenset((
    "run_id", "manifest_sha256", "file_count", "bytes", "tree_sha256", "primary_html_sha256",
))


class StageError(ValueError):
    """A Docs staging plan, marker, or filesystem boundary is unsafe or stale."""


def reject(message: str) -> None:
    raise StageError(message)


def bounded_payload(value: object, maximum: object) -> bytes:
    if not isinstance(value, bytes) or isinstance(maximum, bool) or not isinstance(maximum, int):
        reject("Docs staging payload has an invalid byte limit")
    if not 0 <= maximum <= MAX_OUTPUT_BYTES:
        reject("Docs staging payload has an invalid byte limit")
    if len(value) > maximum:
        reject("Docs staging payload exceeds its byte limit")
    return value


@dataclass(frozen=True)
class StagingPlan:
    value: dict[str, object]
    digest: str
    external_root: Path
    _provenance: object | None = field(default=None, repr=False, compare=False)
    state: str = field(default=STATUS, init=False)
    admitted: bool = field(default=False, init=False)
    cutover_ready: bool = field(default=False, init=False)


_LIVE_PROVENANCE = object()


def _live_plan(plan: StagingPlan) -> StagingPlan:
    return StagingPlan(plan.value, plan.digest, plan.external_root, _LIVE_PROVENANCE)


def require_live_plan(plan: object) -> StagingPlan:
    if not isinstance(plan, StagingPlan) or plan._provenance is not _LIVE_PROVENANCE:
        reject("Docs staging plan lacks live pinned provenance")
    try:
        payload = dict(plan.value)
        sealed = payload.pop("plan_sha256")
        rendered = json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False)
        actual = hashlib.sha256(
            b"webboxvm-graphics-vulkan-docs-build-staging-plan-v1\0" + rendered.encode("utf-8")
        ).hexdigest()
    except (AttributeError, KeyError, TypeError, ValueError) as error:
        reject(f"Docs staging plan cannot retain live provenance: {error}")
    if not isinstance(sealed, str) or sealed != plan.digest or actual != plan.digest:
        reject("Docs staging plan changed after live prerequisite binding")
    return plan
