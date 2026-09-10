#!/usr/bin/env python3
"""Fail closed around a design-only Vulkan-Docs inventory successor."""

from __future__ import annotations

import hashlib
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
RECORD = HERE / "successor_inventory_transition.json"
INVENTORY_DIR = HERE.parents[6] / "01-input-inventory"
sys.path.insert(0, str(INVENTORY_DIR))
import inventory_layout
from successor_inventory_common import (MAX_DOCUMENT_BYTES, ContractError as TransitionError, anchored,
                                        canonical, digest, document, exact, fixed, reject)

POLICY = HERE.parent / "01-derived-docs-policy/derived_docs_policy.json"
ANCHOR = HERE.parent / "02-derived-docs-closure/01-successor-closure-anchor/successor_closure_anchor.json"
BOUNDARY = HERE.parent / "03-source-release-boundary/source_release_boundary.json"
LOCK_SHA256 = "08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6"
POLICY_DOCUMENT_SHA256 = "86abe34a6c9c9b08707a1b85cc75e9366af5eca1bb90d3f68a86e7eb8d23f4e7"
POLICY_SHA256 = "9aac96fcdc340a31e77f44318fd32dbd673eb90551b5ac090d21d8b3a53078aa"
ANCHOR_DOCUMENT_SHA256 = "18635158b68d98665006d3420faed5341f66b4d7591ee70ff525e8538f6f1d5d"
ANCHOR_SHA256 = "65aa77910560fb32624d78c584857fab130769d408df2ece3d4a967cc983aa7c"
BOUNDARY_DOCUMENT_SHA256 = "4544bc95c6021fd85d134eea5d4f2913d4fdaf810e9058d74d220dda7d1cdc40"
BOUNDARY_SHA256 = "7cc44e773bb45e61535f535c93640547dfddead6b0b2f0bb23cc73c62ec06485"
ROOT_SHA256 = "069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0"
LEGACY_FIELDS = ("id", "source_family", "immutable_url", "revision", "sha256", "bytes", "license",
                 "local_cache", "generated_code_role", "provenance")
ACTIVE_FAMILIES = ("linux-uapi", "mesa-virgl", "mesa-venus", "virglrenderer", "venus-protocol",
                   "gl-gles-registry", "glsl", "essl", "vulkan", "spirv", "webgpu", "wgsl",
                   "wgsl-grammar", "vk-gl-cts", "webgpu-cts", "piglit", "webgpu-idl")
EFFECTS = frozenset(("active_inventory_changed", "f02_2_bypassed", "cache_freshness_proved",
                     "f03_changed", "admission_eligible", "admitted", "cutover_ready", "supported",
                     "conformant", "certified", "near_native"))
def active_inventory() -> inventory_layout.Inventory:
    try:
        inventory = inventory_layout.load_inventory(INVENTORY_DIR / "manifest.toml")
    except inventory_layout.InventoryLayoutError as error:
        reject(f"active F02 inventory is invalid: {error}")
    families = tuple(item["source_family"] for item in inventory.inputs)
    registry = [item for item in inventory.inputs if item["id"] == "vulkan-registry"]
    if inventory.schema != 2 or inventory.revision != LOCK_SHA256 or families != ACTIVE_FAMILIES:
        reject("active F02 schema-v2 or 17-family lock has drifted")
    if len(registry) != 1 or tuple(registry[0][key] for key in ("source_family", "revision", "sha256", "bytes")) != (
            "vulkan", "f84d432d5b8912362f96f581f29bbc4f3c8c7843",
            "cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06", 3309653):
        reject("active vulkan-registry predecessor has drifted")
    return inventory


def require_anchors() -> None:
    policy = anchored(POLICY, POLICY_DOCUMENT_SHA256, "derived Docs policy")
    anchor = anchored(ANCHOR, ANCHOR_DOCUMENT_SHA256, "historical closure anchor")
    boundary = anchored(BOUNDARY, BOUNDARY_DOCUMENT_SHA256, "source/release boundary")
    if (policy.get("policy_sha256"), anchor.get("anchor_sha256"), boundary.get("boundary_sha256")) != (
            POLICY_SHA256, ANCHOR_SHA256, BOUNDARY_SHA256):
        reject("reviewed policy, history, or source/release self-hash has drifted")


def transition_value(value: object) -> str:
    fields = frozenset(("schema", "contract", "status", "predecessor", "successor", "closure",
                        "generated", "cutover", "effects", "transition_sha256"))
    record = exact(value, fields, "successor transition")
    if type(record.get("schema")) is not int or (record["schema"], record["contract"], record["status"]) != (
            1, "vulkan-docs-successor-inventory-transition-v1", "design-only-unadmitted"):
        reject("transition has an unexpected contract or status")
    active_inventory()
    require_anchors()
    predecessor = exact(record["predecessor"], frozenset(("schema", "lock_sha256", "families", "legacy_fields",
                        "preserve_predecessor_raw_records", "active_inventory_changed", "successor_schema_required")), "predecessor")
    if (not fixed(predecessor, (("schema", 2, int), ("lock_sha256", LOCK_SHA256, str),
                                ("preserve_predecessor_raw_records", True, bool),
                                ("active_inventory_changed", False, bool), ("successor_schema_required", 3, int)))
            or predecessor["families"] != list(ACTIVE_FAMILIES) or predecessor["legacy_fields"] != list(LEGACY_FIELDS)):
        reject("transition does not preserve the exact active schema-v2 predecessor")
    successor = exact(record["successor"], frozenset(("schema", "contract", "new_family", "retains_family",
                      "legacy_vulkan_alias_allowed", "root_selector", "root_sha256", "raw_member_max_bytes", "raw_member_fields", "family_cardinality",
                      "minimum_members", "root_and_nonroot_required", "immutable_https_required", "pinned_revision_required",
                      "active_cache_grammar_compatible", "cache_template", "generated_outputs_are_raw_sources")), "successor")
    expected_cache = "webboxvm-graphics/f02-successor/vulkan-docs/{closure_sha256}/{id}/{sha256}.source"
    if (not fixed(successor, (("schema", 3, int), ("contract", "vulkan-docs-successor-inventory-v3", str),
                              ("new_family", "vulkan-docs", str), ("retains_family", "vulkan", str),
                              ("legacy_vulkan_alias_allowed", False, bool), ("raw_member_max_bytes", 8 * 1024 * 1024, int),
                              ("root_selector", "vkspec.adoc", str), ("root_sha256", ROOT_SHA256, str),
                              ("family_cardinality", "one-wrapper-with-ordered-raw-members", str),
                              ("minimum_members", 2, int), ("root_and_nonroot_required", True, bool),
                              ("immutable_https_required", True, bool), ("pinned_revision_required", True, bool),
                              ("active_cache_grammar_compatible", False, bool), ("cache_template", expected_cache, str),
                              ("generated_outputs_are_raw_sources", False, bool)))
            or successor["raw_member_fields"] != [*LEGACY_FIELDS, "selector", "source_role", "closure_sha256"]):
        reject("transition weakens the schema-v3 Docs family or cache boundary")
    closure = exact(record["closure"], frozenset(("policy_document_sha256", "policy_sha256",
                    "historical_anchor_document_sha256", "historical_anchor_sha256", "source_release_boundary_document_sha256",
                    "source_release_boundary_sha256", "fresh_complete_raw_closure_required", "root_and_nonroot_required",
                    "ordered_member_ids_required", "raw_closure_sha256_required", "historical_counts_are_current")), "closure")
    if (not fixed(closure, (("policy_document_sha256", POLICY_DOCUMENT_SHA256, str), ("policy_sha256", POLICY_SHA256, str),
                            ("historical_anchor_document_sha256", ANCHOR_DOCUMENT_SHA256, str),
                            ("historical_anchor_sha256", ANCHOR_SHA256, str),
                            ("source_release_boundary_document_sha256", BOUNDARY_DOCUMENT_SHA256, str),
                            ("source_release_boundary_sha256", BOUNDARY_SHA256, str),
                            ("fresh_complete_raw_closure_required", True, bool), ("root_and_nonroot_required", True, bool),
                            ("ordered_member_ids_required", True, bool), ("raw_closure_sha256_required", True, bool),
                            ("historical_counts_are_current", False, bool)))):
        reject("transition weakens the fresh Docs closure requirement")
    generated = exact(record["generated"], frozenset(("output_can_be_raw_source", "per_member_authority_fields",
                      "authority_manifest_sha256_required", "authority_ledger_must_cover_every_raw_member",
                      "write_lineage_sha256_required")), "generated output")
    if (not fixed(generated, (("output_can_be_raw_source", False, bool),
                              ("authority_manifest_sha256_required", True, bool),
                              ("authority_ledger_must_cover_every_raw_member", True, bool),
                              ("write_lineage_sha256_required", True, bool)))
            or generated["per_member_authority_fields"] != ["license-expression", "attribution", "role", "provenance", "producer-lineage"]):
        reject("transition weakens generated-output authority or lineage")
    cutover = exact(record["cutover"], frozenset(("atomic_f02_1_f02_2_revalidation_required",
                    "active_mutation_permitted_by_this_record", "shape_fixture_is_admission_evidence", "f03_transition_allowed",
                    "admission_requires_full_closure_authority_and_lineage")), "cutover")
    if not fixed(cutover, (("atomic_f02_1_f02_2_revalidation_required", True, bool),
                           ("active_mutation_permitted_by_this_record", False, bool),
                           ("shape_fixture_is_admission_evidence", False, bool),
                           ("f03_transition_allowed", False, bool),
                           ("admission_requires_full_closure_authority_and_lineage", True, bool))):
        reject("transition permits an active or partial cutover")
    effects = exact(record["effects"], EFFECTS, "transition effects")
    if any(type(item) is not bool or item is not False for item in effects.values()):
        reject("transition attempts to claim an active, support, or release effect")
    actual = hashlib.sha256(canonical(record, "transition_sha256")).hexdigest()
    if digest(record["transition_sha256"], "transition_sha256") != actual:
        reject("transition sha256 does not bind its contents")
    return actual


def transition(path: Path = RECORD) -> str:
    return transition_value(document(path))


def main() -> None:
    try:
        if len(sys.argv) not in (1, 2):
            reject("usage: successor_inventory_transition.py [TRANSITION.json]")
        value = transition(Path(sys.argv[1]) if len(sys.argv) == 2 else RECORD)
    except TransitionError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"TRANSITION: schema-v2 to future-v3; design-only unadmitted; {value}")


if __name__ == "__main__":
    main()
