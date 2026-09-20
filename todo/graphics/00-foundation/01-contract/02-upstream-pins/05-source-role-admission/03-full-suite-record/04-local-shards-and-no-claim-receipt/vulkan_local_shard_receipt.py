#!/usr/bin/env python3
"""Publish a self-hashed, no-claim receipt for local VCTS api.txt shards."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import vulkan_local_shards as local  # noqa: E402

FRESH_RECEIPT = HERE.parent / "03-vulkan-default-full-ledger/03-fresh-full-suite-receipt/vulkan_full_suite_receipt.json"
FRESH_RECEIPT_SHA256 = "406130b30ca18b1843962456a5309ab24d41b222213b9a9d0e1cac06b6c98bb0"


class ReceiptError(ValueError):
    """A shard receipt overstates local provenance as qualification."""


def reject(message: str) -> None:
    raise ReceiptError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def fresh_receipt() -> str:
    try:
        value = json.loads(FRESH_RECEIPT.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        reject(f"F02.5.3.3 receipt is unavailable: {error}")
    if not isinstance(value, dict) or value.get("receipt_sha256") != FRESH_RECEIPT_SHA256:
        reject("F02.5.3.3 receipt identity changed")
    return FRESH_RECEIPT_SHA256


def catalog_parts(value: object) -> tuple[dict[str, object], dict[str, object], list[dict[str, object]]]:
    records = value.get("records") if isinstance(value, dict) else None
    if not isinstance(records, list) or len(records) != 7 or not all(isinstance(item, dict) for item in records):
        reject("shard catalog has an invalid record count")
    root, member, shards = records[0], records[1], records[2:]
    if not local.replay.alike(value, local.catalog(root)):
        reject("shard catalog is not the exact ordered local transform")
    return root, member, shards


def checked_cache(value: object, root: dict[str, object]) -> dict[str, object]:
    required = {"claims", "cts_executions", "source_root", "ledger_sha256", "member_count",
                "member_total_bytes", "receipt_sha256"}
    root_identity = {key: item for key, item in root.items() if key != "artifact"}
    cached_root = value.get("source_root") if isinstance(value, dict) else None
    cached_identity = ({key: item for key, item in cached_root.items() if key != "artifact"}
                       if isinstance(cached_root, dict) else None)
    if (not isinstance(value, dict) or not required <= set(value) or type(value["cts_executions"]) is not int
            or value["cts_executions"] != 0 or not local.replay.alike(value["claims"], local.ledger_map.NO_CLAIMS)
            or not local.replay.alike(cached_identity, root_identity)):
        reject("offline VCTS cache receipt is not the exact no-claim replay")
    ledger = local.ledger_map.build()["ledger"]
    assert isinstance(ledger, dict)
    if (value["ledger_sha256"], value["member_count"], value["member_total_bytes"]) != (
            ledger["ledger_sha256"], ledger["member_count"], ledger["member_total_bytes"]):
        reject("offline VCTS cache receipt does not bind the pinned ledger")
    return value


def build_from(catalog: object, cache: object) -> dict[str, object]:
    root, member, shards = catalog_parts(catalog)
    checked = checked_cache(cache, root)
    source = {key: member[key] for key in ("id", "immutable_url", "revision", "member_path", "sha256", "bytes")}
    pieces = [{"id": item["id"], "index": item["shard"]["index"], "count": item["shard"]["count"],
               "offset": item["shard"]["offset"], "bytes": item["bytes"], "sha256": item["sha256"],
               "artifact": item["artifact"]} for item in shards]
    body = {"schema": 1, "kind": "webboxvm-vulkan-api-byte-preserving-shard-receipt",
            "authority": "WebBoxVM", "producer": "WebBoxVM", "claims": dict(local.NO_CLAIMS), "cts_executions": 0,
            "source_root": {key: root[key] for key in ("id", "revision", "sha256", "bytes", "selector_path", "unfiltered")},
            "source_member": {**source, "blob_sha1": local.API_BLOB_SHA1},
            "cache": {"mode": "offline-replay", "raw_https_requests": 0, "receipt_sha256": checked["receipt_sha256"],
                      "identity_sha256": local.ledger_map.build()["ledger"]["identity_sha256"],
                      "ledger_sha256": checked["ledger_sha256"], "fresh_full_suite_receipt_sha256": fresh_receipt()},
            "builder": dict(shards[0]["builder"]), "mode": builder_mode(shards), "max_local_bytes": local.builder.MAX_LOCAL_BYTES,
            "shards": pieces, "reassembled_sha256": member["sha256"], "catalog_sha256": hashlib.sha256(canonical(catalog)).hexdigest(),
            "states": {"admitted": False, "cutover_ready": False, "satisfies_vulkan_14_core_manifest": False}}
    return {**body, "receipt_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def builder_mode(shards: list[dict[str, object]]) -> str:
    if any(item["command"][1] != "--mode=byte-preserving-shard" for item in shards):
        reject("catalog does not use the pinned byte-preserving builder mode")
    return "byte-preserving-shard"


def validate_from(value: object, catalog: object, cache: object) -> None:
    if not local.replay.alike(value, build_from(catalog, cache)):
        reject("shard receipt differs from exact cache and artifact evidence")


def admit(cache_root: Path, artifact_root: Path) -> dict[str, object]:
    catalog, cache = local.build(cache_root, artifact_root)
    return build_from(catalog, cache)


def validate(value: object, cache_root: Path, artifact_root: Path) -> None:
    catalog, cache = local.validate(cache_root, artifact_root)
    validate_from(value, catalog, cache)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--artifact-root", type=Path, required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(admit(args.cache_root, args.artifact_root), sort_keys=True))
    except (ReceiptError, local.ShardError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
