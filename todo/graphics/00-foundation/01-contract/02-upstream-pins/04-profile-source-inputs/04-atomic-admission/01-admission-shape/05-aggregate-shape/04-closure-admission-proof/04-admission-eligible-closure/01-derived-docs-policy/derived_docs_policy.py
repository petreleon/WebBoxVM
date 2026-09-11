#!/usr/bin/env python3
"""Fail closed around a future Vulkan-Docs-derived source-policy envelope."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path

from safe_reader import ReaderError, bounded_bytes as safely_bounded_bytes

HERE = Path(__file__).resolve().parent
POLICY = HERE / "derived_docs_policy.json"
REQUIREMENTS = HERE.parents[7] / "03-feature-matrix/01-profile-scope/source_requirements.json"
ANCHORS = {
    "build_witness_sha256": (HERE.parents[2] / "02-vulkan-docs-closure/03-bind-vulkan-docs/02-actual-closure-identity/vulkan_docs_build_witness.json", "eaaaeee1286c73f06f06d522ed225a593274d943f496c92179bcdc8260823069"),
    "scope_manifest_sha256": (HERE.parents[2] / "02-vulkan-docs-closure/03-bind-vulkan-docs/03-capture-core-closure/02-bind-core-input-scope/vulkan_docs_core_input_scope.json", "82a258127c28e00277c0548ebbd2da2c13f383dc5bc0756a742f910736a61730"),
    "capture_comparison_sha256": (HERE.parents[2] / "02-vulkan-docs-closure/03-bind-vulkan-docs/03-capture-core-closure/03-compare-fresh-captures/vulkan_docs_core_input_comparison.json", "7fac74d8b2dcb56bbb2caa4c52f570be254984babe71e9c18ae52e76a4545aed"),
}
MAX_DOCUMENT_BYTES = 64 * 1024
MAX_SOURCE_MEMBER_BYTES = 8 * 1024 * 1024
DIGEST = re.compile(r"^[0-9a-f]{64}$")
REVISION = "f84d432d5b8912362f96f581f29bbc4f3c8c7843"
ROOT_SHA256 = "069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0"
REQUIREMENTS_SHA256 = "a9535611bab654034357d1578c3a2035caa09fda8090114df0867ac6d7fbeca5"
REQUIRED = ("vulkan-1.4-core", "api-limit-format-spec", "vulkan-14-spec")
TARGET_FIELDS = ("profile", "role", "required_input_id")
EFFECTS = frozenset(("inventory_changed", "cache_freshness_proved", "f03_changed", "supported",
                     "conformant", "certified", "near_native", "admitted", "admission_eligible",
                     "cutover_ready", "satisfies_vulkan_14_core_manifest"))

class PolicyError(ValueError):
    """The policy cannot be used as a trusted successor boundary."""

def reject(message: str) -> None:
    raise PolicyError(message)


def pairs(items: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in items:
        if key in result:
            reject(f"duplicate JSON key {key!r}")
        result[key] = value
    return result

def bounded_bytes(path: Path, label: str) -> bytes:
    try:
        return safely_bounded_bytes(path, label, MAX_DOCUMENT_BYTES)
    except ReaderError as error:
        reject(str(error))


def decoded(raw: bytes, label: str) -> dict[str, object]:
    try:
        result = json.loads(raw, object_pairs_hook=pairs)
    except (UnicodeDecodeError, json.JSONDecodeError, PolicyError) as error:
        reject(f"{label} is invalid: {error}")
    if not isinstance(result, dict):
        reject(f"{label} must be an object")
    return result


def document(path: Path, label: str = "policy document") -> dict[str, object]:
    return decoded(bounded_bytes(path, label), label)


def exact(value: object, fields: frozenset[str], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != fields:
        reject(f"{label} has an invalid schema")
    return value


def fixed(value: dict[str, object], expected: tuple[tuple[str, object, type], ...]) -> bool:
    return all(type(value.get(key)) is kind and value[key] == item for key, item, kind in expected)


def digest(value: object, label: str) -> str:
    if not isinstance(value, str) or not DIGEST.fullmatch(value) or value == "0" * 64:
        reject(f"{label} has an invalid sha256")
    return value


def canonical(value: dict[str, object]) -> bytes:
    return json.dumps({key: item for key, item in value.items() if key != "policy_sha256"},
                      sort_keys=True, separators=(",", ":")).encode()


def require_requirements(value: object) -> None:
    if digest(value, "source_requirements_sha256") != REQUIREMENTS_SHA256:
        reject("policy does not retain the reviewed F03 source-requirements identity")
    raw = bounded_bytes(REQUIREMENTS, "F03 source requirements")
    if hashlib.sha256(raw).hexdigest() != REQUIREMENTS_SHA256:
        reject("policy does not bind the current F03 source requirements")
    rows = decoded(raw, "F03 source requirements").get("requirements")
    if not isinstance(rows, list) or not any(tuple(row.get(key) for key in TARGET_FIELDS) == REQUIRED
                                             for row in rows if isinstance(row, dict)):
        reject("policy does not preserve the Vulkan 1.4 source requirement")


def require_anchors(value: object) -> None:
    anchors = exact(value, frozenset(ANCHORS), "anchors")
    for key, (path, expected) in ANCHORS.items():
        if digest(anchors[key], key) != expected or hashlib.sha256(bounded_bytes(path, key)).hexdigest() != expected:
            reject(f"policy does not bind the reviewed {key}")

def policy_value(value: object) -> str:
    fields = frozenset(("schema", "contract", "status", *TARGET_FIELDS, "source_requirements_sha256",
                        "source", "anchors", "build", "generated", "vcts", "effects", "policy_sha256"))
    policy = exact(value, fields, "derived Docs policy")
    if type(policy.get("schema")) is not int or tuple(policy.get(key) for key in ("schema", "contract", "status", *TARGET_FIELDS)) != (
            1, "vulkan-docs-derived-source-policy-v1", "policy-only-unadmitted", *REQUIRED):
        reject("policy has an unexpected contract or target")
    require_requirements(policy["source_requirements_sha256"])
    source = exact(policy["source"], frozenset(("repository", "revision", "root_selector", "root_url",
                   "root_sha256", "raw_member_max_bytes", "tree_identity_required", "immutable_https_required",
                   "tree_manifest_contract")), "source")
    if tuple(source.get(key) for key in ("repository", "revision", "root_selector", "root_url", "root_sha256")) != (
            "KhronosGroup/Vulkan-Docs", REVISION, "vkspec.adoc",
            f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{REVISION}/vkspec.adoc", ROOT_SHA256):
        reject("policy does not retain the reviewed immutable Docs root")
    if not fixed(source, (("raw_member_max_bytes", MAX_SOURCE_MEMBER_BYTES, int), ("tree_identity_required", True, bool),
                          ("immutable_https_required", True, bool), ("tree_manifest_contract", "vulkan-docs-derived-source-tree-v1", str))):
        reject("policy permits a root-only, mutable, or over-cap 8 MiB source")
    require_anchors(policy["anchors"])
    build = exact(policy["build"], frozenset(("required_argv_tokens", "pinned_builder_required", "network",
                  "two_fresh_runs_required", "scope_manifest_required", "clean_build_required",
                  "isolated_generated_path_required")), "build")
    if build.get("required_argv_tokens") != ["./makeSpec", "-spec", "core", "-version", "1.4", "html"]:
        reject("policy does not require the Vulkan 1.4 core build route")
    if not fixed(build, (("pinned_builder_required", True, bool), ("network", "none", str),
                         ("two_fresh_runs_required", True, bool), ("scope_manifest_required", True, bool),
                         ("clean_build_required", True, bool), ("isolated_generated_path_required", True, bool))):
        reject("policy permits an unpinned, networked, incomplete, or shared build")
    generated = exact(policy["generated"], frozenset(("rendered_output_may_satisfy_source",
                      "output_member_max_bytes", "authoritative_license_expression_required",
                      "attribution_manifest_required", "producer_lineage_required")), "generated")
    if not fixed(generated, (("rendered_output_may_satisfy_source", False, bool), ("output_member_max_bytes", 32 * 1024 * 1024, int),
                             ("authoritative_license_expression_required", True, bool), ("attribution_manifest_required", True, bool),
                             ("producer_lineage_required", True, bool))):
        reject("policy weakens generated-output, license, or lineage separation")
    vcts = exact(policy["vcts"], frozenset(("can_satisfy_docs", "local_core_selector_allowed",
                 "canonical_suite_scope")), "vcts")
    if not fixed(vcts, (("can_satisfy_docs", False, bool), ("local_core_selector_allowed", False, bool),
                        ("canonical_suite_scope", "broader-than-vulkan-1.4-core", str))):
        reject("policy misrepresents VCTS as a Docs or core-selector source")
    effects = exact(policy["effects"], EFFECTS, "effects")
    if any(item is not False for item in effects.values()):
        reject("policy attempts to claim an active, support, or release effect")
    actual = hashlib.sha256(canonical(policy)).hexdigest()
    if digest(policy["policy_sha256"], "policy_sha256") != actual:
        reject("policy sha256 does not bind its contents")
    return actual

def policy(path: Path = POLICY) -> str:
    return policy_value(document(path))


def main() -> None:
    try:
        policy(Path(sys.argv[1]) if len(sys.argv) == 2 else POLICY)
    except PolicyError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"POLICY: {REVISION} raw-cap={MAX_SOURCE_MEMBER_BYTES} unadmitted")

if __name__ == "__main__":
    main()
