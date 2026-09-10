#!/usr/bin/env python3
"""Hermetic hostile tests for the unpopulated VCTS Git-tree plan contract."""

from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCHEMA = HERE.parent.parent / "02-canonical-suite-schema"
sys.path.insert(0, str(HERE))
import vcts_tree_plan as plan


def write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")


class TreePlanTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.identity_path = SCHEMA / "vcts_root_identity.json"
        self.output = Path(self.temp.name) / "tree-plan.json"
        self.root = plan.checked_identity(self.identity_path)
        self.good = self.api_input()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def api_input(self) -> dict[str, object]:
        hashes = iter(f"{number:040x}" for number in range(1, 200))
        commit_tree = next(hashes)
        chain, parent = [], commit_tree
        for name in plan.TREE_NAMES:
            child = next(hashes)
            chain.append({"name": name, "mode": "040000", "parent_tree_sha1": parent, "tree_sha1": child})
            parent = child
        return {"schema": 1, "kind": plan.INPUT_KIND, "tag_ref": plan.identity.EXPECTED["tag_ref"],
                "tag_object_sha1": plan.identity.EXPECTED["tag_object_sha1"], "tag_target_sha1": self.root.peeled_commit,
                "tag_target_type": "commit", "peeled_commit_sha1": self.root.peeled_commit,
                "commit_tree_sha1": commit_tree, "tree_chain": chain,
                "root": {"path": self.root.root_path, "mode": "100644", "blob_sha1": next(hashes),
                         "bytes": plan.identity.ROOT["bytes"]},
                "members": [{"path": path, "mode": "100644", "blob_sha1": next(hashes), "bytes": number + 1}
                            for number, path in enumerate(self.root.direct_members)]}

    def reject_build(self, candidate: dict[str, object]) -> None:
        with self.assertRaises(plan.TreePlanError):
            plan.build(candidate, self.identity_path)

    def test_synthetic_round_trip_binds_all_98_members(self) -> None:
        built = plan.build(self.good, self.identity_path)
        write(self.output, built)
        checked = plan.validate(self.output, self.identity_path)
        self.assertEqual((checked.member_count, checked.total_bytes), (98, sum(range(1, 99))))
        self.assertEqual(tuple(row["path"] for row in built["members"]), self.root.direct_members)
        self.assertEqual(built["plan_sha256"], plan.digest(built))

    def test_builder_rejects_substitution_order_scope_and_limits(self) -> None:
        cases = (
            ("tag", lambda item: item.__setitem__("tag_object_sha1", "f" * 40)),
            ("commit", lambda item: item.__setitem__("peeled_commit_sha1", "e" * 40)),
            ("tag-target", lambda item: item.__setitem__("tag_target_sha1", "e" * 40)),
            ("tree", lambda item: item["tree_chain"][0].__setitem__("name", "escape")),
            ("tree-parent", lambda item: item["tree_chain"][1].__setitem__("parent_tree_sha1", "e" * 40)),
            ("root", lambda item: item["root"].__setitem__("mode", "120000")),
            ("unsafe", lambda item: item["members"][0].__setitem__("path", "../escape.txt")),
            ("reordered", lambda item: item["members"].__setitem__(slice(0, 2), reversed(item["members"][:2]))),
            ("missing", lambda item: item["members"].pop()),
            ("extra", lambda item: item["members"].append(copy.deepcopy(item["members"][0]))),
            ("member-limit", lambda item: item["members"][0].__setitem__("bytes", plan.MAX_MEMBER_BYTES + 1)),
            ("total-limit", lambda item: [row.__setitem__("bytes", 6 * 1024 * 1024) for row in item["members"]]),
        )
        for label, change in cases:
            with self.subTest(label=label):
                candidate = copy.deepcopy(self.good)
                change(candidate)
                self.reject_build(candidate)

    def test_validator_rejects_stale_digest_and_duplicate_json_keys(self) -> None:
        built = plan.build(self.good, self.identity_path)
        built["plan_sha256"] = "0" * 64
        write(self.output, built)
        with self.assertRaises(plan.TreePlanError):
            plan.validate(self.output, self.identity_path)
        built = plan.build(self.good, self.identity_path)
        built["member_total_bytes"] += 1
        write(self.output, built)
        with self.assertRaises(plan.TreePlanError):
            plan.validate(self.output, self.identity_path)
        self.output.write_text('{"schema":1,"schema":1}', encoding="utf-8")
        with self.assertRaises(plan.TreePlanError):
            plan.validate(self.output, self.identity_path)


if __name__ == "__main__":
    unittest.main(verbosity=2)
