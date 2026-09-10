#!/usr/bin/env python3
"""Bind historical Docs witnesses without promoting them to a successor closure."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
POLICY_DIR = HERE.parents[1] / "01-derived-docs-policy"
sys.path.insert(0, str(POLICY_DIR))
import derived_docs_policy as source_policy

RECORD = HERE / "successor_closure_anchor.json"
HISTORY = HERE.parents[3] / "02-vulkan-docs-closure/03-bind-vulkan-docs"
BUILD = HISTORY / "02-actual-closure-identity/vulkan_docs_build_witness.json"
SCOPE = HISTORY / "03-capture-core-closure/02-bind-core-input-scope/vulkan_docs_core_input_scope.json"
COMPARISON = HISTORY / "03-capture-core-closure/03-compare-fresh-captures/vulkan_docs_core_input_comparison.json"
sys.path.insert(0, str(BUILD.parent))
import vulkan_docs_identity_contract as legacy_contract
TARGET = ("vulkan-1.4-core", "api-limit-format-spec", "vulkan-14-spec")
TARGET_FIELDS = ("profile", "role", "required_input_id")
EFFECTS = frozenset(("admission_eligible", "admitted", "cutover_ready", "f03_changed", "supported",
                     "conformant", "certified", "near_native"))
FACTS = frozenset(("historical_evidence_is_fresh_closure", "rendered_output_can_satisfy_source",
                   "source_role_proved"))


class AnchorError(ValueError):
    """The historical witness is not a valid successor-policy anchor."""


def reject(message: str) -> None:
    raise AnchorError(message)


def exact(value: object, fields: frozenset[str], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != fields:
        reject(f"{label} has an invalid schema")
    return value


def digest(value: object, label: str) -> str:
    try:
        return source_policy.digest(value, label)
    except source_policy.PolicyError as error:
        reject(str(error))


def fixed(value: dict[str, object], expected: tuple[tuple[str, object, type], ...]) -> bool:
    return all(type(value.get(key)) is kind and value[key] == item for key, item, kind in expected)


def canonical(value: dict[str, object]) -> bytes:
    return json.dumps({key: item for key, item in value.items() if key != "anchor_sha256"},
                      sort_keys=True, separators=(",", ":")).encode()


def document(path: Path, label: str) -> dict[str, object]:
    try:
        return source_policy.document(path, label)
    except source_policy.PolicyError as error:
        reject(str(error))


def policy_digest() -> str:
    try:
        return source_policy.policy()
    except source_policy.PolicyError as error:
        reject(f"derived Docs policy is invalid: {error}")


def historical_document(key: str) -> dict[str, object]:
    path, expected = source_policy.ANCHORS[key]
    try:
        raw = source_policy.bounded_bytes(path, key)
        if hashlib.sha256(raw).hexdigest() != expected:
            reject(f"{key} does not retain its immutable byte identity")
        return source_policy.decoded(raw, key)
    except source_policy.PolicyError as error:
        reject(str(error))


def historical() -> tuple[dict[str, object], dict[str, object], dict[str, object]]:
    return (historical_document("build_witness_sha256"), historical_document("scope_manifest_sha256"),
            historical_document("capture_comparison_sha256"))


def require_history(record: dict[str, object]) -> None:
    build, scope, comparison = historical()
    try:
        legacy_contract.witness_value(build)
    except legacy_contract.DocsIdentityError as error:
        reject(f"historical build witness lacks the exact pinned configuration: {error}")
    if tuple(build.get(key) for key in ("profile", "role", "required_input_id", "status")) != (*TARGET, "build-witness-only-unadmitted"):
        reject("historical build witness has an unexpected target or status")
    if tuple(scope.get(key) for key in ("profile", "role", "required_input_id", "status")) != (*TARGET, "input-scope-only-unadmitted"):
        reject("historical input scope has an unexpected target or status")
    if tuple(comparison.get(key) for key in ("profile", "role", "required_input_id", "status")) != (*TARGET, "input-comparison-only-unadmitted"):
        reject("historical capture comparison has an unexpected target or status")
    expected = (("build_witness_sha256", build.get("witness_sha256"), str),
                ("scope_receipt_sha256", scope.get("scope_receipt_sha256"), str),
                ("comparison_sha256", comparison.get("comparison_sha256"), str))
    if not fixed(record["historical"], expected):
        reject("historical record identities are stale or incomplete")
    inputs, counts, tree = scope.get("inputs"), comparison.get("counts"), build.get("output_tree")
    if (not isinstance(inputs, dict) or not isinstance(counts, dict) or not isinstance(tree, dict)
            or not fixed(record["counts"], (("raw", inputs.get("raw_count"), int),
                                              ("derived", inputs.get("derived_count"), int),
                                              ("captures", counts.get("capture_count"), int),
                                              ("output_files", tree.get("file_count"), int)))):
        reject("historical record counts are stale or incomplete")
    output, output_witness = build.get("outputs"), comparison.get("output_witness")
    if (not isinstance(output, list) or len(output) != 1 or not isinstance(output[0], dict)
            or not isinstance(output_witness, dict)
            or tuple(output[0].get(key) for key in ("kind", "id", "bytes")) != ("rendered-output", "vkspec-html", 10377052)
            or output_witness.get("relation") != "two-distinct-observations-known-witness-only"):
        reject("historical output is not separately classified")


def anchor_value(value: object) -> str:
    fields = frozenset(("schema", "contract", "status", "profile", "role", "required_input_id", "policy_sha256",
                        "historical", "counts", "facts", "effects", "anchor_sha256"))
    record = exact(value, fields, "successor closure anchor")
    if type(record.get("schema")) is not int or tuple(record.get(key) for key in ("schema", "contract", "status", *TARGET_FIELDS)) != (
            1, "vulkan-docs-successor-closure-anchor-v1", "historical-anchor-unadmitted", *TARGET):
        reject("anchor has an unexpected contract or target")
    if digest(record["policy_sha256"], "policy_sha256") != policy_digest():
        reject("anchor does not bind the derived Docs policy")
    anchors = exact(record["historical"], frozenset(("build_witness_sha256", "scope_receipt_sha256", "comparison_sha256")), "historical")
    record["historical"] = anchors
    record["counts"] = exact(record["counts"], frozenset(("raw", "derived", "captures", "output_files")), "counts")
    facts = exact(record["facts"], FACTS, "facts")
    effects = exact(record["effects"], EFFECTS, "effects")
    if any(item is not False for item in facts.values()) or any(item is not False for item in effects.values()):
        reject("anchor attempts to claim fresh, source, admission, or release effects")
    require_history(record)
    actual = hashlib.sha256(canonical(record)).hexdigest()
    if digest(record["anchor_sha256"], "anchor_sha256") != actual:
        reject("anchor sha256 does not bind its contents")
    return actual


def anchor(path: Path = RECORD) -> str:
    return anchor_value(document(path, "successor closure anchor"))


def main() -> None:
    try:
        if len(sys.argv) not in (1, 2):
            reject("usage: successor_closure_anchor.py [ANCHOR.json]")
        anchor(Path(sys.argv[1]) if len(sys.argv) == 2 else RECORD)
    except AnchorError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print("ANCHOR: historical 298 raw 1462 derived unadmitted")


if __name__ == "__main__":
    main()
