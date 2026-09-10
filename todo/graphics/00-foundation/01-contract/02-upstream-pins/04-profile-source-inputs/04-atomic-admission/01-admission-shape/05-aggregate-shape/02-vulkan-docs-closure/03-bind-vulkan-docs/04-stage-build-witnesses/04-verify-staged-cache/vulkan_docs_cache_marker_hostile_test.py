#!/usr/bin/env python3
"""Hostile regressions for the one-way reusable Docs cache marker."""

from __future__ import annotations

import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

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
from vulkan_docs_input_bind import members
from vulkan_docs_input_contract import stage_inputs
from vulkan_docs_input_paths import staged_relative as input_relative
from vulkan_docs_input_paths import worktree_relative
from vulkan_docs_output_contract import stage_outputs
from vulkan_docs_output_model import decode, encode
from vulkan_docs_output_paths import receipt_relative as output_receipt_relative
from vulkan_docs_output_paths import staged_relative as output_relative
from vulkan_docs_stage_bind import PROJECT_ROOT, build_plan
from vulkan_docs_stage_marker import marker_relative
from vulkan_docs_stage_model import MARKER_CONTRACT, RUN_IDS, StageError
from vulkan_docs_stage_parse import canonical

BIND = HERE.parent.parent
OBSERVATION = BIND / "03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json"
ARTIFACTS = PROJECT_ROOT / ".artifacts/graphics/f02.4.4.1.5.2.3.3.1"


class CacheMarkerHostileTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temporary = tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-marker-hostile-")
        cls.addClassCleanup(cls.temporary.cleanup)
        cls.fixture = Path(cls.temporary.name).resolve()
        stage_inputs(OBSERVATION, ARTIFACTS, cls.fixture)
        stage_outputs(OBSERVATION, ARTIFACTS, cls.fixture)

    def clone(self) -> tuple[Path, object]:
        temporary = tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-marker-case-")
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name).resolve() / "cache"
        shutil.copytree(self.fixture, root)
        return root, build_plan(OBSERVATION, ARTIFACTS, root)

    def reject_reuse(self, root: Path) -> None:
        with self.assertRaises(StageError):
            reuse_staged_cache(OBSERVATION, ARTIFACTS, root)

    def seal(self, payload: bytes, **changes: object) -> bytes:
        marker = decode(payload)
        marker.update(changes)
        marker.pop("marker_sha256")
        marker["marker_sha256"] = canonical(marker, MARKER_CONTRACT)
        return encode(marker)

    def test_missing_or_partial_stage_refuses_publication_without_a_marker(self) -> None:
        for kind in ("input", "output-receipt"):
            with self.subTest(kind=kind):
                root, plan = self.clone()
                if kind == "input":
                    item = next(item for item in members(plan) if item.bytes)
                    target = root / input_relative(plan, item)
                else:
                    target = root / output_receipt_relative(plan)
                target.unlink()
                with self.assertRaises(StageError):
                    publish_staged_cache(OBSERVATION, ARTIFACTS, root)
                self.assertFalse((root / marker_relative(plan)).exists())

    def test_reuse_rejects_mutated_marker_payloads_and_closure_extras(self) -> None:
        root, plan = self.clone()
        published = publish_staged_cache(OBSERVATION, ARTIFACTS, root)
        marker = root / marker_relative(plan)
        original = marker.read_bytes()

        def forbidden(*_args: object, **_kwargs: object) -> bytes:
            raise AssertionError("reuse must not reopen a staging provider")

        with mock.patch.object(INPUT_STORE.Providers, "read", new=forbidden), \
             mock.patch.object(OUTPUT_PROVIDER.Providers, "read", new=forbidden):
            self.assertEqual(reuse_staged_cache(OBSERVATION, ARTIFACTS, root), published)
        stale = decode(original)
        stale["marker_sha256"] = "0" * 64
        reordered = decode(original)
        reordered["output_witnesses"].reverse()
        reordered.pop("marker_sha256")
        reordered["marker_sha256"] = canonical(reordered, MARKER_CONTRACT)
        variants = {
            "malformed": b"{",
            "stale-self-digest": encode(stale),
            "active-state": self.seal(original, status="active", admitted=True, cutover_ready=True),
            "cross-plan": self.seal(original, plan_sha256="0" * 64),
            "reordered-output-witnesses": encode(reordered),
        }
        for label, payload in variants.items():
            with self.subTest(marker=label):
                marker.write_bytes(payload)
                try:
                    self.reject_reuse(root)
                finally:
                    marker.write_bytes(original)
        input_item = next(item for item in members(plan) if item.bytes)
        output = root / output_relative(plan, RUN_IDS[0], "out/html/vkspec.html")
        for label, target in (("input", root / input_relative(plan, input_item)), ("output", output)):
            with self.subTest(cache=label):
                saved = target.read_bytes(); target.write_bytes(b"x")
                try:
                    self.reject_reuse(root)
                finally:
                    target.write_bytes(saved)
        extra = root / worktree_relative(plan) / "root-level-junk"
        extra.write_bytes(b"x")
        try:
            self.reject_reuse(root)
        finally:
            extra.unlink()
        sibling = marker.parent / "not-the-marker.json"
        sibling.write_bytes(b"x")
        try:
            self.reject_reuse(root)
        finally:
            sibling.unlink()

    def test_competing_marker_publication_never_overwrites_the_winner(self) -> None:
        root, plan = self.clone()
        original = STORE.Cache.atomic

        def race(cache: object, value: str, payload: object, label: str, **kwargs: object) -> None:
            original(cache, value, b"{}", label, maximum_bytes=kwargs["maximum_bytes"])
            original(cache, value, payload, label, **kwargs)

        with mock.patch.object(STORE.Cache, "atomic", new=race), self.assertRaisesRegex(StageError, "refuses to overwrite"):
            publish_staged_cache(OBSERVATION, ARTIFACTS, root)
        self.assertEqual((root / marker_relative(plan)).read_bytes(), b"{}")


if __name__ == "__main__":
    unittest.main()
