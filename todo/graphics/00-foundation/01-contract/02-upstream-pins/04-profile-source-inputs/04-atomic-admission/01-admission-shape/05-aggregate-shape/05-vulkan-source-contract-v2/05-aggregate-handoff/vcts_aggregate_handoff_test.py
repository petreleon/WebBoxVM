#!/usr/bin/env python3
"""Hermetic positive and hostile tests for the V2 aggregate handoff."""

from __future__ import annotations

import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import vcts_aggregate_handoff as handoff

V1_AUDIT = handoff.V2.parents[3] / "03-vulkan-input-audit/mustpass_references.json"


def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")


def seal_live(value: dict[str, object]) -> None:
    value["receipt_sha256"] = handoff.capture.digest(value)


def seal_ledger(value: dict[str, object]) -> None:
    members = value["members"]
    assert isinstance(members, list)
    value["member_count"] = len(members)
    value["member_total_bytes"] = sum(row["bytes"] for row in members)
    value["ledger_sha256"] = handoff.capture.source.ledger.digest(value)


def seal_cache(value: dict[str, object]) -> None:
    body = {key: item for key, item in value.items() if key != "receipt_sha256"}
    value["receipt_sha256"] = hashlib.sha256(
        json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


class HandoffTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        root = Path(self.temp.name)
        names = ("identity", "plan", "ledger", "cache", "live", "taxonomy", "handoff")
        sources = (handoff.IDENTITY, handoff.PLAN, handoff.LEDGER, handoff.CACHE_RECEIPT,
                   handoff.CAPTURE_RECEIPT, handoff.TAXONOMY_RECEIPT, handoff.HANDOFF)
        self.sources = dict(zip(names, sources))
        self.paths = {name: root / source.name for name, source in zip(names, sources)}
        for source, target in zip(sources, self.paths.values()):
            shutil.copyfile(source, target)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def checked(self):
        return handoff.validate(self.paths["handoff"], self.paths["identity"], self.paths["plan"],
                                self.paths["ledger"], self.paths["cache"], self.paths["live"],
                                self.paths["taxonomy"])

    def changed(self, name: str, edit, seal=None) -> None:
        shutil.copyfile(self.sources[name], self.paths[name])
        value = json.loads(self.paths[name].read_text(encoding="utf-8"))
        edit(value)
        if seal:
            seal(value)
        write(self.paths[name], value)
        with self.assertRaises(handoff.HandoffError):
            self.checked()

    def test_committed_evidence_is_exactly_verified_and_unadmitted(self) -> None:
        value = self.checked()
        self.assertEqual(value, handoff.build(self.paths["identity"], self.paths["plan"], self.paths["ledger"],
                                              self.paths["cache"], self.paths["live"], self.paths["taxonomy"]))
        self.assertEqual((value["member_count"], value["member_total_bytes"]), (98, 434669348))
        self.assertEqual(tuple(value[key] for key in ("admitted", "cutover_ready",
                                                       "satisfies_vulkan_14_core_manifest")), handoff.STATE)

    def test_live_receipt_requires_all_capture_proofs_and_false_admission_flags(self) -> None:
        self.changed("live", lambda value: value.__setitem__("admitted", True), seal_live)
        self.changed("live", lambda value: value["raw_streams"].__setitem__("member_streams", 97), seal_live)
        self.changed("live", lambda value: value["offline_verification"].__setitem__("status", "skipped"), seal_live)

    def test_v1_or_stale_root_cannot_substitute_for_v2_identity(self) -> None:
        with self.assertRaises(handoff.HandoffError):
            handoff.build(V1_AUDIT, self.paths["plan"], self.paths["ledger"], self.paths["cache"],
                          self.paths["live"], self.paths["taxonomy"])
        self.changed("identity", lambda value: value.__setitem__("tag_name", "vulkan-cts-1.4.0"),
                     lambda value: value.__setitem__("identity_sha256", handoff.capture.source.identity.digest(value)))

    def test_partial_reordered_or_mixed_ledger_cannot_handoff(self) -> None:
        self.changed("ledger", lambda value: value["members"].pop(), seal_ledger)
        self.changed("ledger", lambda value: value["members"].reverse(), seal_ledger)
        self.changed("ledger", lambda value: value["members"][0].__setitem__("revision", "f" * 40), seal_ledger)

    def test_cache_receipt_must_bind_the_root_ledger_and_unadmitted_state(self) -> None:
        self.changed("cache", lambda value: value.__setitem__("kind", "v1-cache-receipt"), seal_cache)
        self.changed("cache", lambda value: value.__setitem__("ledger_sha256", "0" * 64), seal_cache)
        self.changed("cache", lambda value: value.__setitem__("admitted", True), seal_cache)

    def test_taxonomy_cannot_reclassify_or_detach_from_the_ledger(self) -> None:
        self.changed("taxonomy", lambda value: value["rules"][0].__setitem__("category", "core"),
                     lambda value: value.__setitem__("taxonomy_sha256", handoff.taxonomy.digest(value)))
        self.changed("taxonomy", lambda value: value.__setitem__("suite_identity_sha256", "0" * 64),
                     lambda value: value.__setitem__("taxonomy_sha256", handoff.taxonomy.digest(value)))

    def test_handoff_schema_rejects_v1_fields_docs_admission_and_tampering(self) -> None:
        self.changed("handoff", lambda value: value.__setitem__("inventory_sha256", "0" * 64),
                     lambda value: value.__setitem__("handoff_sha256", handoff.digest(value)))
        self.changed("handoff", lambda value: value.__setitem__("docs_generated_artifacts", "admitted"),
                     lambda value: value.__setitem__("handoff_sha256", handoff.digest(value)))
        self.changed("handoff", lambda value: value.__setitem__("handoff_sha256", "0" * 64))

    def test_handoff_rejects_an_oversize_json_before_decoding(self) -> None:
        self.paths["handoff"].write_bytes(b" " * (handoff.MAX_HANDOFF_BYTES + 1))
        with self.assertRaisesRegex(handoff.HandoffError, "byte limit"):
            self.checked()


if __name__ == "__main__":
    unittest.main(verbosity=2)
