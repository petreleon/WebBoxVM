#!/usr/bin/env python3
"""Hermetic hostile tests for the self-hashed VCTS live-capture receipt."""

from __future__ import annotations

import copy
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCHEMA = HERE.parent.parent / "02-canonical-suite-schema"
sys.path.insert(0, str(HERE))
import vcts_capture_input as source
import vcts_capture_json as output
import vcts_capture_receipt as receipt


class CaptureReceiptTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.directory = Path(self.temp.name)
        self.identity_path = SCHEMA / "vcts_root_identity.json"
        self.root = source.identity.validate(self.identity_path)
        self.plan, self.ledger = self.sources()
        self.plan_path, self.ledger_path = self.directory / "plan.json", self.directory / "ledger.json"
        output.write_json(self.plan_path, self.plan)
        output.write_json(self.ledger_path, self.ledger)
        self.marker_path = self.directory / "cache-receipt.json"
        self.marker = source.cache.marker(self.ledger, self.root.digest, source.identity.ROOT["sha256"],
                                          source.ledger.LIMITS)
        self.marker_path.write_text(json.dumps(self.marker), encoding="utf-8")
        self.receipt_path = self.directory / "capture-receipt.json"
        self.value = receipt.build(self.identity_path, self.plan_path, self.ledger_path, self.marker_path,
                                   self.captured_root(), self.streams(), self.offline())
        output.write_json(self.receipt_path, self.value)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def sources(self) -> tuple[dict[str, object], dict[str, object]]:
        hashes = iter(f"{number:040x}" for number in range(1, 400))
        commit_tree, parent = next(hashes), None
        chain = []
        for name in source.tree.TREE_NAMES:
            child = next(hashes)
            chain.append({"name": name, "mode": "040000", "parent_tree_sha1": parent or commit_tree,
                          "tree_sha1": child})
            parent = child
        rows = [{"path": path, "mode": "100644", "blob_sha1": next(hashes), "bytes": number}
                for number, path in enumerate(self.root.direct_members, 1)]
        api = {"schema": 1, "kind": source.tree.INPUT_KIND, "tag_ref": source.identity.EXPECTED["tag_ref"],
               "tag_object_sha1": source.identity.EXPECTED["tag_object_sha1"],
               "tag_target_sha1": self.root.peeled_commit, "tag_target_type": "commit",
               "peeled_commit_sha1": self.root.peeled_commit, "commit_tree_sha1": commit_tree,
               "tree_chain": chain, "root": {"path": self.root.root_path, "mode": "100644",
               "blob_sha1": next(hashes), "bytes": source.identity.ROOT["bytes"]}, "members": rows}
        plan = source.tree.build(api, self.identity_path)
        members = [{"path": row["path"], "parent_path": None, "revision": self.root.peeled_commit,
                    "blob_sha1": row["blob_sha1"], "sha256": f"{number:064x}", "bytes": row["bytes"]}
                   for number, row in enumerate(rows, 1)]
        ledger = {"schema": 1, "kind": "canonical-upstream-suite", "root_identity_sha256": self.root.digest,
                  "limits": dict(source.ledger.LIMITS), "member_count": len(members),
                  "member_total_bytes": sum(row["bytes"] for row in members), "ledger_sha256": "0" * 64,
                  "members": members}
        ledger["ledger_sha256"] = source.ledger.digest(ledger)
        return plan, ledger

    def captured_root(self) -> dict[str, object]:
        return {"sha256": source.identity.ROOT["sha256"], "git_blob_sha1": self.plan["root"]["blob_sha1"],
                "bytes": source.identity.ROOT["bytes"]}

    def streams(self) -> dict[str, int]:
        total = source.identity.ROOT["bytes"] + self.ledger["member_total_bytes"]
        return {"root_streams": 1, "member_streams": 98, "total_streams": 99, "raw_https_requests": 99,
                "sha256_verified_streams": 99, "git_blob_sha1_verified_streams": 99, "total_bytes": total}

    def offline(self) -> dict[str, object]:
        return {"status": "passed", "verify_calls": 1, "raw_https_requests": 0, "member_count": 98,
                "member_total_bytes": self.ledger["member_total_bytes"],
                "cache_receipt_sha256": self.marker["receipt_sha256"]}

    def write_receipt(self, value: dict[str, object], hash_it: bool = True) -> None:
        if hash_it:
            value["receipt_sha256"] = receipt.digest(value)
        output.write_json(self.receipt_path, value)

    def checked(self) -> dict[str, object]:
        return receipt.validate(self.receipt_path, self.identity_path, self.plan_path, self.ledger_path,
                                self.marker_path)

    def test_round_trip_binds_all_sources_and_compact_members(self) -> None:
        checked = self.checked()
        self.assertEqual((checked["member_count"], checked["raw_streams"]["total_streams"]), (98, 99))
        self.assertFalse(checked["satisfies_vulkan_14_core_manifest"])
        self.assertEqual(output.render(self.plan).count("\n    {"), 98)
        self.assertEqual(json.loads(output.render(self.ledger)), self.ledger)

    def test_rejects_stale_self_hash_admission_and_raw_claims(self) -> None:
        cases = (
            ("self-hash", lambda item: item.__setitem__("receipt_sha256", "0" * 64), False),
            ("admitted", lambda item: item.__setitem__("admitted", True), True),
            ("bool-as-int", lambda item: item.__setitem__("cutover_ready", 0), True),
            ("core", lambda item: item.__setitem__("satisfies_vulkan_14_core_manifest", True), True),
            ("stream", lambda item: item["raw_streams"].__setitem__("raw_https_requests", 98), True),
            ("offline", lambda item: item["offline_verification"].__setitem__("raw_https_requests", 1), True),
        )
        for label, change, rehash in cases:
            with self.subTest(label=label):
                candidate = copy.deepcopy(self.value)
                change(candidate)
                self.write_receipt(candidate, rehash)
                with self.assertRaises(source.CaptureReceiptError):
                    self.checked()

    def test_rejects_duplicate_json_and_plan_ledger_substitution(self) -> None:
        self.receipt_path.write_text('{"schema":1,"schema":1}', encoding="utf-8")
        with self.assertRaises(source.CaptureReceiptError):
            self.checked()

    def test_rejects_a_self_hashed_but_admitted_cache_marker(self) -> None:
        bad_marker = copy.deepcopy(self.marker)
        bad_marker["admitted"] = True
        body = {key: value for key, value in bad_marker.items() if key != "receipt_sha256"}
        bad_marker["receipt_sha256"] = hashlib.sha256(
            json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
        self.marker_path.write_text(json.dumps(bad_marker), encoding="utf-8")
        with self.assertRaises(source.CaptureReceiptError):
            self.checked()
        bad_plan = copy.deepcopy(self.plan)
        bad_plan["members"][0]["blob_sha1"] = "f" * 40
        bad_plan["plan_sha256"] = source.tree.digest(bad_plan)
        output.write_json(self.plan_path, bad_plan)
        self.write_receipt(copy.deepcopy(self.value))
        with self.assertRaises(source.CaptureReceiptError):
            self.checked()


if __name__ == "__main__":
    unittest.main(verbosity=2)
