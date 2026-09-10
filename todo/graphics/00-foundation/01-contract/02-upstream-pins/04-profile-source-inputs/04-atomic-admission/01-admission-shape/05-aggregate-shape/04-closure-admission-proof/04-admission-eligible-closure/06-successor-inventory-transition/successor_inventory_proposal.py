#!/usr/bin/env python3
"""Validate a non-evidentiary schema-v3 shape fixture without admitting or writing it."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path, PurePosixPath

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import successor_inventory_transition as transition

MAX_CANDIDATE_BYTES = 8 * 1024 * 1024
REVISION = "f84d432d5b8912362f96f581f29bbc4f3c8c7843"
MEMBER_FIELDS = frozenset((*transition.LEGACY_FIELDS, "selector", "source_role", "closure_sha256"))
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9._-]*$")


class ProposalError(ValueError):
    """The proposed successor inventory cannot safely be considered."""


def reject(message: str) -> None:
    raise ProposalError(message)


def exact(value: object, fields: frozenset[str], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != fields:
        reject(f"{label} has an invalid schema")
    return value


def digest(value: object, label: str) -> str:
    try:
        return transition.digest(value, label)
    except transition.TransitionError as error:
        reject(str(error))


def canonical(value: dict[str, object]) -> bytes:
    return json.dumps({key: item for key, item in value.items() if key != "candidate_sha256"},
                      sort_keys=True, separators=(",", ":")).encode()


def raw_closure_sha256(members: list[dict[str, object]]) -> str:
    identities = [{key: value for key, value in row.items() if key not in ("local_cache", "closure_sha256")}
                  for row in members]
    return hashlib.sha256(json.dumps(identities, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def member(value: object, closure_sha256: str) -> dict[str, object]:
    row = exact(value, MEMBER_FIELDS, "Docs raw member")
    identifier, selector, revision = row["id"], row["selector"], row["revision"]
    parts = selector.split("/") if isinstance(selector, str) else ()
    if (not isinstance(identifier, str) or not IDENTIFIER.fullmatch(identifier) or not isinstance(selector, str)
            or not selector or selector.startswith("/") or any(part in ("", ".", "..") for part in parts)
            or PurePosixPath(selector).as_posix() != selector or any(char in selector for char in "?#%\\\\")
            or any(ord(char) < 33 for char in selector)):
        reject("Docs raw member has an unsafe id or selector")
    if type(row["bytes"]) is not int or not 0 < row["bytes"] <= 8 * 1024 * 1024:
        reject("Docs raw member violates the 8 MiB cap")
    for key in MEMBER_FIELDS - {"bytes"}:
        if not isinstance(row[key], str) or not row[key]:
            reject(f"Docs raw member has an invalid {key}")
    if row["source_family"] != "vulkan-docs" or row["source_role"] != "api-limit-format-spec" or revision != REVISION:
        reject("Docs raw member has an unexpected family, role, or revision")
    if (row["license"], row["generated_code_role"], row["provenance"]) != (
            "shape-only-unverified", "shape-only-no-output", "shape-only-unverified"):
        reject("shape fixture attempts to invent member authority or generated output")
    prefix = f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{REVISION}/"
    if row["immutable_url"] != prefix + selector or row["closure_sha256"] != closure_sha256:
        reject("Docs raw member lacks an immutable pinned URL or closure binding")
    source_sha256 = digest(row["sha256"], "Docs raw member sha256")
    path = PurePosixPath(row["local_cache"])
    expected = PurePosixPath("webboxvm-graphics/f02-successor/vulkan-docs") / closure_sha256 / identifier / f"{source_sha256}.source"
    if path.is_absolute() or str(path) != row["local_cache"] or path != expected:
        reject("Docs raw member has an unsafe schema-v3 cache path")
    return row


def proposal_value(value: object) -> str:
    fields = frozenset(("schema", "contract", "status", "transition_sha256", "predecessor_lock_sha256",
                        "legacy_records", "source_families", "vulkan_docs", "effects", "candidate_sha256"))
    record = exact(value, fields, "schema-v3 proposal")
    try:
        design = transition.transition()
        active = transition.active_inventory()
    except transition.TransitionError as error:
        reject(f"transition prerequisite is invalid: {error}")
    if type(record.get("schema")) is not int or (record["schema"], record["contract"], record["status"]) != (
            3, "vulkan-docs-successor-inventory-v3", "shape-only-unadmitted"):
        reject("proposal has an unexpected schema, contract, or status")
    if digest(record["transition_sha256"], "transition_sha256") != design or record["predecessor_lock_sha256"] != transition.LOCK_SHA256:
        reject("proposal does not bind the reviewed transition and predecessor lock")
    legacy = [dict(item) for item in active.inputs]
    expected_families = [*transition.ACTIVE_FAMILIES, "vulkan-docs"]
    if record["legacy_records"] != legacy or record["source_families"] != expected_families:
        reject("proposal does not preserve every predecessor record and exactly one new family")
    wrapper = exact(record["vulkan_docs"], frozenset(("source_family", "raw_members", "closure",
                    "generated_outputs_are_raw_sources")), "vulkan-docs family wrapper")
    if wrapper["source_family"] != "vulkan-docs" or wrapper["generated_outputs_are_raw_sources"] is not False:
        reject("proposal misclassifies the sole vulkan-docs family wrapper")
    closure = exact(wrapper["closure"], frozenset(("root_id", "ordered_member_ids", "raw_member_count",
                    "raw_closure_sha256", "authority_manifest_sha256", "write_lineage_sha256",
                    "fresh_authorized_capture")), "proposal closure")
    closure_sha256 = digest(closure["raw_closure_sha256"], "raw_closure_sha256")
    raw = wrapper["raw_members"]
    if type(raw) is not list or len(raw) < 2:
        reject("proposal requires a root-and-nonroot Docs closure")
    members = [member(item, closure_sha256) for item in raw]
    identifiers = [item["id"] for item in members]
    if (len(set(identifiers)) != len(identifiers) or len({item["selector"] for item in members}) != len(members)
            or len({item["local_cache"] for item in members}) != len(members)):
        reject("proposal has duplicate Docs member identities, selectors, or caches")
    if (set(identifiers) & {item["id"] for item in legacy} or closure["root_id"] != identifiers[0]
            or members[0]["selector"] != "vkspec.adoc" or members[0]["sha256"] != transition.ROOT_SHA256
            or closure["ordered_member_ids"] != identifiers
            or type(closure["raw_member_count"]) is not int or closure["raw_member_count"] != len(members)
            or closure_sha256 != raw_closure_sha256(members)):
        reject("proposal does not retain an ordered complete Docs closure")
    if (closure["authority_manifest_sha256"], closure["write_lineage_sha256"], closure["fresh_authorized_capture"]) != (
            None, None, False):
        reject("shape fixture carries unproven closure authority, lineage, or freshness")
    effects = exact(record["effects"], transition.EFFECTS, "proposal effects")
    if any(type(item) is not bool or item is not False for item in effects.values()):
        reject("proposal attempts to self-admit or claim an implementation effect")
    actual = hashlib.sha256(canonical(record)).hexdigest()
    if digest(record["candidate_sha256"], "candidate_sha256") != actual:
        reject("proposal sha256 does not bind its contents")
    return actual


def proposal(path: Path) -> str:
    try:
        value = transition.document(path, "schema-v3 proposal", MAX_CANDIDATE_BYTES)
    except transition.TransitionError as error:
        reject(str(error))
    return proposal_value(value)


def main() -> None:
    try:
        if len(sys.argv) != 2:
            reject("usage: successor_inventory_proposal.py PROPOSAL.json")
        value = proposal(Path(sys.argv[1]))
    except ProposalError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"PROPOSAL: schema-v3 shape-only unadmitted; {value}")


if __name__ == "__main__":
    main()
