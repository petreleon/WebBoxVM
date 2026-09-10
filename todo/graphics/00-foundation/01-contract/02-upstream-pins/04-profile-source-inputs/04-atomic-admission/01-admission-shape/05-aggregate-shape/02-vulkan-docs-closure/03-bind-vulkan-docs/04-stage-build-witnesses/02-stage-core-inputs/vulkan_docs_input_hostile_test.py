"""Hostile provider, selector, receipt, and partial-cache checks for Docs input staging."""

from __future__ import annotations

import copy
import hashlib
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import vulkan_docs_input_bind as BIND
import vulkan_docs_input_provider as PROVIDER
import vulkan_docs_input_store as STORE
from vulkan_docs_input_bind import members, parse, receipt_value
from vulkan_docs_input_cache import cache_session
from vulkan_docs_input_model import DERIVED, INPUT_STAGE_CONTRACT, MAX_INPUT_BYTES, RAW, InputMember, decode, member
from vulkan_docs_input_paths import staged_relative, worktree_relative
from vulkan_docs_input_provider import providers
from vulkan_docs_stage_bind import PROJECT_ROOT, build_plan, plan_value
from vulkan_docs_stage_fs import atomic_file
from vulkan_docs_stage_model import StageError
from vulkan_docs_stage_parse import canonical

HERE = Path(__file__).resolve().parent
BIND_ROOT = HERE.parent.parent
OBSERVATION = BIND_ROOT / "03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json"
ARTIFACTS = PROJECT_ROOT / ".artifacts/graphics/f02.4.4.1.5.2.3.3.1"


def input_row(kind: str, selector: str, size: int = 1) -> dict[str, object]:
    base = {"kind": kind, "selector": selector, "sha256": "a" * 64, "bytes": size, "phase_roles": ["build"]}
    return {**base, "revision": "r", "immutable_url": "https://example.invalid/r"} if kind == RAW else {**base, "generation_id": "g"}


class InputHostileTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temporary = tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-input-hostile-")
        cls.cache = Path(cls.temporary.name).resolve()
        cls.plan = build_plan(OBSERVATION, ARTIFACTS, cls.cache)

    @classmethod
    def tearDownClass(cls) -> None:
        cls.temporary.cleanup()

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-input-provider-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()

    def fake_provider(self) -> tuple[Path, InputMember]:
        for capture in self.plan.value["captures"]:
            (self.root / "sources" / capture["run_id"]).mkdir(parents=True)
            (self.root / capture["artifact"] / "generated").mkdir(parents=True)
            (self.root / "sources" / capture["run_id"] / "one.txt").write_bytes(b"x")
        return self.root, InputMember(RAW, "one.txt", "one.txt", 1, hashlib.sha256(b"x").hexdigest(), ("build",))

    def test_provider_roots_are_held_and_read_both_selected_capture_keys(self) -> None:
        root, item = self.fake_provider()
        left, right = self.plan.value["captures"]
        with providers(self.plan, root) as source:
            held = root / "sources" / left["run_id"]
            held.rename(root / "held")
            held.mkdir()
            (held / "one.txt").write_bytes(b"z")
            self.assertEqual(source.read(left, item), b"x")
            self.assertEqual(source.read(right, item), b"x")
            with self.assertRaisesRegex(StageError, "not one of the two selected"):
                source.read({"run_id": "foreign", "artifact": "runs/foreign"}, item)

    def test_provider_rejects_aliases_nonregular_rewrite_and_nonprivate_roots(self) -> None:
        root, item = self.fake_provider()
        target = root / "sources" / self.plan.value["captures"][0]["run_id"] / "one.txt"
        side = root / "side.txt"
        side.write_bytes(b"x")
        for kind in ("symlink", "hardlink", "directory"):
            target.unlink()
            if kind == "symlink":
                target.symlink_to(side)
            elif kind == "hardlink":
                os.link(side, target)
            else:
                target.mkdir()
            with self.subTest(kind=kind), providers(self.plan, root) as source, self.assertRaises(StageError):
                source.read(self.plan.value["captures"][0], item)
            if target.is_dir():
                target.rmdir()
            else:
                target.unlink()
            target.write_bytes(b"x")
        original, changed = PROVIDER.os.read, False

        def rewrite(descriptor: int, count: int) -> bytes:
            nonlocal changed
            if not changed:
                changed = True
                target.write_bytes(b"y")
            return original(descriptor, count)

        with mock.patch.object(PROVIDER.os, "read", side_effect=rewrite), providers(self.plan, root) as source:
            with self.assertRaisesRegex(StageError, "changed|sha256"):
                source.read(self.plan.value["captures"][0], item)
        target.write_bytes(b"x")
        target.parent.chmod(0o770)
        with self.assertRaisesRegex(StageError, "not private"):
            with providers(self.plan, root):
                pass

    def test_selectors_caps_and_live_plan_provenance_are_non_bypassable(self) -> None:
        for kind, selector, size in ((RAW, "generated/x", 1), (DERIVED, "missing/x", 1), (DERIVED, "generated/out/x", 1), (RAW, "../x", 1), (RAW, "x", MAX_INPUT_BYTES + 1)):
            with self.subTest(kind=kind, selector=selector, size=size), self.assertRaises(StageError):
                member(input_row(kind, selector, size), kind)
        with mock.patch.object(BIND, "MAX_STAGED_BYTES", 1), self.assertRaisesRegex(StageError, "cumulative"):
            members(self.plan)
        untrusted = plan_value(copy.deepcopy(self.plan.value), self.cache)
        with self.assertRaisesRegex(StageError, "live pinned provenance"):
            STORE._stage(untrusted, ARTIFACTS)

    def test_pair_divergence_and_existing_member_refuse_before_acceptance(self) -> None:
        item = InputMember(RAW, "fake.txt", "fake.txt", 1, hashlib.sha256(b"x").hexdigest(), ("build",))

        class Pair:
            def read(self, capture, unused):
                return b"x" if capture["run_id"] == "observer-a" else b"y"

        with cache_session(self.plan, create=True) as cache, self.assertRaisesRegex(StageError, "diverge"):
            STORE._stage_member(self.plan, cache, Pair(), self.plan.value["captures"], item)
        self.assertFalse((self.cache / staged_relative(self.plan, item)).exists())
        first = members(self.plan)[0]
        existing = staged_relative(self.plan, first)
        atomic_file(self.plan, existing, b"x", "hostile existing", maximum_bytes=1)
        with self.assertRaises(StageError):
            STORE._stage(self.plan, ARTIFACTS)
        self.assertEqual((self.cache / existing).read_bytes(), b"x")
        stray = f"{worktree_relative(self.plan)}/stray"
        atomic_file(self.plan, stray, b"x", "hostile stray", maximum_bytes=1)
        with self.assertRaisesRegex(StageError, "existing worktree"):
            STORE._stage(self.plan, ARTIFACTS)

    def test_held_cache_root_rejects_a_named_root_swap(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-input-swap-") as temporary:
            root = Path(temporary).resolve()
            plan = build_plan(OBSERVATION, ARTIFACTS, root)
            moved = root.parent / f"{root.name}-moved"
            with cache_session(plan, create=True) as cache:
                root.rename(moved)
                root.mkdir(mode=0o700)
                with self.assertRaisesRegex(StageError, "root changed"):
                    cache.confirm()
                root.rmdir()
                moved.rename(root)

    def test_receipt_context_and_duplicate_json_cannot_be_resealed(self) -> None:
        receipt = receipt_value(self.plan, members(self.plan))
        receipt["inputs"]["total_count"] = 0
        payload = dict(receipt)
        payload.pop("input_stage_sha256")
        receipt["input_stage_sha256"] = canonical(payload, INPUT_STAGE_CONTRACT)
        with self.assertRaises(StageError):
            parse(receipt, self.plan, members(self.plan))
        aliases = receipt_value(self.plan, members(self.plan))
        aliases["schema"] = 1.0
        payload = dict(aliases)
        payload.pop("input_stage_sha256")
        aliases["input_stage_sha256"] = canonical(payload, INPUT_STAGE_CONTRACT)
        with self.assertRaisesRegex(StageError, "exactly bind"):
            parse(aliases, self.plan, members(self.plan))
        changed = list(members(self.plan))
        changed[0] = InputMember(RAW, "other.txt", "other.txt", 1, "a" * 64, ("build",))
        with self.assertRaisesRegex(StageError, "unbound staged members"):
            receipt_value(self.plan, tuple(changed))
        with self.assertRaisesRegex(StageError, "duplicate JSON keys"):
            decode(b'{"x":1,"x":2}')


if __name__ == "__main__":
    unittest.main()
