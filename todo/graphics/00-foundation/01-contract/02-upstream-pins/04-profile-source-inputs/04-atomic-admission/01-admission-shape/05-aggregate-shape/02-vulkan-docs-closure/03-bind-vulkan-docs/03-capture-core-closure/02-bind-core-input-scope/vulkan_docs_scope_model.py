"""Fixed vocabulary and bounded result types for the core input/scope binder."""

from __future__ import annotations

from dataclasses import dataclass, field

RAW = "raw-observed-input"
DERIVED = "derived-observed-input"
PHASES = frozenset(("producer", "make-control", "generator", "asciidoctor", "postprocess", "asset-copy"))
MAX_RECORDS = 8192
MAX_INCLUDES = 65536
MAX_IGNORED_READS = 200000
RECORD_FIELDS = frozenset(("kind", "selector", "sha256", "bytes", "phase_roles"))
INCLUDE_FIELDS = frozenset(("kind", "selector", "line"))
NORMALIZED_FIELDS = frozenset((
    "records", "includes", "raw_count", "derived_count", "ignored_runtime_reads", "phase_counts",
    "input_manifest_sha256", "include_identity_sha256",
))
CAPTURE_FIELDS = frozenset(("run_id", "artifact", "run_sha256", "normalized_sha256"))
PRODUCER_FIELDS = frozenset((
    "source_tree_sha256", "producer_argv_sha256", "generation_id", "phase_counts", "identity_sha256",
))
INPUTS_FIELDS = frozenset((
    "raw_count", "derived_count", "raw_identity_sha256", "derived_identity_sha256", "input_manifest_sha256",
))
INCLUDES_FIELDS = frozenset(("count", "identity_sha256"))
CONDITIONS_FIELDS = frozenset((
    "wsi", "video", "extension_controls", "configuration_inputs", "individual_extension_branches", "images",
    "promotions", "identity_sha256",
))
MANIFEST_FIELDS = frozenset((
    "schema", "contract", "status", "profile", "role", "required_input_id", "capture", "build_witness_sha256",
    "configuration_sha256", "producer", "raw_records", "derived_records", "includes", "conditions",
    "scope_identity_sha256", "manifest_sha256",
))
RECEIPT_FIELDS = frozenset((
    "schema", "contract", "status", "profile", "role", "required_input_id", "observation_sha256", "capture",
    "build_witness_sha256", "configuration_sha256", "producer", "inputs", "includes", "conditions",
    "scope_identity_sha256", "manifest_sha256", "scope_receipt_sha256",
))
ROOT_SELECTOR = "vkspec.adoc"
WSI = "chapters/VK_KHR_surface/wsi.adoc"
VIDEO = "chapters/videocoding.adoc"
INDIVIDUAL_PREFIXES = ("appendices/VK_", "chapters/VK_")
INACTIVE_EXTENSION_SELECTORS = (
    "chapters/descriptorheaps.adoc", "chapters/descriptorbuffers.adoc", "chapters/fragmentdensitymapops.adoc",
    "chapters/VK_ARM_tensors/tensorops.adoc", "chapters/gpa_interface.adoc", "chapters/VK_NV_mesh_shader/mesh.adoc",
    "chapters/VK_HUAWEI_cluster_culling_shader/clusterculling.adoc",
    "chapters/device_generated_commands/generatedcommands.adoc",
    "chapters/VK_KHR_deferred_host_operations/deferred_host_operations.adoc", "chapters/accelstructures.adoc",
    "chapters/VK_AMDX_dense_geometry_format/dense_geometry_format.adoc",
    "chapters/VK_KHR_opacity_micromap/micromaps.adoc", "chapters/raytraversal.adoc", "chapters/raytracing.adoc",
    "chapters/memory_decompression.adoc", "chapters/VK_NV_optical_flow/optical_flow.adoc", "chapters/executiongraphs.adoc",
    "chapters/VK_NV_external_compute_queue/VK_NV_external_compute_queue.adoc", "chapters/VK_ARM_data_graph/graphs.adoc",
)
IMAGE_PREFIX = "images/"
IMAGE_COUNT = 42
CONFIG_INPUTS = ("generated/specattribs.adoc",)
EXTENSION_CONTROLS = (
    (RAW, "chapters/extensions.adoc"), (RAW, "appendices/extensions.adoc"),
    (DERIVED, "generated/meta/current_extensions_appendix.adoc"),
    (DERIVED, "generated/meta/deprecated_extensions_guard_macro.adoc"),
    (DERIVED, "generated/meta/current_extension_appendices_toc.adoc"),
    (DERIVED, "generated/meta/current_extension_appendices.adoc"),
    (DERIVED, "generated/meta/provisional_extensions_appendix.adoc"),
    (DERIVED, "generated/meta/provisional_extensions_guard_macro.adoc"),
    (DERIVED, "generated/meta/deprecated_extensions_appendix.adoc"),
)
PROMOTIONS = tuple((DERIVED, f"generated/meta/promoted_extensions_VK_VERSION_1_{minor}.adoc") for minor in range(1, 5))


class ScopeError(ValueError):
    """A capture, input identity, or core-scope claim is unsafe or incomplete."""


def reject(message: str) -> None:
    raise ScopeError(message)


@dataclass(frozen=True)
class CaptureExpectation:
    observation_digest: str
    identifier: str
    artifact: str
    run_digest: str
    normalized_digest: str
    source_tree_digest: str
    generated_tree_digest: str
    primary_html_digest: str
    producer_digest: str
    raw_count: int
    derived_count: int
    include_count: int
    phase_counts: dict[str, int]
    input_manifest_digest: str
    include_digest: str


@dataclass(frozen=True)
class ScopeManifest:
    value: dict[str, object]
    capture: CaptureExpectation
    scope_digest: str
    manifest_digest: str
    state: str = field(default="input-scope-only-unadmitted", init=False)
    admitted: bool = field(default=False, init=False)
    cutover_ready: bool = field(default=False, init=False)
