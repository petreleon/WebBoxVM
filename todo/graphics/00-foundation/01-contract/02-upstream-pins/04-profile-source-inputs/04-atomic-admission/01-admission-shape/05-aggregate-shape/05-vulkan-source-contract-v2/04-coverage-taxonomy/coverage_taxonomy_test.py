#!/usr/bin/env python3
"""Hermetic positive and hostile tests for V2 coverage taxonomy and reports."""

from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCHEMA = HERE.parent / "02-canonical-suite-schema"
sys.path[:0] = [str(HERE), str(SCHEMA)]
import canonical_suite_identity as identity
import canonical_suite_ledger as ledger
import coverage_report as report
import coverage_taxonomy as taxonomy

IDENTITY = SCHEMA / "vcts_root_identity.json"
TAXONOMY = HERE / "coverage_taxonomy.json"


def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")


def seal_ledger(value: dict[str, object]) -> None:
    members = value["members"]
    assert isinstance(members, list)
    value["member_count"] = len(members)
    value["member_total_bytes"] = sum(item["bytes"] for item in members if isinstance(item, dict))
    value["ledger_sha256"] = ledger.digest(value)


def seal_report(value: dict[str, object]) -> None:
    value["report_sha256"] = report.digest(value)


class CoverageTaxonomyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.directory = Path(self.temp.name)
        self.ledger_path = self.directory / "ledger.json"
        self.taxonomy_path = self.directory / "taxonomy.json"
        self.report_path = self.directory / "report.json"
        self.root = identity.validate(IDENTITY)
        write(self.taxonomy_path, json.loads(TAXONOMY.read_text(encoding="utf-8")))
        write(self.ledger_path, self.suite())

    def tearDown(self) -> None:
        self.temp.cleanup()

    def suite(self) -> dict[str, object]:
        paths = self.root.direct_members + ("external/vulkancts/mustpass/main/vk-default/recursive/tail.txt",)
        members = [{"path": path, "parent_path": None if index < self.root.direct_member_count else self.root.direct_members[0],
                    "revision": self.root.peeled_commit, "blob_sha1": f"{index + 1:040x}",
                    "sha256": f"{index + 1:064x}", "bytes": index + 1} for index, path in enumerate(paths)]
        value: dict[str, object] = {"schema": 1, "kind": "canonical-upstream-suite",
                                    "root_identity_sha256": self.root.digest, "limits": ledger.LIMITS,
                                    "member_count": 0, "member_total_bytes": 0, "ledger_sha256": "", "members": members}
        seal_ledger(value)
        return value

    def outcomes(self) -> dict[str, object]:
        view = taxonomy.classify(self.taxonomy_path, IDENTITY, self.ledger_path)
        return {row["path"]: {"tests": index + 1, "skips": 0, "failures": 0}
                for index, row in enumerate(view.members)}

    def built(self, mode: str = "complete-suite-diagnostics") -> dict[str, object]:
        return report.build(self.taxonomy_path, IDENTITY, self.ledger_path, mode, self.outcomes())

    def test_pinned_taxonomy_classifies_all_ordered_members(self) -> None:
        checked = taxonomy.validate(self.taxonomy_path, IDENTITY)
        view = taxonomy.classify(self.taxonomy_path, IDENTITY, self.ledger_path)
        self.assertEqual(checked.digest, "752a3ae5ff10ea0d9f9a2638c0d1612fb7af3d376dabad1601f8b9c1e76cb2f7")
        self.assertEqual(tuple(row["path"] for row in view.members), self.root.direct_members +
                         ("external/vulkancts/mustpass/main/vk-default/recursive/tail.txt",))
        self.assertEqual({name: sum(row["category"] == name for row in view.members)
                          for name in taxonomy.CATEGORIES}, {"core": 0, "wsi": 1, "video": 1, "extension": 4, "unknown": 93})
        self.assertTrue(all(row["source_locator"].endswith(row["path"]) for row in view.members))

    def test_core_and_complete_suite_reports_are_diagnostic_only(self) -> None:
        core = self.built("core-readiness")
        complete = self.built()
        write(self.report_path, core)
        self.assertEqual(report.validate(self.report_path, self.taxonomy_path, IDENTITY, self.ledger_path), core)
        self.assertEqual(core["status"], "blocked-no-reviewed-core-members")
        self.assertEqual(core["summary"]["tests"], 4950)
        self.assertEqual(core["summary"]["skips"], 0)
        self.assertEqual(core["summary"]["failures"], 0)
        self.assertEqual(complete["status"], "complete-suite-diagnostics-clean-not-conformance")
        self.assertEqual(complete["coverage_claim"], report.CLAIM)
        empty = {path: {"tests": 0, "skips": 0, "failures": 0} for path in self.outcomes()}
        self.assertEqual(report.build(self.taxonomy_path, IDENTITY, self.ledger_path,
                                      "complete-suite-diagnostics", empty)["status"],
                         "complete-suite-diagnostics-incomplete-no-recorded-member-tests")
        partial = self.outcomes()
        partial[next(iter(partial))]["tests"] = 0
        self.assertEqual(report.build(self.taxonomy_path, IDENTITY, self.ledger_path,
                                      "complete-suite-diagnostics", partial)["status"],
                         "complete-suite-diagnostics-incomplete-no-recorded-member-tests")

    def test_taxonomy_rejects_stale_and_reclassified_rules(self) -> None:
        stale = json.loads(TAXONOMY.read_text(encoding="utf-8"))
        stale["taxonomy_sha256"] = "0" * 64
        write(self.taxonomy_path, stale)
        with self.assertRaisesRegex(taxonomy.TaxonomyError, "stale"):
            taxonomy.validate(self.taxonomy_path, IDENTITY)
        changed = json.loads(TAXONOMY.read_text(encoding="utf-8"))
        changed["rules"][0]["category"] = "core"
        changed["taxonomy_sha256"] = taxonomy.digest(changed)
        write(self.taxonomy_path, changed)
        with self.assertRaisesRegex(taxonomy.TaxonomyError, "reclassified"):
            taxonomy.validate(self.taxonomy_path, IDENTITY)

    def test_reports_reject_missing_duplicate_stale_reclassified_and_filtered_members(self) -> None:
        cases = (
            ("missing", lambda value: value["members"].pop()),
            ("duplicate", lambda value: value["members"].append(copy.deepcopy(value["members"][0]))),
            ("reordered", lambda value: value["members"].reverse()),
            ("blob-hash", lambda value: value["members"][0].__setitem__("blob_sha1", "f" * 40)),
            ("member-hash", lambda value: value["members"][0].__setitem__("member_sha256", "f" * 64)),
            ("stale", lambda value: value.__setitem__("suite_ledger_sha256", "0" * 64)),
            ("reclassified", lambda value: value["members"][0].__setitem__("category", "core")),
        )
        for label, change in cases:
            with self.subTest(label=label):
                value = self.built()
                change(value)
                value["member_count"] = len(value["members"])
                seal_report(value)
                write(self.report_path, value)
                with self.assertRaises(report.ReportError):
                    report.validate(self.report_path, self.taxonomy_path, IDENTITY, self.ledger_path)
        outcomes = self.outcomes()
        outcomes.pop(next(iter(outcomes)))
        with self.assertRaisesRegex(report.ReportError, "silently filter"):
            report.build(self.taxonomy_path, IDENTITY, self.ledger_path, "complete-suite-diagnostics", outcomes)


if __name__ == "__main__":
    unittest.main(verbosity=2)
