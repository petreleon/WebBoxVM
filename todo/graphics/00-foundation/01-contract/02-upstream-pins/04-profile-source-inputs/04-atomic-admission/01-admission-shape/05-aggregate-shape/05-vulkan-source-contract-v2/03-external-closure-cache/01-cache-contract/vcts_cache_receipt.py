"""Self-hashed, unadmitted receipt bound to one exact closure ledger."""

from __future__ import annotations

import hashlib
import json

from vcts_cache_fs import reject

FIELDS = frozenset(("schema", "kind", "status", "root_identity_sha256", "ledger_sha256",
                    "root_sha256", "member_count", "member_total_bytes", "limits",
                    "member_order_sha256", "admitted", "cutover_ready", "receipt_sha256"))
MEMBER_KEYS = ("path", "parent_path", "revision", "blob_sha1", "sha256", "bytes")


def order_digest(document: dict[str, object]) -> str:
    rows = [{key: row[key] for key in MEMBER_KEYS} for row in document["members"]]
    return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def marker(document: dict[str, object], root_digest: str, root_sha256: str,
           limits: dict[str, int]) -> dict[str, object]:
    value: dict[str, object] = {
        "schema": 1, "kind": "canonical-upstream-suite-cache-receipt",
        "status": "external-cache-verified-unadmitted", "root_identity_sha256": root_digest,
        "ledger_sha256": document["ledger_sha256"], "root_sha256": root_sha256,
        "member_count": document["member_count"], "member_total_bytes": document["member_total_bytes"],
        "limits": dict(limits), "member_order_sha256": order_digest(document),
        "admitted": False, "cutover_ready": False, "receipt_sha256": "0" * 64,
    }
    body = {key: item for key, item in value.items() if key != "receipt_sha256"}
    value["receipt_sha256"] = hashlib.sha256(
        json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    return value


def name(document: dict[str, object]) -> str:
    return "receipt-" + str(document["ledger_sha256"]) + ".json"


def payload(value: dict[str, object]) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def object_pairs(pairs: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in pairs:
        if key in value:
            reject("cache receipt has a duplicate JSON key")
        value[key] = item
    return value


def parse(payload: bytes) -> dict[str, object]:
    try:
        value = json.loads(payload.decode("utf-8"), object_pairs_hook=object_pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"cache receipt cannot be parsed: {error}")
    if not isinstance(value, dict) or set(value) != FIELDS:
        reject("cache receipt has an unexpected schema")
    claimed = value.get("receipt_sha256")
    body = {key: item for key, item in value.items() if key != "receipt_sha256"}
    actual = hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    if not isinstance(claimed, str) or claimed != actual:
        reject("cache receipt self-hash is invalid")
    return value
