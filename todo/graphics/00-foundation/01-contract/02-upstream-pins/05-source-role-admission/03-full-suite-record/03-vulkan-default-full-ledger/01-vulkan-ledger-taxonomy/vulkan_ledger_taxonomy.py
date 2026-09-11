#!/usr/bin/env python3
"""Bind the exact VCTS default ledger to a WebBoxVM no-claim observation."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "01-canonical-full-suite-roots"))
import full_suite_roots as roots  # noqa: E402

V2 = HERE.parents[3] / "04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/05-vulkan-source-contract-v2"
SCHEMA, LIVE, TAXONOMY = V2 / "02-canonical-suite-schema", V2 / "03-external-closure-cache/02-live-closure", V2 / "04-coverage-taxonomy"
IDENTITY, LEDGER, TAXONOMY_JSON = SCHEMA / "vcts_root_identity.json", LIVE / "vcts_closure_ledger.json", TAXONOMY / "coverage_taxonomy.json"
IDENTITY_SHA256 = "30b272f8c563e0dabf307795c01496eb70f744514b4790439ebbfc69c7bd5218"
LEDGER_SHA256 = "608d520463bfd4724e79b52ee639a12d446e14c5e8d7b3687872eaf3c4e917f0"
TAXONOMY_SHA256 = "752a3ae5ff10ea0d9f9a2638c0d1612fb7af3d376dabad1601f8b9c1e76cb2f7"
MEMBER_COUNT, MEMBER_BYTES = 98, 434669348
CATEGORY_COUNTS = {"core": 0, "wsi": 1, "video": 1, "extension": 4, "unknown": 92}


def load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load V2 contract: {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[name] = value
    spec.loader.exec_module(value)
    return value


identity = load("f02531_identity", SCHEMA / "canonical_suite_identity.py")
ledger = load("f02531_ledger", SCHEMA / "canonical_suite_ledger.py")
taxonomy = load("f02531_taxonomy", TAXONOMY / "coverage_taxonomy.py")
NO_CLAIMS = dict(roots.NO_CLAIMS)


class VulkanLedgerError(ValueError):
    """The Vulkan default-suite observation is mixed, incomplete, or overclaimed."""


def reject(message: str) -> None:
    raise VulkanLedgerError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def alike(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return set(left) == set(right) and all(alike(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(alike(one, two) for one, two in zip(left, right))
    return left == right


def source_root():
    value = roots.catalog()
    roots.validate_full_suite_catalog(value)
    record = next((item for item in value["records"] if item["id"] == "vulkan-cts-default"), None)
    if record is None:
        reject("F02.5.3 catalog has no Vulkan default root")
    return record, roots.proof_for(record)


def inputs(identity_path: Path, ledger_path: Path, taxonomy_path: Path):
    record, proof = source_root()
    try:
        suite, closure = identity.validate(identity_path), ledger.validate(ledger_path, identity_path)
        raw, view = ledger.document(ledger_path), taxonomy.classify(taxonomy_path, identity_path, ledger_path)
    except (identity.IdentityError, ledger.LedgerError, taxonomy.TaxonomyError) as error:
        reject(f"V2 Vulkan evidence is invalid: {error}")
    bridge = (record["revision"], record["selector_path"], record["sha256"], record["bytes"], proof["tag"], proof["tag_object"])
    expected = (suite.peeled_commit, suite.root_path, identity.ROOT["sha256"], identity.ROOT["bytes"],
                identity.EXPECTED["tag_name"], identity.EXPECTED["tag_object_sha1"])
    members = raw["members"]
    if (bridge != expected or (suite.digest, closure.digest, view.taxonomy_digest) != (IDENTITY_SHA256, LEDGER_SHA256, TAXONOMY_SHA256)
            or (ledger.digest(raw), view.identity_digest, view.revision, view.ledger_digest) != (closure.digest, suite.digest, suite.peeled_commit, closure.digest)
            or (closure.member_count, closure.total_bytes, len(members)) != (MEMBER_COUNT, MEMBER_BYTES, MEMBER_COUNT)
            or tuple(item["path"] for item in members) != suite.direct_members
            or any(item["parent_path"] is not None for item in members)):
        reject("Vulkan root bridge or exact direct closure differs from the reviewed release")
    if tuple(item["path"] for item in view.members) != suite.direct_members:
        reject("Vulkan taxonomy does not preserve exact ledger order")
    counts = {name: sum(item["category"] == name for item in view.members) for name in CATEGORY_COUNTS}
    if counts != CATEGORY_COUNTS:
        reject("Vulkan taxonomy has inferred or reclassified member coverage")
    return record, proof, suite, closure, members, view, counts


def build(identity_path: Path = IDENTITY, ledger_path: Path = LEDGER, taxonomy_path: Path = TAXONOMY_JSON) -> dict[str, object]:
    record, proof, suite, closure, members, view, counts = inputs(identity_path, ledger_path, taxonomy_path)
    observed = []
    for raw, classified in zip(members, view.members):
        if (raw["path"], raw["blob_sha1"], raw["sha256"]) != (classified["path"], classified["blob_sha1"], classified["member_sha256"]):
            reject("Vulkan taxonomy member does not bind its exact upstream identity")
        observed.append({**raw, "authority": "Khronos", "producer": "Khronos", "scope": "suite-member",
                         "category": classified["category"], "rule_id": classified["rule_id"],
                         "source_locator": classified["source_locator"]})
    body = {
        "schema": 1, "kind": "webboxvm-engineering-map", "authority": "WebBoxVM", "producer": "WebBoxVM",
        "claims": dict(NO_CLAIMS), "cts_executions": 0, "source_root": record,
        "bridge": {"f025_root_id": record["id"], "v2_suite_id": suite.suite_id, "release_tag": proof["tag"],
                   "tag_object_sha1": proof["tag_object"], "peeled_commit_sha1": suite.peeled_commit,
                   "selector_scope": "khronos-default-mustpass-broader-than-vulkan-1.4-core", "local_filtering": "forbidden"},
        "ledger": {"identity_sha256": suite.digest, "ledger_sha256": closure.digest, "member_count": closure.member_count,
                   "member_total_bytes": closure.total_bytes, "members": observed},
        "taxonomy": {"taxonomy_sha256": view.taxonomy_digest, "category_counts": counts},
        "states": {"admitted": False, "cutover_ready": False, "satisfies_vulkan_14_core_manifest": False},
    }
    return {**body, "record_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate_record(value: object, identity_path: Path = IDENTITY, ledger_path: Path = LEDGER,
                    taxonomy_path: Path = TAXONOMY_JSON) -> None:
    if not alike(value, build(identity_path, ledger_path, taxonomy_path)):
        reject("local Vulkan ledger record differs from exact V2 evidence")


def main() -> None:
    if sys.argv[1:]:
        raise SystemExit("usage: vulkan_ledger_taxonomy.py")
    try:
        print(json.dumps(build(), sort_keys=True))
    except VulkanLedgerError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
