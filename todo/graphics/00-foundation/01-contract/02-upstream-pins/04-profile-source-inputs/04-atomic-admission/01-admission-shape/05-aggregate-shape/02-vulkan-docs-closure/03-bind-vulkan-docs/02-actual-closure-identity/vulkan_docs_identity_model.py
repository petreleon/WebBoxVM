"""Types and fixed schemas for the unadmitted actual-Docs successor grammar."""

from __future__ import annotations

from dataclasses import dataclass, field

RAW = "raw-source-input"
DERIVED = "derived-source-input"
RENDERED = "rendered-output"
MAX_RENDERED_BYTES = 16 * 1024 * 1024
MAX_RENDERED_TREE_BYTES = 32 * 1024 * 1024
MAX_RENDERED_FILES = 4096

RAW_FIELDS = frozenset((
    "kind", "id", "source_family", "immutable_url", "revision", "sha256", "bytes", "license",
    "local_cache", "generated_code_role", "provenance", "selector",
))
DERIVED_FIELDS = frozenset((
    "kind", "id", "producer_input_ids", "generation_id", "selector", "sha256", "bytes", "license",
    "local_cache", "generated_code_role", "provenance",
))
RENDERED_FIELDS = frozenset((
    "kind", "id", "producer_generation_id", "selector", "sha256", "bytes", "license", "local_cache",
    "artifact_kind", "provenance",
))
BUILD_FIELDS = frozenset((
    "image", "platform", "network", "user", "source_mount", "work_mount", "generated_mount",
    "workdir", "home", "path", "safe_directory", "environment", "argv", "toolchain", "build_sha256",
))
TOOLCHAIN_FIELDS = frozenset(("python", "python_version", "pyparsing_version"))
ENV_FIELDS = frozenset(("name", "value"))
TREE_FIELDS = frozenset((
    "algorithm", "file_count", "bytes", "sha256", "primary_output_id", "clean_run_tree_sha256s",
))
WITNESS_FIELDS = frozenset((
    "schema", "contract", "status", "profile", "role", "required_input_id", "root", "build", "outputs",
    "output_tree", "predecessor_adapter", "witness_sha256",
))
SCOPE_FIELDS = frozenset((
    "scope_kind", "evidence_mode", "ordered_input_ids", "derived_input_ids", "excluded_members",
    "predecessor_include_list_sha256", "configuration_sha256", "member_identity_sha256", "scope_sha256",
))
CLOSURE_FIELDS = frozenset((
    "schema", "contract", "status", "profile", "role", "required_input_id", "root_input_id", "inputs",
    "scope", "build_witness_sha256", "closure_sha256",
))


class DocsIdentityError(ValueError):
    """A Docs witness or closure is unsafe, stale, incomplete, or too broad."""


def reject(message: str) -> None:
    raise DocsIdentityError(message)


@dataclass(frozen=True)
class RawInput:
    identifier: str
    source_family: str
    immutable_url: str
    revision: str
    digest: str
    byte_count: int
    license: str
    selector: str
    cache_path: str
    generated_code_role: str
    provenance: str


@dataclass(frozen=True)
class DerivedInput:
    identifier: str
    digest: str
    byte_count: int
    selector: str
    cache_path: str
    producers: tuple[str, ...]
    generation_id: str
    license: str
    generated_code_role: str
    provenance: str


@dataclass(frozen=True)
class RenderedOutput:
    identifier: str
    digest: str
    byte_count: int
    selector: str
    cache_path: str
    producer: str
    license: str
    artifact_kind: str
    provenance: str


@dataclass(frozen=True)
class BuildRecipe:
    digest: str
    configuration_digest: str
    generation_id: str


@dataclass(frozen=True)
class BuildWitness:
    digest: str
    root: RawInput
    recipe: BuildRecipe
    outputs: tuple[RenderedOutput, ...]
    tree_digest: str
    state: str = field(default="actual-docs-build-witness", init=False)
    admitted: bool = field(default=False, init=False)
    cutover_ready: bool = field(default=False, init=False)


@dataclass(frozen=True)
class ActualClosure:
    digest: str
    inputs: tuple[RawInput | DerivedInput, ...]
    output_witness_digest: str
    state: str = field(default="actual-docs-closure-unadmitted", init=False)
    admitted: bool = field(default=False, init=False)
    cutover_ready: bool = field(default=False, init=False)
