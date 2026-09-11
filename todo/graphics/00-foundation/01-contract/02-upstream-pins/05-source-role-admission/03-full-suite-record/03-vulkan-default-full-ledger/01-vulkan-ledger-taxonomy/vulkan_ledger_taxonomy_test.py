#!/usr/bin/env python3
"""Hostile checks for the exact VCTS default-ledger observation."""

import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import vulkan_ledger_taxonomy as target


def copied(source: Path, mutate):
    value = json.loads(source.read_text(encoding="utf-8"))
    mutate(value)
    temporary = tempfile.TemporaryDirectory()
    path = Path(temporary.name, source.name)
    path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")
    return temporary, path


class VulkanLedgerTaxonomyTests(unittest.TestCase):
    def test_exact_unfiltered_vulkan_root_has_98_non_core_members_and_no_claims(self):
        value = target.build()
        target.validate_record(value)
        self.assertEqual((value["authority"], value["cts_executions"]), ("WebBoxVM", 0))
        self.assertTrue(all(not claim for claim in value["claims"].values()))
        self.assertEqual((value["source_root"]["authority"], value["source_root"]["producer"]),
                         ("Khronos", "Khronos"))
        self.assertEqual(value["bridge"]["selector_scope"], "khronos-default-mustpass-broader-than-vulkan-1.4-core")
        self.assertEqual(value["taxonomy"]["category_counts"], target.CATEGORY_COUNTS)
        self.assertEqual((value["ledger"]["member_count"], value["ledger"]["member_total_bytes"]),
                         (98, 434669348))
        self.assertEqual(sum(item["bytes"] > 8 * 1024 * 1024 for item in value["ledger"]["members"]), 14)
        self.assertEqual(max(item["bytes"] for item in value["ledger"]["members"]), 61932251)

    def test_rejects_missing_duplicate_reordered_mixed_or_resealed_ledger(self):
        def seal(value):
            value["member_count"] = len(value["members"])
            value["member_total_bytes"] = sum(item["bytes"] for item in value["members"])
            value["ledger_sha256"] = target.ledger.digest(value)
        def altered(value):
            value["members"][0]["sha256"] = "f" * 64
            seal(value)
        def tailed(value):
            item = copy.deepcopy(value["members"][0])
            item.update(path="external/vulkancts/mustpass/main/f02531-tail.txt", parent_path=value["members"][0]["path"],
                        blob_sha1="a" * 40, sha256="b" * 64, bytes=1)
            value["members"].append(item); seal(value)
        def missing(value):
            value["members"].pop(); seal(value)
        def duplicate(value):
            value["members"].append(copy.deepcopy(value["members"][0])); seal(value)
        def reordered(value):
            value["members"].reverse(); seal(value)
        def mixed_revision(value):
            value["members"][0]["revision"] = "a" * 40; seal(value)
        for mutate in (altered, tailed, missing, duplicate, reordered, mixed_revision):
            temporary, path = copied(target.LEDGER, mutate)
            with temporary, self.subTest(mutate=mutate), self.assertRaises(target.VulkanLedgerError):
                target.build(ledger_path=path)

    def test_rejects_resealed_taxonomy_or_forged_local_record_types_and_claims(self):
        def altered_taxonomy(value):
            value["fallback"]["reason"] = "forged"
            value["taxonomy_sha256"] = target.taxonomy.digest(value)
        temporary, path = copied(target.TAXONOMY_JSON, altered_taxonomy)
        with temporary, self.assertRaises(target.VulkanLedgerError):
            target.build(taxonomy_path=path)
        value = target.build()
        mutations = [
            lambda item: item["claims"].update(conformance=True),
            lambda item: item.update(cts_executions=1),
            lambda item: item["ledger"].update(member_count=True),
            lambda item: item["ledger"]["members"].reverse(),
            lambda item: item["taxonomy"]["category_counts"].update(core=1),
            lambda item: item["states"].update(satisfies_vulkan_14_core_manifest=True),
        ]
        for mutate in mutations:
            forged = copy.deepcopy(value); mutate(forged)
            with self.subTest(mutate=mutate), self.assertRaises(target.VulkanLedgerError):
                target.validate_record(forged)

    def test_rejects_a_raw_ledger_snapshot_changed_after_validation(self):
        original = target.ledger.document(target.LEDGER)
        changed = copy.deepcopy(original)
        changed["members"][0]["bytes"] = 1
        with patch.object(target.ledger, "document", side_effect=(original, changed)):
            with self.assertRaises(target.VulkanLedgerError):
                target.build()


if __name__ == "__main__":
    unittest.main()
