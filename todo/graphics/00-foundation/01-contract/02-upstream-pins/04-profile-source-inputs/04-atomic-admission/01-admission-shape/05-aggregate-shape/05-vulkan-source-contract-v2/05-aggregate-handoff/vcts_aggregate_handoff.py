#!/usr/bin/env python3
"""Bind verified V2 closure evidence into a non-admitting aggregate handoff."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
V2 = HERE.parent
SCHEMA = V2 / "02-canonical-suite-schema"
LIVE = V2 / "03-external-closure-cache/02-live-closure"
TAXONOMY = V2 / "04-coverage-taxonomy"
IDENTITY = SCHEMA / "vcts_root_identity.json"
PLAN = LIVE / "vcts_tree_plan.json"
LEDGER = LIVE / "vcts_closure_ledger.json"
CACHE_RECEIPT = HERE / "vcts_cache_receipt.json"
CAPTURE_RECEIPT = LIVE / "vcts_live_capture_receipt.json"
TAXONOMY_RECEIPT = TAXONOMY / "coverage_taxonomy.json"
HANDOFF = HERE / "vcts_aggregate_handoff.json"
for directory in (LIVE, TAXONOMY):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))
import coverage_taxonomy as taxonomy
import vcts_capture_receipt as capture

FIELDS = frozenset(("schema", "kind", "status", "root_identity_sha256", "tree_plan_sha256",
                    "ledger_sha256", "cache_receipt_sha256", "capture_receipt_sha256", "taxonomy_sha256",
                    "member_count", "member_total_bytes", "selector_scope", "local_filtering",
                    "docs_generated_artifacts", "admitted", "cutover_ready",
                    "satisfies_vulkan_14_core_manifest", "handoff_sha256"))
MAX_HANDOFF_BYTES = 64 * 1024
STATE = (False, False, False)


class HandoffError(ValueError):
    """The V2 handoff is incomplete, mixed, stale, or falsely admitting."""


def reject(message: str) -> None:
    raise HandoffError(message)


def pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in rows:
        if key in value:
            reject("handoff has a duplicate JSON key")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    try:
        raw = path.read_bytes()
        if len(raw) > MAX_HANDOFF_BYTES:
            reject("handoff exceeds its JSON byte limit")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except HandoffError:
        raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"handoff cannot be read: {error}")
    if not isinstance(value, dict) or set(value) != FIELDS or type(value.get("schema")) is not int:
        reject("handoff has an unexpected schema")
    return value


def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "handoff_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def alike(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return set(left) == set(right) and all(alike(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(alike(one, two) for one, two in zip(left, right))
    return left == right


def inputs(identity_path: Path, plan_path: Path, ledger_path: Path, cache_receipt_path: Path,
           capture_receipt_path: Path, taxonomy_path: Path):
    try:
        root, plan, closure, ledger_document = capture.source.source_inputs(identity_path, plan_path, ledger_path)
        live = capture.validate(capture_receipt_path, identity_path, plan_path, ledger_path, cache_receipt_path)
        view = taxonomy.classify(taxonomy_path, identity_path, ledger_path)
    except (capture.CaptureReceiptError, taxonomy.TaxonomyError) as error:
        reject(f"V2 evidence is invalid: {error}")
    paths = tuple(row["path"] for row in ledger_document["members"])
    view_paths = tuple(row["path"] for row in view.members)
    state = (live["admitted"], live["cutover_ready"], live["satisfies_vulkan_14_core_manifest"])
    if (plan.member_count != closure.member_count or plan.total_bytes != closure.total_bytes
            or view_paths != paths or (view.identity_digest, view.ledger_digest) != (root.digest, closure.digest)
            or len(view.members) != closure.member_count or state != STATE):
        reject("V2 evidence is partial, mixed, stale, or falsely admitting")
    return root, plan, closure, live, view


def build(identity_path: Path = IDENTITY, plan_path: Path = PLAN, ledger_path: Path = LEDGER,
          cache_receipt_path: Path = CACHE_RECEIPT, capture_receipt_path: Path = CAPTURE_RECEIPT,
          taxonomy_path: Path = TAXONOMY_RECEIPT) -> dict[str, object]:
    root, plan, closure, live, view = inputs(identity_path, plan_path, ledger_path, cache_receipt_path,
                                             capture_receipt_path, taxonomy_path)
    value: dict[str, object] = {
        "schema": 1, "kind": "canonical-upstream-suite-handoff",
        "status": "verified-canonical-suite-receipt-unadmitted",
        "root_identity_sha256": root.digest, "tree_plan_sha256": plan.digest, "ledger_sha256": closure.digest,
        "cache_receipt_sha256": live["cache_receipt_sha256"], "capture_receipt_sha256": live["receipt_sha256"],
        "taxonomy_sha256": view.taxonomy_digest, "member_count": closure.member_count,
        "member_total_bytes": closure.total_bytes,
        "selector_scope": "khronos-default-mustpass-broader-than-vulkan-1.4-core",
        "local_filtering": "forbidden",
        "docs_generated_artifacts": "outside-admitted-implementation-source-closure",
        "admitted": False, "cutover_ready": False, "satisfies_vulkan_14_core_manifest": False,
        "handoff_sha256": "",
    }
    value["handoff_sha256"] = digest(value)
    return value


def validate(path: Path = HANDOFF, identity_path: Path = IDENTITY, plan_path: Path = PLAN,
             ledger_path: Path = LEDGER, cache_receipt_path: Path = CACHE_RECEIPT,
             capture_receipt_path: Path = CAPTURE_RECEIPT,
             taxonomy_path: Path = TAXONOMY_RECEIPT) -> dict[str, object]:
    value = document(path)
    if not isinstance(value["handoff_sha256"], str) or value["handoff_sha256"] != digest(value):
        reject("handoff self-hash is invalid")
    expected = build(identity_path, plan_path, ledger_path, cache_receipt_path, capture_receipt_path, taxonomy_path)
    if not alike(value, expected):
        reject("handoff does not bind the exact V2 closure")
    return value


def main() -> None:
    try:
        value = build() if sys.argv[1:] == ["--build"] else validate() if len(sys.argv) == 1 else None
        if value is None:
            raise SystemExit("usage: vcts_aggregate_handoff.py [--build]")
    except HandoffError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    if sys.argv[1:] == ["--build"]:
        print(json.dumps(value, indent=2, sort_keys=True))
    else:
        print(f"HANDOFF: {value['member_count']} members {value['handoff_sha256']} unadmitted")


if __name__ == "__main__":
    main()
