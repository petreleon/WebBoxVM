#!/usr/bin/env python3
"""Fail closed around a future Vulkan-Docs-derived source-policy envelope."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

HERE = Path(__file__).resolve().parent
POLICY = HERE / "derived_docs_policy.json"
REQUIREMENTS = HERE.parents[7] / "03-feature-matrix/01-profile-scope/source_requirements.json"
MAX_POLICY_BYTES = 64 * 1024
MAX_SOURCE_MEMBER_BYTES = 8 * 1024 * 1024
DIGEST = re.compile(r"^[0-9a-f]{64}$")
REVISION = "f84d432d5b8912362f96f581f29bbc4f3c8c7843"
REQUIRED = ("vulkan-1.4-core", "api-limit-format-spec", "vulkan-14-spec")
TARGET_FIELDS = ("profile", "role", "required_input_id")
EFFECTS = frozenset(("inventory_changed", "cache_freshness_proved", "f03_changed", "supported",
                     "conformant", "certified", "near_native", "admitted", "admission_eligible",
                     "cutover_ready"))


class PolicyError(ValueError):
    """The policy cannot be used as a trusted successor boundary."""


def reject(message: str) -> None:
    raise PolicyError(message)


def object_no_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            reject(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


def document(path: Path) -> dict[str, object]:
    raw = path.read_bytes()
    if len(raw) > MAX_POLICY_BYTES:
        reject("policy document exceeds its bounded size")
    try:
        value = json.loads(raw, object_pairs_hook=object_no_duplicates)
    except (UnicodeDecodeError, json.JSONDecodeError, PolicyError) as error:
        reject(f"policy document is invalid: {error}")
    if not isinstance(value, dict):
        reject("policy document must be an object")
    return value


def text(value: object, field: str) -> str:
    if not isinstance(value, str) or not value:
        reject(f"{field} must be a nonempty string")
    return value


def exact_object(value: object, fields: frozenset[str], field: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != fields:
        reject(f"{field} has an invalid schema")
    return value


def digest(value: object, field: str) -> str:
    result = text(value, field)
    if not DIGEST.fullmatch(result) or result == "0" * 64:
        reject(f"{field} has an invalid sha256")
    return result


def canonical(value: dict[str, object]) -> bytes:
    return json.dumps({key: item for key, item in value.items() if key != "policy_sha256"},
                      sort_keys=True, separators=(",", ":")).encode()


def require_false_flags(value: object) -> None:
    flags = exact_object(value, EFFECTS, "effects")
    if any(item is not False for item in flags.values()):
        reject("policy attempts to claim an active, support, or release effect")


def require_requirements(path: Path, expected_digest: str) -> None:
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected_digest:
        reject("policy does not bind the current F03 source requirements")
    value = document(path)
    rows = value.get("requirements")
    if not isinstance(rows, list) or not any(
            tuple(row.get(name) for name in ("profile", "role", "required_input_id")) == REQUIRED
            for row in rows if isinstance(row, dict)):
        reject("policy does not preserve the Vulkan 1.4 source requirement")


@dataclass(frozen=True)
class DerivedDocsPolicy:
    sha256: str
    source_revision: str
    source_member_cap: int


def policy_value(value: object, requirements: Path = REQUIREMENTS) -> DerivedDocsPolicy:
    fields = frozenset(("schema", "contract", "status", "profile", "role", "required_input_id",
                        "source_requirements_sha256", "source", "build", "generated", "vcts",
                        "effects", "policy_sha256"))
    policy = exact_object(value, fields, "derived Docs policy")
    if tuple(policy.get(name) for name in ("schema", "contract", "status", *TARGET_FIELDS)) != (
            1, "vulkan-docs-derived-source-policy-v1", "policy-only-unadmitted", *REQUIRED):
        reject("policy has an unexpected contract or target")
    requirements_digest = digest(policy["source_requirements_sha256"], "source_requirements_sha256")
    require_requirements(requirements, requirements_digest)
    source = exact_object(policy["source"], frozenset(("repository", "revision", "root_selector",
                          "raw_member_max_bytes", "tree_identity_required", "immutable_https_required")), "source")
    if tuple(source.get(name) for name in ("repository", "revision", "root_selector")) != (
            "KhronosGroup/Vulkan-Docs", REVISION, "vkspec.adoc"):
        reject("policy does not retain the reviewed immutable Docs root")
    if source.get("raw_member_max_bytes") != MAX_SOURCE_MEMBER_BYTES:
        reject("policy changes the 8 MiB raw-source cap")
    if source.get("tree_identity_required") is not True or source.get("immutable_https_required") is not True:
        reject("policy permits a root-only or mutable source")
    build = exact_object(policy["build"], frozenset(("required_argv_tokens", "pinned_builder_required",
                         "network", "two_fresh_runs_required", "scope_manifest_required")), "build")
    if build.get("required_argv_tokens") != ["./makeSpec", "-spec", "core", "-version", "1.4", "html"]:
        reject("policy does not require the Vulkan 1.4 core build route")
    if tuple(build.get(name) for name in ("pinned_builder_required", "network", "two_fresh_runs_required",
                                          "scope_manifest_required")) != (True, "none", True, True):
        reject("policy permits an unpinned, networked, or one-run build")
    generated = exact_object(policy["generated"], frozenset(("rendered_output_may_satisfy_source",
                             "output_member_max_bytes", "authoritative_license_expression_required",
                             "attribution_manifest_required", "producer_lineage_required")), "generated")
    if tuple(generated.get(name) for name in ("rendered_output_may_satisfy_source", "output_member_max_bytes",
                                               "authoritative_license_expression_required",
                                               "attribution_manifest_required", "producer_lineage_required")) != (
            False, 32 * 1024 * 1024, True, True, True):
        reject("policy weakens generated-output, license, or lineage separation")
    vcts = exact_object(policy["vcts"], frozenset(("can_satisfy_docs", "local_core_selector_allowed",
                       "canonical_suite_scope")), "vcts")
    if tuple(vcts.get(name) for name in ("can_satisfy_docs", "local_core_selector_allowed", "canonical_suite_scope")) != (
            False, False, "broader-than-vulkan-1.4-core"):
        reject("policy misrepresents VCTS as a Docs or core-selector source")
    require_false_flags(policy["effects"])
    actual = hashlib.sha256(canonical(policy)).hexdigest()
    if digest(policy["policy_sha256"], "policy_sha256") != actual:
        reject("policy sha256 does not bind its contents")
    return DerivedDocsPolicy(actual, REVISION, MAX_SOURCE_MEMBER_BYTES)


def policy(path: Path = POLICY, requirements: Path = REQUIREMENTS) -> DerivedDocsPolicy:
    return policy_value(document(path), requirements)


def main() -> None:
    try:
        result = policy(Path(sys.argv[1]) if len(sys.argv) == 2 else POLICY)
    except PolicyError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"POLICY: {result.source_revision} raw-cap={result.source_member_cap} unadmitted")


if __name__ == "__main__":
    main()
