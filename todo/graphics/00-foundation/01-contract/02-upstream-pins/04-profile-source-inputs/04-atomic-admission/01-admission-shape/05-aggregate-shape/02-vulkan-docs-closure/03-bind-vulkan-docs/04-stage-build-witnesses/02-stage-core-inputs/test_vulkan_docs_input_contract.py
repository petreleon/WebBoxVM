#!/usr/bin/env python3
"""Public integration coverage for the unadmitted Docs input snapshot."""

from __future__ import annotations

from collections import Counter, defaultdict
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import vulkan_docs_input_store as STORE
from vulkan_docs_input_bind import members
from vulkan_docs_input_contract import stage_inputs, verify_staged_inputs
from vulkan_docs_input_paths import receipt_relative, staged_relative, worktree_relative
from vulkan_docs_stage_bind import PROJECT_ROOT, build_plan
from vulkan_docs_stage_marker import marker_relative
from vulkan_docs_stage_model import StageError

HERE = Path(__file__).resolve().parent
BIND = HERE.parent.parent
OBSERVATION = BIND / "03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json"
ARTIFACTS = PROJECT_ROOT / ".artifacts/graphics/f02.4.4.1.5.2.3.3.1"


class InputStageIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temporary = tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-input-stage-")
        cls.addClassCleanup(cls.temporary.cleanup)
        cls.root = Path(cls.temporary.name).resolve()
        cls.plan = build_plan(OBSERVATION, ARTIFACTS, cls.root)
        cls.planned = members(cls.plan)
        cls.calls: list[tuple[str, str, str]] = []
        original = STORE.Providers.read

        def traced(source: object, capture: object, item: object) -> bytes:
            assert isinstance(capture, dict)
            cls.calls.append((capture["run_id"], item.kind, item.selector))
            return original(source, capture, item)

        with patch.object(STORE.Providers, "read", new=traced):
            cls.receipt = stage_inputs(OBSERVATION, ARTIFACTS, cls.root)

    def test_stage_reads_every_member_from_both_recorded_providers(self) -> None:
        expected = Counter(
            (capture["run_id"], item.kind, item.selector)
            for capture in self.plan.value["captures"] for item in self.planned
        )
        self.assertEqual(len(self.planned), 1760)
        self.assertEqual(len(self.calls), 3520)
        self.assertEqual(Counter(self.calls), expected)

    def test_receipt_tree_and_state_are_staging_only(self) -> None:
        inputs = self.receipt["inputs"]
        self.assertEqual((self.receipt["status"], self.receipt["admitted"], self.receipt["cutover_ready"]),
                         ("staging-only-unadmitted", False, False))
        self.assertEqual(inputs, {
            "raw_count": 298, "derived_count": 1462, "total_count": 1760,
            "manifest_sha256": self.plan.value["inputs"]["input_manifest_sha256"],
        })
        expected = {staged_relative(self.plan, item) for item in self.planned}
        expected.add(receipt_relative(self.plan))
        found = {item.relative_to(self.root).as_posix() for item in self.root.rglob("*") if item.is_file()}
        self.assertEqual(found, expected)
        self.assertFalse((self.root / marker_relative(self.plan)).exists())
        self.assertFalse(any("/outputs/" in f"/{item}/" for item in found))

    def test_same_digest_members_keep_distinct_cache_targets(self) -> None:
        grouped: dict[str, list[object]] = defaultdict(list)
        for item in self.planned:
            grouped[item.sha256].append(item)
        repeated = max(grouped.values(), key=len)
        targets = {staged_relative(self.plan, item) for item in repeated}
        self.assertEqual(len(repeated), 36)
        self.assertEqual(len({(item.kind, item.selector) for item in repeated}), 36)
        self.assertEqual(len(targets), 36)
        self.assertTrue(all((self.root / target).is_file() for target in targets))

    def test_second_public_stage_refuses_existing_receipt(self) -> None:
        with self.assertRaisesRegex(StageError, "existing input-stage receipt"):
            stage_inputs(OBSERVATION, ARTIFACTS, self.root)

    def test_verify_rehashes_the_complete_staged_snapshot(self) -> None:
        before = len(self.calls)
        self.assertEqual(verify_staged_inputs(OBSERVATION, ARTIFACTS, self.root), self.receipt)
        self.assertEqual(len(self.calls), before)

    def test_verify_rejects_an_unlisted_input_member(self) -> None:
        stray = self.root / worktree_relative(self.plan) / "inputs/raw/unlisted.txt"
        stray.write_bytes(b"x")
        with self.assertRaisesRegex(StageError, "inventory (is not exact|exceeds)"):
            verify_staged_inputs(OBSERVATION, ARTIFACTS, self.root)
        stray.unlink()

    def test_z_cache_mutation_is_rejected_by_public_verification(self) -> None:
        item = next(item for item in self.planned if item.bytes)
        target = self.root / staged_relative(self.plan, item)
        payload = target.read_bytes()
        target.write_bytes(payload[:-1] + bytes((payload[-1] ^ 1,)))
        with self.assertRaisesRegex(StageError, "sha256 mismatch"):
            verify_staged_inputs(OBSERVATION, ARTIFACTS, self.root)


if __name__ == "__main__":
    unittest.main()
