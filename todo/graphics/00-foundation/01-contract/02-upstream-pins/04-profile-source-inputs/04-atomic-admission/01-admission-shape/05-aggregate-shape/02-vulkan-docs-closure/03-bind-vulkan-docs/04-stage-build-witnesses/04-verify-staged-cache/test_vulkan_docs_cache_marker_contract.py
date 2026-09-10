#!/usr/bin/env python3
"""Positive end-to-end coverage for the reusable Docs staging marker."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
INPUT = HERE.parent / "02-stage-core-inputs"
OUTPUT = HERE.parent / "03-stage-output-witnesses"
STAGE = HERE.parent / "01-stage-contract"
for directory in (INPUT, OUTPUT, STAGE):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))

import vulkan_docs_cache_marker_store as STORE
import vulkan_docs_input_store as INPUT_STORE
import vulkan_docs_output_provider as OUTPUT_PROVIDER
from vulkan_docs_cache_marker_contract import publish_staged_cache, reuse_staged_cache
from vulkan_docs_input_contract import stage_inputs
from vulkan_docs_output_contract import stage_outputs
from vulkan_docs_stage_bind import PROJECT_ROOT, build_plan
from vulkan_docs_stage_marker import marker_relative, parse
from vulkan_docs_stage_model import RUN_IDS

BIND = HERE.parent.parent
OBSERVATION = BIND / "03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json"
ARTIFACTS = PROJECT_ROOT / ".artifacts/graphics/f02.4.4.1.5.2.3.3.1"


class CacheMarkerIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temporary = tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-cache-marker-")
        cls.addClassCleanup(cls.temporary.cleanup)
        cls.root = Path(cls.temporary.name).resolve()
        cls.plan = build_plan(OBSERVATION, ARTIFACTS, cls.root)
        cls.inputs = stage_inputs(OBSERVATION, ARTIFACTS, cls.root)
        cls.outputs = stage_outputs(OBSERVATION, ARTIFACTS, cls.root)

    def test_publish_and_reuse_rehash_the_closed_cache_without_providers(self) -> None:
        input_batches: list[int] = []
        output_batches: list[tuple[int, ...]] = []
        original_rehash, original_trees = STORE._rehash, STORE.staged_trees

        def traced_rehash(plan: object, cache: object, planned: object) -> None:
            input_batches.append(len(planned))
            original_rehash(plan, cache, planned)

        def traced_trees(plan: object, cache: object) -> object:
            trees = original_trees(plan, cache)
            output_batches.append(tuple(len(tree.members) for tree in trees))
            return trees

        def provider_read(*_args: object, **_kwargs: object) -> bytes:
            raise AssertionError("marker publication and reuse must not reopen a provider")

        with patch.object(INPUT_STORE.Providers, "read", new=provider_read), \
             patch.object(OUTPUT_PROVIDER.Providers, "read", new=provider_read), \
             patch.object(STORE, "_rehash", new=traced_rehash), \
             patch.object(STORE, "staged_trees", new=traced_trees):
            published = publish_staged_cache(OBSERVATION, ARTIFACTS, self.root)
            reused = reuse_staged_cache(OBSERVATION, ARTIFACTS, self.root)

        self.assertEqual(reused, published)
        self.assertTrue(input_batches and all(count == 1760 for count in input_batches))
        self.assertGreaterEqual(len(input_batches), 3)
        self.assertTrue(output_batches and all(counts == (2530, 2530) for counts in output_batches))
        self.assertGreaterEqual(len(output_batches), 3)
        self.assertEqual((published["status"], published["admitted"], published["cutover_ready"]),
                         ("staging-only-unadmitted", False, False))
        self.assertEqual(published["plan_sha256"], self.plan.digest)
        self.assertEqual(published["inputs"], self.inputs["inputs"])
        self.assertEqual(tuple(item["run_id"] for item in published["output_witnesses"]), RUN_IDS)
        self.assertEqual(tuple(item["file_count"] for item in published["output_witnesses"]), (2530, 2530))
        self.assertEqual(published["output_witnesses"], self.outputs["output_witnesses"])
        marker = self.root / marker_relative(self.plan)
        stored = json.loads(marker.read_text(encoding="utf-8"))
        self.assertEqual(parse(stored, self.plan), published)
        self.assertEqual(sum(item.is_file() for item in self.root.rglob("*")), 6823)


if __name__ == "__main__":
    unittest.main()
