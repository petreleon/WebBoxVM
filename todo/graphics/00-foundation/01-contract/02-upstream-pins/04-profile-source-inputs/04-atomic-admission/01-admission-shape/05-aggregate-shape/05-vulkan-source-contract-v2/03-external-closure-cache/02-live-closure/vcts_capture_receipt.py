"""Build and validate a self-hashed, explicitly unadmitted VCTS capture receipt."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))
import vcts_capture_input as source

CaptureReceiptError = source.CaptureReceiptError

FIELDS = frozenset(("schema", "kind", "status", "root_identity_sha256", "tree_plan_sha256",
                    "root_git_blob_sha1", "root_sha256", "root_bytes", "ledger_sha256", "member_count",
                    "member_total_bytes", "raw_streams", "cache_receipt_sha256", "offline_verification",
                    "admitted", "cutover_ready", "satisfies_vulkan_14_core_manifest", "receipt_sha256"))


def pairs(items: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in items:
        if key in value:
            source.reject("capture receipt has a duplicate JSON key")
        value[key] = item
    return value


def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "receipt_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode("utf-8")).hexdigest()


def document(path: Path) -> dict[str, object]:
    try:
        raw = path.read_bytes()
        if len(raw) > source.MAX_RECEIPT_BYTES:
            source.reject("capture receipt exceeds its JSON byte limit")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except source.CaptureReceiptError:
        raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        source.reject(f"capture receipt cannot be read: {error}")
    if not isinstance(value, dict) or set(value) != FIELDS:
        source.reject("capture receipt has an unexpected schema")
    return value


def build(identity_path: Path, plan_path: Path, ledger_path: Path, cache_receipt_path: Path,
          captured_root: object, stream_stats: object, offline_verification: object) -> dict[str, object]:
    root, plan, closure, ledger_document = source.source_inputs(identity_path, plan_path, ledger_path)
    cache_digest = source.marker(cache_receipt_path, root, ledger_document)
    root_input = source.root_capture(captured_root, plan)
    value: dict[str, object] = {
        "schema": 1, "kind": "vcts-live-closure-capture-receipt",
        "status": "external-cache-offline-verified-unadmitted", "root_identity_sha256": root.digest,
        "tree_plan_sha256": plan.digest, "root_git_blob_sha1": root_input["git_blob_sha1"],
        "root_sha256": root_input["sha256"], "root_bytes": root_input["bytes"],
        "ledger_sha256": closure.digest, "member_count": closure.member_count,
        "member_total_bytes": closure.total_bytes, "raw_streams": source.raw_streams(stream_stats, plan, closure),
        "cache_receipt_sha256": cache_digest,
        "offline_verification": source.offline(offline_verification, closure, cache_digest),
        "admitted": False, "cutover_ready": False, "satisfies_vulkan_14_core_manifest": False,
        "receipt_sha256": "0" * 64,
    }
    value["receipt_sha256"] = digest(value)
    return value


def validate(path: Path, identity_path: Path, plan_path: Path, ledger_path: Path,
             cache_receipt_path: Path) -> dict[str, object]:
    value = document(path)
    if not isinstance(value["receipt_sha256"], str) or value["receipt_sha256"] != digest(value):
        source.reject("capture receipt self-hash is invalid")
    expected = build(identity_path, plan_path, ledger_path, cache_receipt_path,
                     {"sha256": value["root_sha256"], "git_blob_sha1": value["root_git_blob_sha1"],
                      "bytes": value["root_bytes"]}, value["raw_streams"], value["offline_verification"])
    if not source.alike(value, expected):
        source.reject("capture receipt is stale, malformed, or claims admission")
    return value
