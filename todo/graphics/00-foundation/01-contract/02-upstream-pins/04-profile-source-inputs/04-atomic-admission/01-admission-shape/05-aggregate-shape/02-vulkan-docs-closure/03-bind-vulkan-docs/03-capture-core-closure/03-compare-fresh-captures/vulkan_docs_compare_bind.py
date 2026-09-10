"""Bind two distinct observed captures to one unadmitted semantic comparison."""

from __future__ import annotations

import sys
from pathlib import Path

from vulkan_docs_compare_model import (
    CAPTURE_FIELDS, COMPARISON_FIELDS, COUNT_FIELDS, IDENTITY_FIELDS, OUTPUT_FIELDS, RUN_IDS, STATUS,
    CaptureComparison, ComparisonError, reject,
)
from vulkan_docs_compare_locations import independent_locations
from vulkan_docs_compare_parse import canonical, digest, identifier, selector

HERE = Path(__file__).resolve().parent
SCOPE = HERE.parent / "02-bind-core-input-scope"
IDENTITY = HERE.parent.parent / "02-actual-closure-identity"
for directory in (SCOPE, IDENTITY):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))

from vulkan_docs_scope_bind import bind_capture  # noqa: E402
from vulkan_docs_identity_build import OUTPUT, TREE  # noqa: E402


def capture_ref(scope) -> dict[str, object]:
    capture = scope.capture
    value = {
        "run_id": capture.identifier, "artifact": capture.artifact, "run_sha256": capture.run_digest,
        "normalized_sha256": capture.normalized_digest, "io_trace_sha256": capture.io_trace_digest,
        "include_trace_sha256": capture.include_trace_digest,
    }
    if set(value) != CAPTURE_FIELDS:
        reject("capture reference has an invalid schema")
    identifier(value["run_id"], "capture run id")
    selector(value["artifact"], "capture artifact")
    for name in CAPTURE_FIELDS - {"run_id", "artifact"}:
        digest(value[name], f"capture {name}")
    return value


def identities(scope) -> dict[str, str]:
    try:
        value = scope.value
        result = {
            "normalized_sha256": scope.capture.normalized_digest,
            "input_manifest_sha256": scope.capture.input_manifest_digest,
            "raw_identity_sha256": canonical(value["raw_records"], "webboxvm-graphics-vulkan-docs-core-raw-identities-v1"),
            "derived_identity_sha256": canonical(value["derived_records"], "webboxvm-graphics-vulkan-docs-core-derived-identities-v1"),
            "include_identity_sha256": scope.capture.include_digest,
            "producer_identity_sha256": value["producer"]["identity_sha256"],
            "configuration_sha256": value["configuration_sha256"],
            "conditions_identity_sha256": value["conditions"]["identity_sha256"],
            "scope_identity_sha256": scope.scope_digest, "build_witness_sha256": value["build_witness_sha256"],
        }
    except (AttributeError, KeyError, TypeError) as error:
        reject(f"bound scope has an invalid semantic identity: {error}")
    if set(result) != IDENTITY_FIELDS:
        reject("bound scope has an incomplete semantic identity")
    return {name: digest(item, f"bound scope {name}") for name, item in result.items()}


def counts(scope) -> dict[str, int]:
    try:
        conditions = scope.value["conditions"]
        result = {
            "capture_count": 2, "raw_count": len(scope.value["raw_records"]),
            "derived_count": len(scope.value["derived_records"]), "include_count": len(scope.value["includes"]),
            "condition_count": len(conditions) - 1, "promotion_count": len(conditions["promotions"]["selectors"]),
            "extension_control_count": len(conditions["extension_controls"]["selectors"]), "image_count": conditions["images"]["count"],
        }
    except (AttributeError, KeyError, TypeError) as error:
        reject(f"bound scope has invalid count evidence: {error}")
    if set(result) != COUNT_FIELDS or any(type(item) is not int or item < 1 for item in result.values()):
        reject("bound scope has zero or malformed comparison evidence")
    if (result["condition_count"], result["promotion_count"], result["extension_control_count"], result["image_count"]) != (7, 4, 9, 42):
        reject("bound scope does not retain the reviewed core condition counts")
    return result


def equal(values, label: str):
    values = tuple(values)
    if len(set(values)) != 1:
        reject(f"independent captures differ in {label}")
    return values[0]


def output_witness(scopes) -> dict[str, object]:
    try:
        for scope in scopes:
            if (scope.capture.generated_tree_digest, scope.capture.primary_html_digest) != (TREE[2], OUTPUT["sha256"]):
                reject("capture does not retain the known output witness relation")
    except AttributeError as error:
        reject(f"bound scope lacks output witness metadata: {error}")
    return {"file_count": TREE[0], "bytes": TREE[1], "tree_sha256": TREE[2], "primary_html_sha256": OUTPUT["sha256"],
            "relation": "two-distinct-observations-known-witness-only"}


def _compare(scopes) -> CaptureComparison:
    scopes = tuple(scopes)
    if len(scopes) != 2:
        reject("comparison needs exactly two captures")
    pairs = sorted(((capture_ref(scope), scope) for scope in scopes), key=lambda item: item[0]["run_id"])
    captures, scopes = tuple(item[0] for item in pairs), tuple(item[1] for item in pairs)
    if tuple(item["run_id"] for item in captures) != RUN_IDS:
        reject("comparison does not name the two reviewed capture ids")
    for name in ("artifact", "run_sha256", "io_trace_sha256"):
        if len({item[name] for item in captures}) != 2:
            reject(f"comparison reuses a capture {name}")
    if captures[0]["normalized_sha256"] != captures[1]["normalized_sha256"]:
        reject("independent captures differ in normalized_sha256")
    if captures[0]["include_trace_sha256"] != captures[1]["include_trace_sha256"]:
        reject("independent captures differ in include_trace_sha256")
    identity_rows, count_rows = tuple(identities(scope) for scope in scopes), tuple(counts(scope) for scope in scopes)
    shared = {name: equal((row[name] for row in identity_rows), name) for name in IDENTITY_FIELDS}
    shared_counts = {name: equal((row[name] for row in count_rows), name) for name in COUNT_FIELDS}
    observation = equal((scope.capture.observation_digest for scope in scopes), "observation identity")
    payload = {
        "schema": 1, "contract": "vulkan-docs-core-capture-comparison-v1", "status": STATUS,
        "profile": "vulkan-1.4-core", "role": "api-limit-format-spec", "required_input_id": "vulkan-14-spec",
        "observation_sha256": digest(observation, "comparison observation sha256"), "captures": list(captures),
        "identities": dict(sorted(shared.items())), "counts": dict(sorted(shared_counts.items())),
        "output_witness": output_witness(scopes),
    }
    if set(payload["output_witness"]) != OUTPUT_FIELDS or set(payload) != COMPARISON_FIELDS - {"comparison_sha256"}:
        reject("comparison cannot be sealed from an invalid schema")
    payload["comparison_sha256"] = canonical(payload, "webboxvm-graphics-vulkan-docs-core-capture-comparison-v1")
    return CaptureComparison(payload, payload["comparison_sha256"])


def _compare_fixture(scopes) -> CaptureComparison:
    """Test-only semantic comparator; production callers must use bind_pair()."""
    return _compare(scopes)


def bind_pair(observation: Path, artifact_root: Path) -> CaptureComparison:
    try:
        scopes = tuple(bind_capture(observation, artifact_root, identifier) for identifier in RUN_IDS)
        independent_locations(artifact_root, scopes)
        return _compare(scopes)
    except ComparisonError:
        raise
    except Exception as error:
        reject(str(error))
