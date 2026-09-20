"""Bounded facts for one fresh VCTS capture and its no-claim receipt."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path
from urllib.request import build_opener

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "02-streaming-cache-replay"))
import vulkan_cache_replay as replay  # noqa: E402
import vulkan_ledger_taxonomy as ledger_map  # noqa: E402

LIVE = HERE.parents[3] / "04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/05-vulkan-source-contract-v2/03-external-closure-cache/02-live-closure"


def load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load V2 live stage: {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[name] = value
    spec.loader.exec_module(value)
    return value


stage = load("f02533_stage", LIVE / "vcts_live_stage.py")


class FullReceiptError(ValueError):
    """The full VCTS capture is partial, reused, or falsely represented."""


def reject(message: str) -> None:
    raise FullReceiptError(message)


class CountingOpener:
    """Allow each expected immutable raw URL exactly once, in plan order."""
    def __init__(self, urls: tuple[str, ...], delegate=None):
        self.urls, self.delegate, self.index = urls, delegate or build_opener(stage.transport.DenyRedirect()), 0

    def open(self, request, timeout):
        url = getattr(request, "full_url", None)
        if self.index >= len(self.urls) or url != self.urls[self.index]:
            reject("fresh capture raw request is missing, repeated, or outside the pinned plan")
        self.index += 1
        return self.delegate.open(request, timeout=timeout)

    def complete(self) -> None:
        if self.index != len(self.urls):
            reject("fresh capture did not request every pinned raw URL exactly once")


def required_space(tree: dict[str, object]) -> int:
    members, total = tree.get("members"), tree.get("member_total_bytes")
    if not isinstance(members, list) or type(total) is not int:
        reject("pinned VCTS tree plan cannot establish capture space")
    sizes = [row.get("bytes") for row in members if isinstance(row, dict)]
    if len(sizes) != len(members) or any(type(size) is not int or size < 1 for size in sizes):
        reject("pinned VCTS tree plan has invalid member sizes")
    root = stage.plan.identity.ROOT["bytes"]
    return 2 * (root + total) + max(root, *sizes) + 8 * 1024 * 1024


def cache_facts(checked: object, observed: dict[str, object]) -> None:
    ledger = observed["ledger"]
    assert isinstance(ledger, dict)
    digest = checked.get("receipt_sha256") if isinstance(checked, dict) else None
    if (not isinstance(checked, dict) or not isinstance(digest, str) or len(digest) != 64 or set(digest) - set("0123456789abcdef")
            or type(checked.get("cts_executions")) is not int or checked.get("cts_executions") != 0
            or not replay.alike(checked.get("claims"), ledger_map.NO_CLAIMS) or not replay.alike(checked.get("source_root"), observed["source_root"])
            or not replay.alike(checked.get("states"), observed["states"])
            or (checked.get("ledger_sha256"), checked.get("member_count"), checked.get("member_total_bytes")) !=
            (ledger["ledger_sha256"], ledger["member_count"], ledger["member_total_bytes"])):
        reject("offline cache replay differs from the exact local Vulkan observation")


def build_from(observed: dict[str, object], checked: object, network: object, disk: object) -> dict[str, object]:
    cache_facts(checked, observed)
    ledger, root = observed["ledger"], observed["source_root"]
    assert isinstance(ledger, dict) and isinstance(root, dict)
    streams, total = ledger["member_count"] + 1, ledger["member_total_bytes"]
    expected_network = {"root_streams": 1, "member_streams": ledger["member_count"], "total_streams": streams,
                        "raw_https_requests": streams, "sha256_verified_streams": streams,
                        "git_blob_sha1_verified_streams": streams, "total_bytes": root["bytes"] + total}
    need = required_space({"members": ledger["members"], "member_total_bytes": total})
    if (not replay.alike(network, expected_network) or not isinstance(disk, dict) or set(disk) != {"required_free_bytes", "available_free_bytes"}
            or type(disk["required_free_bytes"]) is not int or type(disk["available_free_bytes"]) is not int
            or disk["required_free_bytes"] != need or disk["available_free_bytes"] < need):
        reject("fresh capture network or disk observations are incomplete")
    oversize = [{key: row[key] for key in ("path", "bytes", "sha256")} for row in ledger["members"]
                if row["bytes"] > 8 * 1024 * 1024]
    if (len(oversize), max((row["bytes"] for row in oversize), default=0)) != (14, 61932251):
        reject("fresh receipt excluded or changed an oversize released member")
    body = {
        "schema": 1, "kind": "webboxvm-vulkan-full-suite-receipt", "authority": "WebBoxVM", "producer": "WebBoxVM",
        "claims": dict(ledger_map.NO_CLAIMS), "cts_executions": 0, "source_root": root,
        "ledger": {"identity_sha256": ledger["identity_sha256"], "ledger_sha256": ledger["ledger_sha256"],
                   "member_count": ledger["member_count"], "member_total_bytes": total},
        "capture": {"status": "fresh-99-raw-stream-capture-and-offline-replay", "failure_or_skip": {"status": "none"}, "network": network,
                    "cache": {"fresh": True, "reused": False, "offline_raw_https_requests": 0,
                              "replay_receipt_sha256": checked.get("receipt_sha256")}, "disk": disk},
        "oversize_members": {"threshold_bytes": 8 * 1024 * 1024, "count": len(oversize),
                              "largest_bytes": max(row["bytes"] for row in oversize), "members": oversize},
        "states": dict(observed["states"]),
    }
    return {**body, "receipt_sha256": hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()).hexdigest()}


def validate_from(value: object, observed: dict[str, object], checked: object, network: object, disk: object) -> None:
    if not replay.alike(value, build_from(observed, checked, network, disk)):
        reject("full Vulkan suite receipt differs from fresh capture and offline replay evidence")
