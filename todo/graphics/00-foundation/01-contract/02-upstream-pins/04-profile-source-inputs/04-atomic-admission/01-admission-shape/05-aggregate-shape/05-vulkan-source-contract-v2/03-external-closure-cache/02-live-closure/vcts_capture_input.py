"""Validate the checked sources and exact evidence supplied to a VCTS capture receipt."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCHEMA = HERE.parent.parent / "02-canonical-suite-schema"
CACHE = HERE.parent / "01-cache-contract"
for directory in (HERE, CACHE):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))
import vcts_tree_plan as tree
import vcts_cache_receipt as cache


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load reviewed module: {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


identity = module("f025_v2_capture_identity", SCHEMA / "canonical_suite_identity.py")
ledger = module("f025_v2_capture_ledger", SCHEMA / "canonical_suite_ledger.py")
MAX_RECEIPT_BYTES = 64 * 1024
ROOT_FIELDS = frozenset(("sha256", "git_blob_sha1", "bytes"))
RAW_FIELDS = frozenset(("root_streams", "member_streams", "total_streams", "raw_https_requests",
                        "sha256_verified_streams", "git_blob_sha1_verified_streams", "total_bytes"))
OFFLINE_FIELDS = frozenset(("status", "verify_calls", "raw_https_requests", "member_count",
                            "member_total_bytes", "cache_receipt_sha256"))


class CaptureReceiptError(ValueError):
    """The claimed live capture is incomplete, stale, or falsely admitted."""


def reject(message: str) -> None:
    raise CaptureReceiptError(message)


def alike(value: object, expected: object) -> bool:
    if type(value) is not type(expected):
        return False
    if isinstance(value, dict):
        return set(value) == set(expected) and all(alike(value[key], expected[key]) for key in value)
    if isinstance(value, list):
        return len(value) == len(expected) and all(alike(left, right) for left, right in zip(value, expected))
    return value == expected


def source_inputs(identity_path: Path, plan_path: Path, ledger_path: Path):
    try:
        root = identity.validate(identity_path)
        plan, closure = tree.validate(plan_path, identity_path), ledger.validate(ledger_path, identity_path)
        plan_document, ledger_document = tree.document(plan_path), ledger.document(ledger_path)
    except (identity.IdentityError, tree.TreePlanError, ledger.LedgerError) as error:
        reject(f"capture source metadata is invalid: {error}")
    try:
        plan_rows = tuple((row["path"], row["blob_sha1"], row["bytes"]) for row in plan_document["members"])
        ledger_rows = tuple((row["path"], row["blob_sha1"], row["bytes"]) for row in ledger_document["members"])
    except (KeyError, TypeError):
        reject("capture source metadata changed after validation")
    if (plan_document.get("plan_sha256") != plan.digest or ledger_document.get("ledger_sha256") != closure.digest
            or plan.member_count != closure.member_count or plan.total_bytes != closure.total_bytes or plan_rows != ledger_rows):
        reject("capture ledger does not bind the immutable tree plan")
    return root, plan, closure, ledger_document


def marker(path: Path, root, ledger_document: dict[str, object]) -> str:
    try:
        raw = path.read_bytes()
        if len(raw) > MAX_RECEIPT_BYTES:
            reject("cache receipt exceeds its JSON byte limit")
        value = cache.parse(raw)
    except CaptureReceiptError:
        raise
    except (OSError, ValueError) as error:
        reject(f"cache receipt is invalid: {error}")
    expected = cache.marker(ledger_document, root.digest, identity.ROOT["sha256"], ledger.LIMITS)
    if not alike(value, expected):
        reject("cache receipt does not bind the exact unadmitted closure")
    return str(value["receipt_sha256"])


def root_capture(value: object, plan) -> dict[str, object]:
    expected = {"sha256": identity.ROOT["sha256"], "git_blob_sha1": plan.root_blob_sha1,
                "bytes": identity.ROOT["bytes"]}
    if not isinstance(value, dict) or set(value) != ROOT_FIELDS or not alike(value, expected):
        reject("captured root digest, Git blob identity, or byte count is invalid")
    return expected


def raw_streams(value: object, plan, closure) -> dict[str, int]:
    streams = plan.member_count + 1
    expected = {"root_streams": 1, "member_streams": plan.member_count, "total_streams": streams,
                "raw_https_requests": streams, "sha256_verified_streams": streams,
                "git_blob_sha1_verified_streams": streams,
                "total_bytes": identity.ROOT["bytes"] + closure.total_bytes}
    if not isinstance(value, dict) or set(value) != RAW_FIELDS or not alike(value, expected):
        reject("raw stream statistics do not prove exactly one verified stream per pinned input")
    return expected


def offline(value: object, closure, cache_digest: str) -> dict[str, object]:
    expected = {"status": "passed", "verify_calls": 1, "raw_https_requests": 0,
                "member_count": closure.member_count, "member_total_bytes": closure.total_bytes,
                "cache_receipt_sha256": cache_digest}
    if not isinstance(value, dict) or set(value) != OFFLINE_FIELDS or not alike(value, expected):
        reject("offline verification does not prove a transport-free cache recheck")
    return expected
