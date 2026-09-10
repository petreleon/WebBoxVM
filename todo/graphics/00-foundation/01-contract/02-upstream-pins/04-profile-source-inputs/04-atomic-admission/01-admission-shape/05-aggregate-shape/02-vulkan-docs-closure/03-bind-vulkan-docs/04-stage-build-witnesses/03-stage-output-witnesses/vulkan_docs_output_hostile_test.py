#!/usr/bin/env python3
"""Hostile held-provider and sealed-receipt regressions for Docs outputs."""

from __future__ import annotations

import copy
import os
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve().parent
INPUT = HERE.parent / "02-stage-core-inputs"
STAGE = HERE.parent / "01-stage-contract"
for directory in (INPUT, STAGE):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))

import vulkan_docs_output_provider_fs as PROVIDER_FS
import vulkan_docs_output_store as STORE
from vulkan_docs_input_bind import members as input_members, receipt_value as input_receipt_value
from vulkan_docs_output_bind import parse, receipt_from_bytes, receipt_value
from vulkan_docs_output_model import OUTPUT_STAGE_CONTRACT, OutputMember, OutputTree
from vulkan_docs_output_provider import providers
from vulkan_docs_output_tree import validate
from vulkan_docs_stage_bind import PROJECT_ROOT, build_plan, plan_value
from vulkan_docs_stage_model import StageError, _live_plan
from vulkan_docs_stage_parse import canonical

BIND = HERE.parent.parent
OBSERVATION = BIND / "03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json"
ARTIFACTS = PROJECT_ROOT / ".artifacts/graphics/f02.4.4.1.5.2.3.3.1"


def reseal(value: dict[str, object]) -> None:
    payload = dict(value)
    payload.pop("output_stage_sha256")
    value["output_stage_sha256"] = canonical(payload, OUTPUT_STAGE_CONTRACT)


class OutputHostileTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temporary = tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-output-hostile-")
        cls.addClassCleanup(cls.temporary.cleanup)
        cls.cache = Path(cls.temporary.name).resolve()
        cls.plan = build_plan(OBSERVATION, ARTIFACTS, cls.cache)
        cls.inputs = input_receipt_value(cls.plan, input_members(cls.plan))
        with providers(cls.plan, ARTIFACTS) as source:
            cls.trees = tuple(validate(cls.plan, source.scan(capture)) for capture in cls.plan.value["captures"])
        cls.receipt = receipt_value(cls.plan, cls.inputs, cls.trees)

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-output-provider-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()

    def fake_provider(self, files: dict[str, bytes] | None = None) -> Path:
        root = Path(tempfile.mkdtemp(dir=self.root, prefix="artifact-")).resolve()
        for capture in self.plan.value["captures"]:
            source = root / "sources" / capture["run_id"]
            generated = root / capture["artifact"] / "generated"
            observer = root / capture["artifact"] / "observer"
            source.mkdir(parents=True)
            generated.mkdir(parents=True)
            observer.mkdir(parents=True)
            run = ARTIFACTS / capture["artifact"] / "observer/run.json"
            (observer / "run.json").write_bytes(run.read_bytes())
            for selector, payload in (files or {"out/html/vkspec.html": b"x"}).items():
                target = generated / selector
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(payload)
        return root

    def target(self, root: Path, run_id: str = "observer-a") -> Path:
        capture = next(item for item in self.plan.value["captures"] if item["run_id"] == run_id)
        return root / capture["artifact"] / "generated/out/html/vkspec.html"

    def test_provider_rejects_unsafe_members_and_tightened_bounds(self) -> None:
        left = self.plan.value["captures"][0]
        for kind in ("symlink", "hardlink", "fifo"):
            with self.subTest(kind=kind):
                root, target, side = self.fake_provider(), None, None
                target = self.target(root); side = root / "side"
                side.write_bytes(b"x"); target.unlink()
                if kind == "symlink": target.symlink_to(side)
                elif kind == "hardlink": os.link(side, target)
                else: os.mkfifo(target)
                with providers(self.plan, root) as source, self.assertRaises(StageError): source.scan(left)
        root = self.fake_provider({"out/html/vkspec.html": b"x", "other": b"y"})
        with mock.patch.object(PROVIDER_FS, "MAX_OUTPUT_FILES", 1), providers(self.plan, root) as source:
            with self.assertRaisesRegex(StageError, "regular-file bound"): source.scan(left)
        root = self.fake_provider()
        with mock.patch.object(PROVIDER_FS, "MAX_OUTPUT_MEMBER_BYTES", 0), providers(self.plan, root) as source:
            with self.assertRaisesRegex(StageError, "bounded"): source.scan(left)

    def test_provider_rejects_aliases_mutation_and_held_root_swap(self) -> None:
        left, right = self.plan.value["captures"]
        for run_id in (left["run_id"], right["run_id"]):
            root = self.fake_provider(); target = self.target(root, run_id).parents[2]
            shutil.rmtree(target)
            target.symlink_to(root / "sources" / left["run_id"] if run_id == left["run_id"] else self.target(root, left["run_id"]).parents[3], target_is_directory=True)
            with self.assertRaises(StageError):
                with providers(self.plan, root): pass
        root, target, changed = self.fake_provider(), None, False
        target = self.target(root); original = PROVIDER_FS.os.read
        def rewrite(descriptor: int, count: int) -> bytes:
            nonlocal changed
            result = original(descriptor, count)
            if not changed: changed = True; target.write_bytes(b"z")
            return result
        with providers(self.plan, root) as source, mock.patch.object(PROVIDER_FS.os, "read", side_effect=rewrite):
            with self.assertRaisesRegex(StageError, "changed"): source.scan(left)
        root = self.fake_provider(); moved = root.parent / f"{root.name}-moved"
        with providers(self.plan, root) as source:
            root.rename(moved); root.mkdir()
            try:
                with self.assertRaisesRegex(StageError, "root changed"): source.confirm()
            finally:
                root.rmdir(); moved.rename(root)

    def test_resealed_receipt_context_and_duplicate_json_are_rejected(self) -> None:
        with self.assertRaisesRegex(StageError, "duplicate JSON keys"):
            receipt_from_bytes(b'{"schema":1,"schema":1}', self.plan, self.inputs, self.trees)
        mutations = (
            lambda value: value.__setitem__("schema", 1.0),
            lambda value: value["captures"][0].__setitem__("artifact", "runs/other"),
            lambda value: value["captures"][0].__setitem__("run_sha256", "a" * 64),
            lambda value: value.__setitem__("build_witness_sha256", "a" * 64),
            lambda value: value["inputs"].__setitem__("total_count", 0),
        )
        for mutate in mutations:
            value = copy.deepcopy(self.receipt); mutate(value); reseal(value)
            with self.assertRaises(StageError): parse(value, self.plan, self.inputs, self.trees)

    def test_untrusted_plan_primary_and_plan_witness_mismatches_are_rejected(self) -> None:
        untrusted = plan_value(copy.deepcopy(self.plan.value), self.cache)
        with self.assertRaisesRegex(StageError, "live pinned provenance"):
            STORE._stage(untrusted, ARTIFACTS, self.inputs)
        tree = self.trees[0]
        bad_primary = OutputTree(tree.run_id, tree.members, tree.bytes, tree.sha256,
                                 OutputMember(tree.primary.selector, tree.primary.bytes, "a" * 64))
        with self.assertRaises(StageError): validate(self.plan, bad_primary)
        value = copy.deepcopy(self.plan.value)
        value["witness"]["outputs"][0]["sha256"] = "a" * 64
        payload = dict(value); payload.pop("plan_sha256")
        value["plan_sha256"] = canonical(payload, "webboxvm-graphics-vulkan-docs-build-staging-plan-v1")
        with self.assertRaisesRegex(StageError, "primary HTML"):
            validate(_live_plan(plan_value(value, self.cache)), tree)


if __name__ == "__main__":
    unittest.main()
