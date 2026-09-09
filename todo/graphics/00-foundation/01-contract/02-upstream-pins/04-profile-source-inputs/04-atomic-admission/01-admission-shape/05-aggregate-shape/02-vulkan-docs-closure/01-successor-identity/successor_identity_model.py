"""Immutable types and field sets for an isolated successor closure fixture."""

from __future__ import annotations

from dataclasses import dataclass, field

RAW = "raw-member"
GENERATED = "generated-member"
RAW_FIELDS = frozenset(
    (
        "kind", "id", "source_family", "immutable_url", "revision", "sha256", "bytes",
        "license", "local_cache", "generated_code_role", "provenance", "selector",
    )
)
GENERATED_FIELDS = frozenset(
    (
        "kind", "id", "producer_member_ids", "generation_id", "selector", "sha256",
        "bytes", "license", "local_cache", "generated_code_role", "provenance",
    )
)
RECIPE_FIELDS = frozenset(("argv", "workdir", "sha256"))
CONFIG_FIELDS = frozenset(("attributes", "sha256"))
TOOLCHAIN_FIELDS = frozenset(("name", "version", "sha256"))
GENERATION_FIELDS = frozenset(
    (
        "id", "recipe", "configuration", "toolchain", "output_member_ids",
        "output_tree_sha256", "clean_run_tree_sha256s",
    )
)
SCOPE_FIELDS = frozenset(
    (
        "scope_kind", "evidence_mode", "ordered_member_ids", "generated_member_ids",
        "excluded_members", "configuration_sha256", "scope_sha256",
    )
)
ADAPTER_FIELDS = frozenset(
    (
        "kind", "predecessor_rules_sha256", "predecessor_audit_sha256",
        "predecessor_boundary_sha256", "candidate_root_id", "candidate_root_sha256",
        "predecessor_decision",
    )
)
FIXTURE_FIELDS = frozenset(
    (
        "schema", "contract", "status", "profile", "role", "required_input_id",
        "root_member_id", "members", "generations", "scope", "predecessor_adapter",
        "closure_sha256",
    )
)


class IdentityError(ValueError):
    """A successor-only fixture is incomplete or could resemble active admission."""


def reject(message: str) -> None:
    raise IdentityError(message)


@dataclass(frozen=True)
class RawMember:
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
class GeneratedMember:
    identifier: str
    producers: tuple[str, ...]
    generation_id: str
    digest: str
    byte_count: int
    license: str
    selector: str
    cache_path: str
    generated_code_role: str
    provenance: str


@dataclass(frozen=True)
class Generation:
    identifier: str
    recipe_digest: str
    configuration_digest: str
    toolchain_name: str
    toolchain_version: str
    toolchain_digest: str
    output_ids: tuple[str, ...]
    output_tree_digest: str
    clean_run_tree_digests: tuple[str, str]


@dataclass(frozen=True)
class Scope:
    member_ids: tuple[str, ...]
    generated_ids: tuple[str, ...]
    configuration_digest: str
    scope_digest: str


@dataclass(frozen=True)
class PredecessorAdapter:
    candidate_root_id: str
    candidate_root_digest: str
    rules_digest: str
    audit_digest: str
    boundary_digest: str
    decision: str


@dataclass(frozen=True)
class ClosureCachePlan:
    logical_id: str
    closure_digest: str
    members: tuple[RawMember | GeneratedMember, ...]
    generations: tuple[Generation, ...]
    scope: Scope
    predecessor: PredecessorAdapter

    @property
    def member_ids(self) -> tuple[str, ...]:
        return tuple(item.identifier for item in self.members)

    @property
    def generated_ids(self) -> tuple[str, ...]:
        return tuple(item.identifier for item in self.members if isinstance(item, GeneratedMember))


@dataclass(frozen=True)
class FixtureClosure:
    cache_plan: ClosureCachePlan
    state: str = field(default="fixture-validated", init=False)
    admitted: bool = field(default=False, init=False)
    cutover_ready: bool = field(default=False, init=False)

    @property
    def member_ids(self) -> tuple[str, ...]:
        return self.cache_plan.member_ids

    @property
    def generated_ids(self) -> tuple[str, ...]:
        return self.cache_plan.generated_ids
