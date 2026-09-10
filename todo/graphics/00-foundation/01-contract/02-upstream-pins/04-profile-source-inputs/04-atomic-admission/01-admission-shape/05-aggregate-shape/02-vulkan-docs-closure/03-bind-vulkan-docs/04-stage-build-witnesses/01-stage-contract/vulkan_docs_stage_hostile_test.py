"""Hostile root, descriptor, plan, and marker regressions for Docs staging."""
from __future__ import annotations
import copy
import hashlib
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock
import vulkan_docs_stage_fs as FS
from vulkan_docs_stage_bind import PROJECT_ROOT, build_plan, plan_value
from vulkan_docs_stage_marker import exact, marker_relative, parse, value
from vulkan_docs_stage_model import StageError
from vulkan_docs_stage_parse import canonical
from vulkan_docs_stage_paths import external_root, relative
HERE = Path(__file__).resolve().parent
BIND = HERE.parent.parent
OBSERVATION = BIND / "03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json"
ARTIFACTS = PROJECT_ROOT / ".artifacts/graphics/f02.4.4.1.5.2.3.3.1"
def reseal(marker: dict[str, object]) -> None:
    payload = dict(marker)
    payload.pop("marker_sha256", None)
    marker["marker_sha256"] = canonical(payload, "webboxvm-graphics-vulkan-docs-build-staging-marker-v1")
class StagingFilesystemTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-stage-fs-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()

    def test_external_root_rejects_relative_repository_ancestor_and_symlink_aliases(self) -> None:
        for target in (Path("relative"), Path("/"), self.root / ".." / "escape", PROJECT_ROOT / "nested", PROJECT_ROOT.parent):
            with self.subTest(target=target), self.assertRaises(StageError):
                external_root(target, PROJECT_ROOT)
        alias = self.root / "stage-root-alias"
        alias.symlink_to(self.root, target_is_directory=True)
        with self.assertRaisesRegex(StageError, "symlink"):
            external_root(alias, PROJECT_ROOT)
        with self.assertRaisesRegex(StageError, "live pinned provenance"):
            FS.atomic_file(self.root, "safe/not-allowed", b"x", "test member")

    def test_descriptor_writes_refuse_traversal_leaf_alias_and_content_change(self) -> None:
        FS._atomic_file(self.root, "safe/member", b"x", "test member")
        self.assertEqual(FS._read_file(self.root, "safe/member", "test member", expected_bytes=1, expected_sha256=hashlib.sha256(b"x").hexdigest()), b"x")
        with self.assertRaisesRegex(StageError, "refuses to overwrite"):
            FS._atomic_file(self.root, "safe/member", b"x", "test member")
        with self.assertRaises(StageError):
            relative("../escape")
        alias = self.root / "safe" / "alias"
        alias.symlink_to(self.root / "safe" / "member")
        with self.assertRaises(StageError):
            FS._read_file(self.root, "safe/alias", "test alias", expected_bytes=1)
        hard_link = self.root / "safe" / "hard-link"
        os.link(self.root / "safe" / "member", hard_link)
        with self.assertRaisesRegex(StageError, "hard-link"):
            FS._read_file(self.root, "safe/member", "test member", expected_bytes=1)
        hard_link.unlink()
        original, changed = FS.os.read, False
        def rewrite(descriptor: int, count: int) -> bytes:
            nonlocal changed
            result = original(descriptor, count)
            if not changed:
                changed = True
                with (self.root / "safe" / "member").open("r+b") as stream:
                    stream.write(b"z")
            return result
        with mock.patch.object(FS.os, "read", side_effect=rewrite), self.assertRaisesRegex(StageError, "changed|sha256 mismatch"):
            FS._read_file(self.root, "safe/member", "test member", expected_bytes=1, expected_sha256=hashlib.sha256(b"x").hexdigest())

    def test_parent_swap_cannot_redirect_atomic_publish(self) -> None:
        outside = self.root / "outside"
        outside.mkdir()
        original = FS.os.link
        def swap(*args, **kwargs):
            result = original(*args, **kwargs)
            safe = self.root / "safe"
            safe.rename(self.root / "moved-safe")
            safe.symlink_to(outside, target_is_directory=True)
            return result

        with mock.patch.object(FS.os, "link", side_effect=swap), self.assertRaises(StageError):
            FS._atomic_file(self.root, "safe/member", b"payload", "test member")
        self.assertFalse((outside / "member").exists())
        self.assertEqual((self.root / "moved-safe" / "member").read_bytes(), b"payload")

    def test_leaf_mutation_after_link_is_rejected_before_publish_returns(self) -> None:
        for replacement in (False, True):
            with self.subTest(replacement=replacement):
                original, target = FS.os.unlink, self.root / "safe" / "member"
                def mutate(name, *args, **kwargs):
                    result = original(name, *args, **kwargs)
                    if isinstance(name, str) and name.startswith(".stage-"):
                        if replacement:
                            original(target)
                        target.write_bytes(b"evil")
                    return result
                with mock.patch.object(FS.os, "unlink", side_effect=mutate), self.assertRaisesRegex(StageError, "changed"):
                    FS._atomic_file(self.root, "safe/member", b"good", "test member")
                self.assertEqual(target.read_bytes(), b"evil")
                original(target)

    def test_descriptor_boundary_refuses_group_writable_cache_directories(self) -> None:
        self.root.chmod(0o770)
        with self.assertRaisesRegex(StageError, "not private"):
            FS._atomic_file(self.root, "safe/member", b"payload", "test member")
        self.root.chmod(0o700)
        (self.root / "safe").mkdir()
        (self.root / "safe").chmod(0o770)
        with self.assertRaisesRegex(StageError, "not private"):
            FS._atomic_file(self.root, "safe/member", b"payload", "test member")

class StagingMarkerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temporary = tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-stage-marker-")
        cls.plan = build_plan(OBSERVATION, ARTIFACTS, Path(cls.temporary.name).resolve())

    @classmethod
    def tearDownClass(cls) -> None:
        cls.temporary.cleanup()

    def manifests(self):
        inputs = self.plan.value["inputs"]
        witness = self.plan.value["comparison"]["output_witness"]
        input_manifest = {
            "raw_count": len(inputs["raw_records"]), "derived_count": len(inputs["derived_records"]),
            "total_count": len(inputs["raw_records"]) + len(inputs["derived_records"]),
            "manifest_sha256": inputs["input_manifest_sha256"],
        }
        output_manifests = [
            {
                "run_id": capture["run_id"], "manifest_sha256": witness["tree_sha256"],
                "file_count": witness["file_count"], "bytes": witness["bytes"],
                "tree_sha256": witness["tree_sha256"], "primary_html_sha256": witness["primary_html_sha256"],
            }
            for capture in self.plan.value["captures"]
        ]
        return input_manifest, output_manifests

    def test_marker_is_self_hashed_and_exactly_binds_all_planned_manifests(self) -> None:
        inputs, outputs = self.manifests()
        marker = value(self.plan, inputs, outputs)
        exact(marker, self.plan, inputs, outputs)
        self.assertEqual(parse(marker, self.plan), marker)

    def test_marker_refuses_a_self_sealed_plan_without_live_provenance(self) -> None:
        inputs, outputs = self.manifests()
        untrusted = plan_value(self.plan.value, self.plan.external_root)
        with self.assertRaisesRegex(StageError, "live pinned provenance"):
            value(untrusted, inputs, outputs)
        with self.assertRaisesRegex(StageError, "live pinned provenance"):
            marker_relative(untrusted)

    def test_live_plan_mutation_cannot_seal_a_marker(self) -> None:
        inputs, outputs = self.manifests()
        build = self.plan.value["witness"]["build"]
        original = build["image"]
        build["image"] = "example.invalid@sha256:" + "a" * 64
        try:
            with self.assertRaisesRegex(StageError, "changed after live"):
                value(self.plan, inputs, outputs)
        finally:
            build["image"] = original

    def test_resealed_marker_cannot_change_state_context_or_output_order(self) -> None:
        inputs, outputs = self.manifests()
        for mutate in (
            lambda marker: marker.__setitem__("status", "active"),
            lambda marker: marker.__setitem__("comparison_sha256", "f" * 64),
            lambda marker: marker["output_witnesses"].reverse(),
        ):
            with self.subTest(mutate=mutate):
                marker = value(self.plan, inputs, outputs)
                mutate(marker)
                reseal(marker)
                with self.assertRaises(StageError):
                    parse(marker, self.plan)


if __name__ == "__main__":
    unittest.main()
