#!/usr/bin/env python3
"""Hermetic positive and hostile tests for the V2 canonical-suite schema."""

from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import canonical_suite_identity as identity
import canonical_suite_ledger as ledger


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")


def seal_identity(value: dict[str, object]) -> None:
    value["identity_sha256"] = identity.digest(value)


def seal_ledger(value: dict[str, object]) -> None:
    members = value["members"]
    assert isinstance(members, list)
    value["member_count"] = len(members)
    value["member_total_bytes"] = sum(item["bytes"] for item in members if isinstance(item, dict))
    value["ledger_sha256"] = ledger.digest(value)


class CanonicalSuiteContractTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.directory = Path(self.temp.name)
        self.identity_path = self.directory / "identity.json"
        self.ledger_path = self.directory / "ledger.json"
        self.root = json.loads((HERE / "vcts_root_identity.json").read_text(encoding="utf-8"))
        write_json(self.identity_path, self.root)
        self.checked_root = identity.validate(self.identity_path)
        self.good = self.valid_ledger()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def valid_ledger(self) -> dict[str, object]:
        paths = self.checked_root.direct_members + (
            "external/vulkancts/mustpass/main/vk-default/recursive/a.txt",
            "external/vulkancts/mustpass/main/vk-default/recursive/b.txt",
        )
        members = [{"path": path, "parent_path": None if index < 98 else self.checked_root.direct_members[0],
                    "revision": self.checked_root.peeled_commit, "blob_sha1": f"{index + 1:040x}",
                    "sha256": f"{index + 1:064x}", "bytes": 100 + index}
                   for index, path in enumerate(paths)]
        value: dict[str, object] = {"schema": 1, "kind": "canonical-upstream-suite",
                                    "root_identity_sha256": self.checked_root.digest, "limits": ledger.LIMITS,
                                    "member_count": len(members),
                                    "member_total_bytes": sum(item["bytes"] for item in members),
                                    "ledger_sha256": "0" * 64, "members": members}
        seal_ledger(value)
        return value

    def reject_identity(self, value: dict[str, object]) -> None:
        seal_identity(value)
        write_json(self.identity_path, value)
        with self.assertRaises(identity.IdentityError):
            identity.validate(self.identity_path)

    def reject_ledger(self, value: dict[str, object]) -> None:
        seal_ledger(value)
        write_json(self.ledger_path, value)
        with self.assertRaises(ledger.LedgerError):
            ledger.validate(self.ledger_path, self.identity_path)

    def test_pinned_identity_and_ordered_recursive_ledger(self) -> None:
        write_json(self.ledger_path, self.good)
        checked = ledger.validate(self.ledger_path, self.identity_path)
        self.assertEqual((checked.member_count, checked.total_bytes), (100, 14950))
        self.assertEqual(self.checked_root.direct_member_count, 98)
        transcript = json.loads((HERE.parents[4] / "03-vulkan-input-audit/mustpass_references.json").read_text())
        self.assertEqual(self.checked_root.direct_members, tuple("external/vulkancts/mustpass/main/" + path
                                                                 for path in transcript["direct_references"]))

    def test_identity_rejects_substitutions_filtering_and_duplicate_keys(self) -> None:
        cases = (
            ("tag", lambda item: item.__setitem__("tag_object_sha1", "1" * 40)),
            ("root-url", lambda item: item["root_input"].__setitem__("immutable_url", "https://example.invalid/root")),
            ("filter", lambda item: item.__setitem__("local_filtering", "permitted")),
            ("boolean-schema", lambda item: item.__setitem__("schema", True)),
        )
        for label, change in cases:
            with self.subTest(label=label):
                candidate = copy.deepcopy(self.root)
                change(candidate)
                self.reject_identity(candidate)
        self.identity_path.write_text('{"schema":1,"schema":1}', encoding="utf-8")
        with self.assertRaises(identity.IdentityError):
            identity.validate(self.identity_path)

    def test_ledger_rejects_root_only_paths_duplicates_cycles_and_reordering(self) -> None:
        cases = (
            ("root-only", lambda item: item.__setitem__("members", item["members"][:1])),
            ("unsafe-path", lambda item: item["members"][0].__setitem__("path", "../escape#.txt")),
            ("duplicate", lambda item: (item["members"][1].__setitem__("path", item["members"][0]["path"]),
                                        item["members"][1].__setitem__("parent_path", None))),
            ("cycle", lambda item: item["members"][0].__setitem__("parent_path", item["members"][1]["path"])),
            ("reordered", lambda item: item["members"].__setitem__(slice(0, 2), reversed(item["members"][:2]))),
            ("recursive-reordered", lambda item: item["members"].__setitem__(slice(-2, None), reversed(item["members"][-2:]))),
            ("root-digest", lambda item: item.__setitem__("root_identity_sha256", "f" * 64)),
        )
        for label, change in cases:
            with self.subTest(label=label):
                candidate = copy.deepcopy(self.good)
                change(candidate)
                self.reject_ledger(candidate)
        self.ledger_path.write_text('{"schema":1,"schema":1}', encoding="utf-8")
        with self.assertRaises(ledger.LedgerError):
            ledger.validate(self.ledger_path, self.identity_path)

    def test_ledger_rejects_mixed_commit_hashes_limits_and_stale_digest(self) -> None:
        cases = (
            ("mixed-commit", lambda item: item["members"][0].__setitem__("revision", "e" * 40)),
            ("bad-hash", lambda item: item["members"][0].__setitem__("sha256", "z" * 64)),
            ("zero-hash", lambda item: item["members"][0].__setitem__("blob_sha1", "0" * 40)),
            ("member-limit", lambda item: item["members"][0].__setitem__("bytes", 64 * 1024 * 1024 + 1)),
            ("aggregate-limit", lambda item: [member.__setitem__("bytes", 6 * 1024 * 1024) for member in item["members"]]),
            ("float-limit", lambda item: item["limits"].__setitem__("max_members", 128.0)),
        )
        for label, change in cases:
            with self.subTest(label=label):
                candidate = copy.deepcopy(self.good)
                change(candidate)
                self.reject_ledger(candidate)
        stale = copy.deepcopy(self.good)
        stale["ledger_sha256"] = "0" * 64
        write_json(self.ledger_path, stale)
        with self.assertRaises(ledger.LedgerError):
            ledger.validate(self.ledger_path, self.identity_path)


if __name__ == "__main__":
    unittest.main(verbosity=2)
