"""Immutable facts and result types for the pre-admission aggregate."""

from __future__ import annotations

from dataclasses import dataclass, field

DIRECT = (
    "opengl-46-core-spec",
    "opengl-cts-manifest",
    "gles-32-spec",
)
BLOCKERS = (
    ("gles-cts-manifest", "requires-multifile-core-selector-closure"),
    ("vulkan-14-spec", "vulkan-docs-core-generated-closure-unadmitted"),
    (
        "vulkan-cts-mustpass",
        "vcts-vk-default-compound-oversize-core-scope-unadmitted",
    ),
)
GLES_COUNTS = (4, 1, 12, 12477, 30574, 1)
VULKAN = (("vulkan-14-spec", 73), ("vulkan-cts-mustpass", 98))


@dataclass(frozen=True)
class GlesClosure:
    core_members: tuple[tuple[str, str, str], ...]
    excluded_members: tuple[tuple[str, str, str], ...]
    configuration_sha256: str
    configuration_counts: tuple[int, int, int, int]
    state: str = field(default="unadmitted", init=False)


@dataclass(frozen=True)
class VulkanObservation:
    required_input_id: str
    root_sha256: str
    observation_count: int
    requirements: tuple[str, ...]
    scope_exclusions: tuple[tuple[str, tuple[str, ...]], ...]
    state: str = field(default="unadmitted", init=False)


@dataclass(frozen=True)
class PreAdmissionAggregate:
    required_input_ids: tuple[str, ...]
    direct_candidate_ids: tuple[str, ...]
    blockers: tuple[tuple[str, str], ...]
    inventory_revision: str
    gles: GlesClosure
    vulkan: tuple[VulkanObservation, ...]
    state: str = field(default="pre-admission", init=False)
    admission_eligible: bool = field(default=False, init=False)
    cutover_ready: bool = field(default=False, init=False)
