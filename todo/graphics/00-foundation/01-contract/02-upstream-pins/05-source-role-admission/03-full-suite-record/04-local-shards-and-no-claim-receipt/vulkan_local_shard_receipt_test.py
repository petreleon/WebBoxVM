#!/usr/bin/env python3
"""Hostile catalog, artifact, and no-claim receipt coverage for F02.5.3.4."""

import copy
import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import vulkan_local_shard_receipt as receipt
import vulkan_local_shards as local
from source_role_contract import validate_catalog


def digest(data):
    return hashlib.sha256(data).hexdigest()


def root_record():
    return copy.deepcopy(local.ledger_map.build()["source_root"])


def cache_record(root):
    ledger = local.ledger_map.build()["ledger"]
    return {"claims": dict(local.NO_CLAIMS), "cts_executions": 0, "source_root": root,
            "ledger_sha256": ledger["ledger_sha256"], "member_count": ledger["member_count"],
            "member_total_bytes": ledger["member_total_bytes"], "receipt_sha256": "c" * 64}


def write(root, key, data):
    path = root / key
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return path


class VulkanLocalShardTests(unittest.TestCase):
    def test_exact_real_api_catalog_has_five_bounded_ordered_shards(self):
        root, value = root_record(), local.catalog(root_record())
        records, pieces = value["records"], value["records"][2:]
        self.assertEqual(validate_catalog(value), tuple(item["id"] for item in records))
        self.assertEqual((records[0]["id"], records[1]["id"], records[1]["bytes"], records[1]["sha256"]),
                         ("vulkan-cts-default", local.API_ID, 40296059, local.API[1]))
        self.assertEqual([(item["shard"]["index"], item["shard"]["offset"], item["bytes"], item["sha256"])
                          for item in pieces], [(0, 0, 8388608, local.SHARDS[0][2]),
                                                  (1, 8388608, 8388608, local.SHARDS[1][2]),
                                                  (2, 16777216, 8388608, local.SHARDS[2][2]),
                                                  (3, 25165824, 8388608, local.SHARDS[3][2]),
                                                  (4, 33554432, 6741627, local.SHARDS[4][2])])
        self.assertEqual(sum(item["bytes"] for item in pieces), records[1]["bytes"])
        self.assertTrue(all(item["bytes"] <= local.builder.MAX_LOCAL_BYTES for item in pieces))
        self.assertEqual(root["id"], records[0]["id"])

    def test_rejects_missing_reordered_oversize_and_root_mismatched_catalogs(self):
        root, value = root_record(), local.catalog(root_record())
        cases = []
        missing = copy.deepcopy(value); missing["records"].pop(); cases.append(missing)
        reordered = copy.deepcopy(value); reordered["records"][2:4] = reversed(reordered["records"][2:4]); cases.append(reordered)
        oversize = copy.deepcopy(value); oversize["records"][2]["bytes"] += 1; cases.append(oversize)
        mismatch = copy.deepcopy(value); mismatch["records"][1]["suite_root_id"] = "wrong-root"; cases.append(mismatch)
        for candidate in cases:
            with self.subTest(candidate=candidate), self.assertRaises(local.ShardError):
                local.verify(candidate, Path("/private/tmp/not-read"), root)

    def test_streamed_artifacts_reject_altered_source_shard_and_builder(self):
        data, source_digest = b"abcdefghij", digest(b"abcdefghij")
        pieces = ((0, 5, digest(data[:5])), (5, 5, digest(data[5:])) )
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory, \
             patch.object(local, "API", (len(data), source_digest)), patch.object(local, "SHARDS", pieces):
            root = root_record(); root["bytes"], root["sha256"] = 4, digest(b"root")
            artifact_root, value = Path(directory), local.catalog(root)
            write(artifact_root, value["records"][0]["artifact"], b"root")
            source = write(artifact_root, value["records"][1]["artifact"], data)
            write(artifact_root, value["records"][2]["builder"]["artifact"], Path(local.builder.__file__).read_bytes())
            for item in value["records"][2:]:
                span = item["shard"]
                local.builder.write_shard(source, artifact_root / item["artifact"], span["offset"], item["bytes"],
                                          len(data), source_digest)
            self.assertEqual(local.verify(value, artifact_root, root), value)
            (artifact_root / value["records"][2]["artifact"]).write_bytes(b"altered")
            with self.assertRaises(local.ShardError):
                local.verify(value, artifact_root, root)
            (artifact_root / value["records"][2]["artifact"]).unlink()
            local.builder.write_shard(source, artifact_root / value["records"][2]["artifact"], 0, 5, len(data), source_digest)
            (artifact_root / value["records"][2]["builder"]["artifact"]).write_bytes(b"altered-builder")
            with self.assertRaises(local.ShardError):
                local.verify(value, artifact_root, root)
            write(artifact_root, value["records"][2]["builder"]["artifact"], Path(local.builder.__file__).read_bytes())
            source.write_bytes(b"altered-api")
            with self.assertRaises(local.ShardError):
                local.verify(value, artifact_root, root)

    def test_cache_bridge_fails_before_any_local_builder_invocation(self):
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory, \
             patch.object(local, "bridge", side_effect=local.ShardError("bad cache")), \
             patch.object(local.builder, "build") as build:
            with self.assertRaises(local.ShardError):
                local.build(Path("/private/tmp/cache"), Path(directory, "artifacts"))
            build.assert_not_called()


class VulkanLocalShardReceiptTests(unittest.TestCase):
    def test_self_hashed_receipt_has_no_claims_and_rejects_forgery(self):
        root, catalog = root_record(), local.catalog(root_record())
        value = receipt.build_from(catalog, cache_record(root))
        self.assertEqual((value["cts_executions"], value["mode"], len(value["shards"])), (0, "byte-preserving-shard", 5))
        self.assertTrue(all(flag is False for flag in value["claims"].values()))
        receipt.validate_from(value, catalog, cache_record(root))
        changes = (lambda item: item["claims"].__setitem__("conformance", True),
                   lambda item: item["claims"].__setitem__("api_support", 0),
                   lambda item: item.__setitem__("cts_executions", 1),
                   lambda item: item["source_member"].__setitem__("sha256", "0" * 64),
                   lambda item: item["shards"].reverse(),
                   lambda item: item["builder"].__setitem__("sha256", "0" * 64),
                   lambda item: item.__setitem__("receipt_sha256", "0" * 64))
        for change in changes:
            forged = copy.deepcopy(value); change(forged)
            with self.subTest(change=change), self.assertRaises(receipt.ReceiptError):
                receipt.validate_from(forged, catalog, cache_record(root))


if __name__ == "__main__":
    unittest.main()
