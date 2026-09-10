"""Fixed schema and unadmitted result type for the independent Docs capture comparison."""

from __future__ import annotations

from dataclasses import dataclass, field

STATUS = "input-comparison-only-unadmitted"
RUN_IDS = ("observer-a", "observer-b")
CAPTURE_FIELDS = frozenset((
    "run_id", "artifact", "run_sha256", "normalized_sha256", "io_trace_sha256", "include_trace_sha256",
))
IDENTITY_FIELDS = frozenset((
    "normalized_sha256", "input_manifest_sha256", "raw_identity_sha256", "derived_identity_sha256",
    "include_identity_sha256", "producer_identity_sha256", "configuration_sha256", "conditions_identity_sha256",
    "scope_identity_sha256", "build_witness_sha256",
))
COUNT_FIELDS = frozenset((
    "capture_count", "raw_count", "derived_count", "include_count", "condition_count", "promotion_count",
    "extension_control_count", "image_count",
))
OUTPUT_FIELDS = frozenset(("file_count", "bytes", "tree_sha256", "primary_html_sha256", "relation"))
COMPARISON_FIELDS = frozenset((
    "schema", "contract", "status", "profile", "role", "required_input_id", "observation_sha256", "captures",
    "identities", "counts", "output_witness", "comparison_sha256",
))


class ComparisonError(ValueError):
    """The two capture artifacts are not a bounded, independent semantic match."""


def reject(message: str) -> None:
    raise ComparisonError(message)


@dataclass(frozen=True)
class CaptureComparison:
    value: dict[str, object]
    digest: str
    state: str = field(default=STATUS, init=False)
    admitted: bool = field(default=False, init=False)
    cutover_ready: bool = field(default=False, init=False)
