#!/usr/bin/env python3
"""Hermetic large-member streaming and hostile replay checks."""

import copy
import hashlib
import json
import tempfile
import unittest
from io import BytesIO
from pathlib import Path
from unittest.mock import patch

import vulkan_cache_replay as replay


def blob(payload: bytes) -> str:
    return hashlib.sha1(f"blob {len(payload)}\0".encode() + payload).hexdigest()


def reseal(value):
    body = {key: item for key, item in value.items() if key != "receipt_sha256"}
    value["receipt_sha256"] = hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    return value


class Response:
    def __init__(self, payload, url):
        self.body, self.url, self.read_sizes = BytesIO(payload), url, []
        self.headers = {"Content-Length": str(len(payload))}
    def close(self): self.body.close()
    def geturl(self): return self.url
    def getcode(self): return 200
    def read(self, size=-1):
        self.read_sizes.append(size)
        return self.body.read(size)


class Opener:
    def __init__(self, rows): self.rows, self.responses = rows, []
    def open(self, request, timeout):
        response = Response(*self.rows[request.full_url]); self.responses.append(response)
        return response


class VulkanCacheReplayTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir="/private/tmp")
        self.root = replay.cache.identity.validate(replay.cache.SCHEMA / "vcts_root_identity.json")
        self.repo, self.cache_root = replay.cache.repository_root(replay.HERE), Path(self.temp.name, "cache")
        names = [item.removeprefix("external/vulkancts/mustpass/main/") for item in self.root.direct_members]
        self.selector = ("\n".join(names) + "\n").encode()
        self.payloads = {path: (b"z" * (8 * 1024 * 1024 + 1) if index == 0 else f"member-{index}".encode())
                         for index, path in enumerate(self.root.direct_members)}
        rows = [{"path": path, "parent_path": None, "revision": self.root.peeled_commit, "blob_sha1": blob(data),
                 "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)} for path, data in self.payloads.items()]
        self.ledger = Path(self.temp.name, "ledger.json")
        self.document = {"schema": 1, "kind": "canonical-upstream-suite", "root_identity_sha256": self.root.digest,
                         "limits": replay.cache.ledger.LIMITS, "member_count": len(rows),
                         "member_total_bytes": sum(item["bytes"] for item in rows), "ledger_sha256": "", "members": rows}
        self.document["ledger_sha256"] = replay.cache.ledger.digest(self.document)
        self.ledger.write_text(json.dumps(self.document), encoding="utf-8")

    def tearDown(self): self.temp.cleanup()

    def closure(self):
        return self.cache_root / "webboxvm-graphics/v2/vulkan-cts-mustpass" / self.root.digest / self.document["ledger_sha256"]

    def observation(self):
        return {"ledger": {"identity_sha256": self.root.digest, "ledger_sha256": self.document["ledger_sha256"],
                            "member_count": self.document["member_count"], "member_total_bytes": self.document["member_total_bytes"],
                            "members": self.document["members"]}, "source_root": {"sha256": replay.cache.identity.ROOT["sha256"]}}

    def opener(self):
        rows = {replay.cache.identity.ROOT["immutable_url"]: (self.selector, replay.cache.identity.ROOT["immutable_url"])}
        rows.update({replay.cache.raw_url(self.root.peeled_commit, path): (data, replay.cache.raw_url(self.root.peeled_commit, path))
                     for path, data in self.payloads.items()})
        return Opener(rows)

    def populate(self):
        return replay.cache.populate(replay.cache.SCHEMA / "vcts_root_identity.json", self.ledger, self.cache_root,
                                     self.repo, 5.0, self.opener())

    def verify(self):
        checked = replay.cache.ledger.validate(self.ledger, replay.cache.SCHEMA / "vcts_root_identity.json")
        replay.verify_closed_cache(self.cache_root, self.repo, self.root, checked, self.document, self.observation())

    def test_streams_above_8mib_and_replays_offline(self):
        opener = self.opener()
        created = replay.cache.populate(replay.cache.SCHEMA / "vcts_root_identity.json", self.ledger, self.cache_root,
                                        self.repo, 5.0, opener)
        closure = replay.cache.io.closure
        with patch.object(replay.cache.io, "closure", wraps=closure) as locked: self.verify()
        self.assertEqual((created.reused, self.document["member_count"]), (False, 98))
        self.assertEqual(locked.call_count, 1)
        self.assertEqual(locked.call_args.args[-2:], (False, False))
        self.assertGreater(max(item["bytes"] for item in self.document["members"]), 8 * 1024 * 1024)
        self.assertTrue(all(size <= replay.cache.io.CHUNK for item in opener.responses for size in item.read_sizes))

    def test_rejects_same_length_member_corruption(self):
        self.populate()
        member = self.closure() / "members" / (self.document["members"][0]["sha256"] + ".source")
        payload = member.read_bytes(); member.write_bytes(bytes((payload[0] ^ 1,)) + payload[1:])
        with self.assertRaises(replay.ReplayError): self.verify()

    def test_rejects_partial_or_symlinked_cache(self):
        with patch.object(replay.cache.io, "publish", side_effect=replay.cache.io.CacheError("forced partial cache")):
            with self.assertRaises(replay.cache.io.CacheError): self.populate()
        with self.assertRaises(replay.ReplayError): self.verify()
        self.populate()
        member = self.closure() / "members" / (self.document["members"][0]["sha256"] + ".source")
        member.unlink(); member.symlink_to("escaped.source")
        with self.assertRaises(replay.ReplayError): self.verify()

    def test_local_receipt_keeps_no_claim_boundary(self):
        with patch.object(replay, "verify_closed_cache") as closed:
            value = replay.replay(Path("/private/tmp/cache")); replay.validate_receipt(value, Path("/private/tmp/cache"))
            closed.assert_called()
            self.assertEqual(value["claims"], replay.ledger_map.NO_CLAIMS)
            self.assertTrue(all(item is False for item in value["claims"].values()))
            self.assertEqual((type(value["cts_executions"]), value["cts_executions"]), (int, 0))
            self.assertEqual((value["source_root"]["authority"], value["source_root"]["claims"]["khronos_selector"]), ("Khronos", True))
            self.assertIs(value["states"]["satisfies_vulkan_14_core_manifest"], False)
            forged = copy.deepcopy(value); forged["claims"]["conformance"] = True
            with self.assertRaises(replay.ReplayError): replay.validate_receipt(forged, Path("/private/tmp/cache"))
        root, checked, document = replay.cache.inputs(replay.ledger_map.IDENTITY, replay.ledger_map.LEDGER)
        wrong = replay.cache.ledger.SuiteLedger(checked.member_count, checked.total_bytes, "0" * 64)
        with patch.object(replay.cache, "inputs", return_value=(root, wrong, document)):
            with self.assertRaises(replay.ReplayError): replay.replay(Path("/private/tmp/cache"))

    def test_closed_cache_rejects_zero_for_boolean_marker(self):
        self.populate()
        marker_path = self.closure() / replay.cache.receipt.name(self.document)
        marker = json.loads(marker_path.read_text(encoding="utf-8")); marker["admitted"] = 0
        marker_path.write_bytes(replay.cache.receipt.payload(reseal(marker)))
        self.assertEqual(replay.cache.receipt.parse(replay.cache.receipt.payload(marker))["admitted"], 0)
        with self.assertRaises(replay.ReplayError): self.verify()


if __name__ == "__main__":
    unittest.main()
