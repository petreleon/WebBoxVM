#!/usr/bin/env python3
"""Public integration coverage for the two unadmitted Docs output witnesses."""

from __future__ import annotations

from collections import Counter
import hashlib
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
INPUT = HERE.parent / "02-stage-core-inputs"
STAGE = HERE.parent / "01-stage-contract"
for directory in (INPUT, STAGE):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))

import vulkan_docs_output_provider as PROVIDER
from vulkan_docs_input_cache import cache_session
from vulkan_docs_input_contract import stage_inputs
from vulkan_docs_output_contract import stage_outputs, verify_staged_outputs
from vulkan_docs_output_inventory import staged_trees
from vulkan_docs_output_paths import output_base, receipt_relative, staged_relative
from vulkan_docs_stage_bind import PROJECT_ROOT, build_plan
from vulkan_docs_stage_marker import marker_relative
from vulkan_docs_stage_model import RUN_IDS, StageError

BIND = HERE.parent.parent
OBSERVATION = BIND / "03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json"
ARTIFACTS = PROJECT_ROOT / ".artifacts/graphics/f02.4.4.1.5.2.3.3.1"


class OutputStageIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temporary = tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-output-stage-")
        cls.addClassCleanup(cls.temporary.cleanup)
        cls.root = Path(cls.temporary.name).resolve()
        cls.plan = build_plan(OBSERVATION, ARTIFACTS, cls.root)
        cls.inputs = stage_inputs(OBSERVATION, ARTIFACTS, cls.root)
        cls.calls: list[tuple[str, str]] = []
        original = PROVIDER.Providers.read

        def traced(source: object, capture: object, item: object) -> bytes:
            assert isinstance(capture, dict)
            cls.calls.append((capture["run_id"], item.selector))
            return original(source, capture, item)

        with patch.object(PROVIDER.Providers, "read", new=traced):
            cls.receipt = stage_outputs(OBSERVATION, ARTIFACTS, cls.root)
        with cache_session(cls.plan) as cache:
            cls.trees = staged_trees(cls.plan, cache)

    def test_stage_reads_each_member_from_each_recorded_run(self) -> None:
        expected = Counter((tree.run_id, item.selector) for tree in self.trees for item in tree.members)
        self.assertEqual(tuple(tree.run_id for tree in self.trees), RUN_IDS)
        self.assertEqual([len(tree.members) for tree in self.trees], [2530, 2530])
        self.assertEqual(len(self.calls), 5060)
        self.assertEqual(Counter(self.calls), expected)

    def test_receipt_and_cache_keep_two_separate_complete_trees(self) -> None:
        expected = {staged_relative(self.plan, tree.run_id, item.selector)
                    for tree in self.trees for item in tree.members}
        found = {item.relative_to(self.root).as_posix()
                 for item in (self.root / output_base(self.plan, RUN_IDS[0]).rsplit("/", 1)[0]).rglob("*")
                 if item.is_file()}
        self.assertEqual((self.receipt["status"], self.receipt["admitted"], self.receipt["cutover_ready"]),
                         ("staging-only-unadmitted", False, False))
        self.assertEqual(self.receipt["inputs"], self.inputs["inputs"])
        self.assertEqual(self.receipt["input_stage_sha256"], self.inputs["input_stage_sha256"])
        self.assertEqual(self.receipt["output_witnesses"], [tree.manifest() for tree in self.trees])
        self.assertEqual(found, expected | {receipt_relative(self.plan)})
        self.assertEqual(len(expected), 5060)
        self.assertFalse((self.root / marker_relative(self.plan)).exists())

    def test_equal_witnesses_do_not_deduplicate_run_targets(self) -> None:
        first, second = self.trees
        self.assertEqual(first.sha256, second.sha256)
        self.assertEqual([(item.selector, item.bytes, item.sha256) for item in first.members],
                         [(item.selector, item.bytes, item.sha256) for item in second.members])
        first_targets = {staged_relative(self.plan, first.run_id, item.selector) for item in first.members}
        second_targets = {staged_relative(self.plan, second.run_id, item.selector) for item in second.members}
        self.assertFalse(first_targets & second_targets)
        self.assertEqual(len(first_targets | second_targets), 5060)

    def test_primary_and_known_zero_byte_members_are_cached_per_run(self) -> None:
        for tree in self.trees:
            primary = self.root / staged_relative(self.plan, tree.run_id, tree.primary.selector)
            zero = self.root / staged_relative(self.plan, tree.run_id, "formats/timeMarker")
            self.assertEqual((tree.primary.selector, tree.primary.bytes), ("out/html/vkspec.html", 10377052))
            self.assertEqual(hashlib.sha256(primary.read_bytes()).hexdigest(), tree.primary.sha256)
            self.assertEqual(zero.read_bytes(), b"")

    def test_public_verify_rehashes_and_accepts_the_staged_witnesses(self) -> None:
        self.assertEqual(verify_staged_outputs(OBSERVATION, ARTIFACTS, self.root), self.receipt)

    def test_second_public_stage_refuses_existing_receipt(self) -> None:
        with self.assertRaisesRegex(StageError, "existing output-stage receipt"):
            stage_outputs(OBSERVATION, ARTIFACTS, self.root)

    def test_stray_output_member_is_rejected(self) -> None:
        stray = self.root / output_base(self.plan, RUN_IDS[0]) / "unlisted.txt"
        stray.write_bytes(b"x")
        with self.assertRaisesRegex(StageError, "(does not match|inventory)"):
            verify_staged_outputs(OBSERVATION, ARTIFACTS, self.root)
        stray.unlink()

    def test_z_tampered_output_is_rejected(self) -> None:
        item = next(item for item in self.trees[0].members if item.bytes)
        target = self.root / staged_relative(self.plan, self.trees[0].run_id, item.selector)
        payload = target.read_bytes()
        target.write_bytes(payload[:-1] + bytes((payload[-1] ^ 1,)))
        with self.assertRaisesRegex(StageError, "(does not match|sha256 mismatch|byte count mismatch)"):
            verify_staged_outputs(OBSERVATION, ARTIFACTS, self.root)


if __name__ == "__main__":
    unittest.main()
