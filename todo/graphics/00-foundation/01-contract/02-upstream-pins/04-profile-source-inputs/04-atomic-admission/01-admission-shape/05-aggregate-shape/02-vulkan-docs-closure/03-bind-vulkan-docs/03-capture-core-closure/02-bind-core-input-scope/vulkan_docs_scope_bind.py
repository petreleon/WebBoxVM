"""Bind one selected observer artifact without comparing it to another capture."""

from __future__ import annotations

from dataclasses import replace
import sys
from pathlib import Path

from vulkan_docs_scope_model import (
    DERIVED, MANIFEST_FIELDS, MAX_INCLUDES, MAX_RECORDS, PHASES, RAW, CaptureExpectation, ScopeManifest, reject,
)
from vulkan_docs_scope_parse import canonical, digest, document, file_digest, identifier, positive, require_safe_child, selector
from vulkan_docs_scope_records import normalized
from vulkan_docs_scope_semantics import conditions, derived_rows, raw_rows

HERE = Path(__file__).resolve().parent
OBSERVER = HERE.parent / "01-observe-pinned-build-inputs"
IDENTITY = HERE.parent.parent / "02-actual-closure-identity"
for directory in (OBSERVER, IDENTITY):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))

from vulkan_docs_observer_contract import observation_value, run as observer_run  # noqa: E402
from vulkan_docs_observer_events import producer_digest as exact_producer_digest  # noqa: E402
from vulkan_docs_observer_plan import SOURCE_TREE  # noqa: E402
from vulkan_docs_identity_build import GENERATION_ID, OUTPUT, TREE  # noqa: E402
from vulkan_docs_identity_contract import witness  # noqa: E402


def inherited(function, *args):
    try:
        return function(*args)
    except Exception as error:
        reject(str(error))


def expected(observation_path: Path, run_id: str) -> CaptureExpectation:
    value = document(observation_path)
    observed = inherited(observation_value, value)
    identity = identifier(run_id, "selected observer run")
    rows = [row for row in value["runs"] if row.get("id") == identity]
    if len(rows) != 1:
        reject("selected observer run is absent or ambiguous")
    row = rows[0]
    return CaptureExpectation(
        observed.digest, identity, selector(row.get("artifact"), "selected observer artifact"),
        digest(row.get("run_sha256"), "selected observer run sha256"), "", row["source_tree_sha256"],
        row["generated_tree_sha256"], row["primary_html_sha256"], row["producer_argv_sha256"], row["raw_count"],
        row["derived_count"], row["include_count"], dict(row["phase_counts"]), row["input_manifest_sha256"],
        row["include_identity_sha256"],
    )


def artifact_file(root: Path, artifact: str, name: str) -> Path:
    try:
        if root.is_symlink():
            reject("observer artifact root is a symlink")
        root = root.resolve(strict=True)
    except OSError as error:
        reject(f"observer artifact root cannot be resolved: {error}")
    if not root.is_dir() or not artifact.startswith("runs/"):
        reject("observer artifact root or run location is unsafe")
    return require_safe_child(root, f"{artifact}/observer/{name}", "observer artifact")


def validate_expected(value: CaptureExpectation):
    identifier(value.identifier, "selected observer run")
    selector(value.artifact, "selected observer artifact")
    for field in ("observation_digest", "run_digest", "normalized_digest", "source_tree_digest", "generated_tree_digest",
                  "primary_html_digest", "producer_digest", "input_manifest_digest", "include_digest"):
        digest(getattr(value, field), f"selected observer {field}")
    if (value.source_tree_digest, value.generated_tree_digest, value.primary_html_digest, value.producer_digest) != (
            SOURCE_TREE[2], TREE[2], OUTPUT["sha256"], exact_producer_digest()):
        reject("selected observer run has stale producer or output-witness metadata")
    counts = tuple(positive(item, "selected observer count", MAX_INCLUDES) for item in (
        value.raw_count, value.derived_count, value.include_count))
    if counts[0] + counts[1] > MAX_RECORDS:
        reject("selected observer run exceeds the bounded input count")
    if not isinstance(value.phase_counts, dict) or set(value.phase_counts) != PHASES:
        reject("selected observer run has an invalid phase-count schema")
    if any(positive(item, "selected observer phase count", MAX_RECORDS) > sum(counts[:2]) for item in value.phase_counts.values()):
        reject("selected observer run has an invalid phase count")
    return inherited(witness)


def bind_value(value: object, capture: CaptureExpectation) -> ScopeManifest:
    build_witness = validate_expected(capture)
    records, includes = normalized(value, capture)
    raw, derived = raw_rows(records), derived_rows(records)
    if len(raw) < 2 or not derived:
        reject("core scope cannot be root-only or lack derived inputs")
    producer = {
        "source_tree_sha256": capture.source_tree_digest, "producer_argv_sha256": capture.producer_digest,
        "generation_id": GENERATION_ID, "phase_counts": dict(sorted(capture.phase_counts.items())),
    }
    producer["identity_sha256"] = canonical(producer, "webboxvm-graphics-vulkan-docs-core-producer-v1")
    scope_conditions = conditions(records, includes)
    semantic = {
        "profile": "vulkan-1.4-core", "role": "api-limit-format-spec", "required_input_id": "vulkan-14-spec",
        "build_witness_sha256": build_witness.digest, "configuration_sha256": build_witness.recipe.configuration_digest,
        "producer": producer, "raw_records": raw, "derived_records": derived, "includes": list(includes),
        "conditions": scope_conditions,
    }
    scope_digest = canonical(semantic, "webboxvm-graphics-vulkan-docs-core-scope-v1")
    capture_value = {"run_id": capture.identifier, "artifact": capture.artifact, "run_sha256": capture.run_digest,
                     "normalized_sha256": capture.normalized_digest}
    manifest = {
        "schema": 1, "contract": "vulkan-docs-core-input-scope-manifest-v1", "status": "input-scope-only-unadmitted",
        **{key: semantic[key] for key in ("profile", "role", "required_input_id", "build_witness_sha256", "configuration_sha256",
                                          "producer", "raw_records", "derived_records", "includes", "conditions")},
        "capture": capture_value, "scope_identity_sha256": scope_digest,
    }
    if set(manifest) != MANIFEST_FIELDS - {"manifest_sha256"}:
        reject("core scope manifest cannot be sealed from an invalid schema")
    manifest["manifest_sha256"] = canonical(manifest, "webboxvm-graphics-vulkan-docs-core-scope-manifest-v1")
    return ScopeManifest(manifest, capture, scope_digest, manifest["manifest_sha256"])


def bind_capture(observation_path: Path, artifact_root: Path, run_id: str) -> ScopeManifest:
    capture = expected(observation_path, run_id)
    run_path = artifact_file(artifact_root, capture.artifact, "run.json")
    local = inherited(observer_run, document(run_path))
    if (local.identifier, local.digest) != (capture.identifier, capture.run_digest):
        reject("observer artifact run does not match the selected header run")
    normalized_path = artifact_file(artifact_root, capture.artifact, "normalized-inputs.json")
    return bind_value(document(normalized_path), replace(capture, normalized_digest=file_digest(normalized_path, "normalized observer")))
