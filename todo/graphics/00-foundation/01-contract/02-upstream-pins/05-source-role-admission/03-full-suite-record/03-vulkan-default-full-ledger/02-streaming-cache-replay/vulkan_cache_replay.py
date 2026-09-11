#!/usr/bin/env python3
"""Replay the exact VCTS ledger from a verified external streaming cache."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "01-vulkan-ledger-taxonomy"))
import vulkan_ledger_taxonomy as ledger_map  # noqa: E402

CACHE = HERE.parents[3] / "04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/05-vulkan-source-contract-v2/03-external-closure-cache/01-cache-contract"


def load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load V2 cache contract: {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[name] = value
    spec.loader.exec_module(value)
    return value


cache = load("f02532_cache", CACHE / "vcts_cache_contract.py")


class ReplayError(ValueError):
    """The external cache does not replay the exact unfiltered VCTS closure."""


def reject(message: str) -> None:
    raise ReplayError(message)


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


def marker_document(observation: dict[str, object]) -> dict[str, object]:
    ledger = observation["ledger"]
    assert isinstance(ledger, dict)
    rows = [{key: item[key] for key in cache.receipt.MEMBER_KEYS} for item in ledger["members"]]
    return {"ledger_sha256": ledger["ledger_sha256"], "member_count": ledger["member_count"],
            "member_total_bytes": ledger["member_total_bytes"], "members": rows}


def expected_marker(observation: dict[str, object]) -> dict[str, object]:
    document = marker_document(observation)
    ledger = observation["ledger"]
    source = observation["source_root"]
    assert isinstance(ledger, dict) and isinstance(source, dict)
    return cache.receipt.marker(document, str(ledger["identity_sha256"]), str(source["sha256"]), cache.ledger.LIMITS)


def check_marker(actual: object, observation: dict[str, object]) -> None:
    if not alike(actual, expected_marker(observation)):
        reject("external cache marker differs from the exact type-safe closure receipt")


def checked_inputs(observation: dict[str, object]):
    try:
        root, checked, document = cache.inputs(ledger_map.IDENTITY, ledger_map.LEDGER)
    except cache.io.CacheError as error:
        reject(str(error))
    expected = observation["ledger"]
    assert isinstance(expected, dict)
    if (type(checked.member_count) is not int or type(checked.total_bytes) is not int
            or (root.digest, checked.digest, checked.member_count, checked.total_bytes) !=
            (expected["identity_sha256"], expected["ledger_sha256"], expected["member_count"], expected["member_total_bytes"])):
        reject("V2 cache inputs do not bind the exact local Vulkan ledger observation")
    return root, checked, document


def verify_closed_cache(external_root: Path, repository: Path, root, checked, document: dict[str, object],
                        observation: dict[str, object]) -> None:
    try:
        with cache.io.closure(external_root, repository, root.digest, checked.digest, False, False) as fd:
            cache.verify_fd(fd, root, document)
            actual = cache.receipt.parse(cache.io.read(fd, cache.receipt.name(document), 16 * 1024))
    except cache.io.CacheError as error:
        reject(str(error))
    check_marker(actual, observation)


def replay(external_root: Path) -> dict[str, object]:
    try:
        observation = ledger_map.build()
        repository = cache.repository_root(HERE)
        root, checked, document = checked_inputs(observation)
        verify_closed_cache(external_root, repository, root, checked, document, observation)
    except (ledger_map.VulkanLedgerError, cache.io.CacheError) as error:
        reject(str(error))
    body = {
        "schema": 1, "kind": "webboxvm-engineering-map", "authority": "WebBoxVM", "producer": "WebBoxVM",
        "claims": dict(ledger_map.NO_CLAIMS), "cts_executions": 0, "source_root": observation["source_root"],
        "ledger_sha256": checked.digest, "member_count": checked.member_count, "member_total_bytes": checked.total_bytes,
        "cache": {"mode": "offline-replay", "stream_chunk_bytes": cache.io.CHUNK,
                  "member_limit_bytes": cache.ledger.LIMITS["max_member_bytes"], "raw_member_policy": "unfiltered"},
        "states": dict(observation["states"]),
    }
    return {**body, "receipt_sha256": hashlib.sha256(canonical(body)).hexdigest()}


def validate_receipt(value: object, external_root: Path) -> None:
    if not alike(value, replay(external_root)):
        reject("Vulkan cache replay receipt differs from exact external-cache evidence")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(replay(args.cache_root), sort_keys=True))
    except ReplayError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
