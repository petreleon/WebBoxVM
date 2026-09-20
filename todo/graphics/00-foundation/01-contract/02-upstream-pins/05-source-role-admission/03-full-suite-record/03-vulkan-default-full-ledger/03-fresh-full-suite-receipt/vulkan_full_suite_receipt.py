#!/usr/bin/env python3
"""Capture the pinned VCTS closure once and publish a no-claim local receipt."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
from pathlib import Path

from vulkan_full_suite_contract import (CountingOpener, FullReceiptError, build_from, ledger_map,
                                        reject, replay, required_space, stage, validate_from)

HERE = Path(__file__).resolve().parent
LIVE = HERE.parents[3] / "04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/05-vulkan-source-contract-v2/03-external-closure-cache/02-live-closure"
PLAN, LEDGER = LIVE / "vcts_tree_plan.json", LIVE / "vcts_closure_ledger.json"
IDENTITY = stage.cache.SCHEMA / "vcts_root_identity.json"


def source():
    observed = ledger_map.build()
    try:
        root = stage.plan.checked_identity(IDENTITY)
        tree, closure = stage.plan.validate(PLAN, IDENTITY), stage.cache.ledger.validate(LEDGER, IDENTITY)
        document = stage.plan.document(PLAN)
    except (stage.plan.TreePlanError, stage.cache.ledger.LedgerError, OSError) as error:
        reject(str(error))
    ledger, root_record = observed["ledger"], observed["source_root"]
    assert isinstance(ledger, dict) and isinstance(root_record, dict)
    tree_rows = tuple((row["path"], row["blob_sha1"], row["bytes"]) for row in document["members"])
    ledger_rows = tuple((row["path"], row["blob_sha1"], row["bytes"]) for row in ledger["members"])
    if ((root.digest, closure.digest, tree.member_count, tree.total_bytes, tree_rows) !=
            (ledger["identity_sha256"], ledger["ledger_sha256"], ledger["member_count"], ledger["member_total_bytes"], ledger_rows)
            or (document["root"]["path"], document["root"]["blob_sha1"], document["root"]["bytes"],
                stage.plan.identity.ROOT["sha256"], root.peeled_commit, stage.plan.identity.ROOT["immutable_url"]) !=
            (root.root_path, tree.root_blob_sha1, root_record["bytes"], root_record["sha256"],
             root_record["revision"], root_record["immutable_url"])):
        reject("pinned V2 tree plan or ledger differs from the F02 Vulkan default observation")
    return observed, root, document


def preflight(cache_root: Path, repository: Path, need: int) -> int:
    try:
        fd = stage.fs.absolute(cache_root, repository, True)
        try:
            nonempty = bool(os.listdir(fd))
        finally:
            os.close(fd)
        free = shutil.disk_usage(cache_root).free
    except (stage.fs.CacheError, OSError) as error:
        reject(str(error))
    if nonempty or free < need:
        reject("fresh capture requires an empty external cache root with sufficient free space")
    return free


def planned_urls(root, tree: dict[str, object]) -> tuple[str, ...]:
    members = tree.get("members")
    if not isinstance(members, list) or len(members) != 98:
        reject("pinned V2 tree plan does not have the required full member set")
    urls = (stage.plan.identity.ROOT["immutable_url"], *tuple(
        stage.cache.raw_url(root.peeled_commit, row["path"]) for row in members))
    if len(urls) != 99 or len(set(urls)) != 99:
        reject("pinned V2 tree plan does not have 99 distinct immutable raw URLs")
    return urls


def capture(cache_root: Path, timeout: float = 60.0) -> dict[str, object]:
    observed, root, tree = source()
    repository, need = stage.cache.repository_root(HERE), required_space(tree)
    free = preflight(cache_root, repository, need)
    urls = planned_urls(root, tree)
    counter, captured = CountingOpener(urls), None
    try:
        captured = stage.capture(IDENTITY, PLAN, cache_root, repository, timeout, counter)
        counter.complete()
        with captured.replay() as local:
            populated = stage.cache.populate(IDENTITY, LEDGER, cache_root, repository, timeout, local)
            local.assert_complete()
        if populated.reused is not False:
            reject("fresh capture unexpectedly reused a cache closure")
        checked = replay.replay(cache_root)
        network = {"root_streams": 1, "member_streams": len(tree["members"]), "total_streams": counter.index,
                   "raw_https_requests": counter.index, "sha256_verified_streams": counter.index,
                   "git_blob_sha1_verified_streams": counter.index, "total_bytes": captured.total_bytes}
        value = build_from(observed, checked, network, {"required_free_bytes": need, "available_free_bytes": free})
        retired, captured = captured, None
        retired.discard()
        return value
    except (stage.StageError, replay.ReplayError, OSError) as error:
        reject(str(error))
    finally:
        if captured is not None:
            retired, captured = captured, None
            try:
                retired.discard()
            except stage.StageError as cleanup:
                reject(f"fresh capture cleanup failed: {cleanup}")


def validate(value: object, cache_root: Path) -> None:
    if not isinstance(value, dict) or not isinstance(value.get("capture"), dict):
        reject("full Vulkan suite receipt is malformed")
    observed, _, _ = source()
    checked, capture_record = replay.replay(cache_root), value["capture"]
    validate_from(value, observed, checked, capture_record.get("network"), capture_record.get("disk"))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=60.0)
    args = parser.parse_args()
    try:
        print(json.dumps(capture(args.cache_root, args.timeout), sort_keys=True))
    except FullReceiptError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
