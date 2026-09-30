"""Dependency closure and isolated-command failure/provenance regressions."""

from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock
import subprocess

import kernel_modules as modules
from kernel_tools import BUILDER, Commands, container


def fixture(root):
    (root / "kernel").mkdir()
    for name in ("virtio-gpu", "drm", "helper", "post", "unrelated"):
        (root / "kernel" / (name + ".ko.xz")).write_bytes(name.encode())
    (root / "modules.dep").write_text("kernel/virtio-gpu.ko.xz: kernel/drm.ko.xz\n"
        "kernel/drm.ko.xz:\nkernel/helper.ko.xz: kernel/drm.ko.xz\n"
        "kernel/post.ko.xz:\nkernel/unrelated.ko.xz:\n")
    (root / "modules.softdep").write_text("# generated\nsoftdep virtio_gpu pre: helper builtin post: post\n")
    (root / "modules.builtin").write_text("kernel/builtin.ko\n")
    (root / "modules.builtin.modinfo").write_bytes(b"builtin.name=builtin\0")
    (root / "modules.order").write_text("kernel/drm.ko\nkernel/unrelated.ko\n"
        "kernel/helper.ko\nkernel/virtio-gpu.ko\nkernel/post.ko\n")


class ModuleTests(unittest.TestCase):
    def test_independent_mmio_transport_is_required_beside_gpu_dependencies(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture(root)
            with self.assertRaisesRegex(ValueError, "virtio_mmio"):
                modules.closure(root, ("virtio_mmio", "virtio_gpu"))
            (root / "kernel/virtio_mmio.ko.xz").write_bytes(b"transport")
            with (root / "modules.dep").open("a") as stream:
                stream.write("kernel/virtio_mmio.ko.xz: kernel/drm.ko.xz\n")
            selected, edges = modules.closure(root, ("virtio_mmio", "virtio_gpu"))
            self.assertIn("kernel/virtio_mmio.ko.xz", selected)
            self.assertEqual(len(selected), 5)
            self.assertIn({"source": "kernel/virtio_mmio.ko.xz", "kind": "hard",
                           "target": "kernel/drm.ko.xz", "builtin": False}, edges)
            self.assertNotIn("kernel/unrelated.ko.xz", selected)

    def test_hard_pre_post_and_builtin_dependencies_are_all_recorded(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture(root)
            selected, edges = modules.closure(root)
            self.assertEqual(selected, ["kernel/drm.ko.xz", "kernel/helper.ko.xz", "kernel/post.ko.xz", "kernel/virtio-gpu.ko.xz"])
            self.assertEqual({row["kind"] for row in edges}, {"hard", "soft-pre", "soft-post"})
            self.assertIn({"source": "kernel/virtio-gpu.ko.xz", "kind": "soft-pre", "target": "builtin", "builtin": True}, edges)
            target = root / "pruned"
            target.mkdir()
            modules.copy_closure(root, target, selected)
            self.assertFalse((target / "kernel/unrelated.ko.xz").exists())
            self.assertNotIn("unrelated", (target / "modules.order").read_text())
            self.assertEqual((target / "modules.builtin.modinfo").read_bytes(), b"builtin.name=builtin\0")

    def test_cycles_are_finite_and_missing_soft_dependencies_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture(root)
            (root / "modules.softdep").write_text("softdep virtio_gpu post: helper\nsoftdep helper pre: virtio_gpu\n")
            self.assertEqual(len(modules.closure(root)[0]), 3)
            (root / "modules.softdep").write_text("softdep virtio_gpu pre: missing\n")
            with self.assertRaises(ValueError):
                modules.closure(root)

    def test_only_selected_soft_dependencies_affect_the_gpu_closure(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture(root)
            original = modules.closure(root)
            with (root / "modules.softdep").open("a") as stream:
                stream.write("softdep unrelated gcm\n")
            self.assertEqual(modules.closure(root), original)
            with (root / "modules.softdep").open("a") as stream:
                stream.write("softdep virtio_gpu gcm\n")
            with self.assertRaisesRegex(ValueError, "pre/post"):
                modules.closure(root)

    def test_unsafe_duplicate_and_unresolved_hard_rows_are_rejected(self):
        for text in ["../escape.ko:\n", "/absolute.ko:\n", "kernel/a.ko: missing.ko\n",
                     "kernel/a.ko:\nkernel/a.ko:\n", "kernel/a.ko::\n", "kernel/a.txt:\n"]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                modules.dependencies(text)
        for text in ["softdep a b\n", "bad a pre: b\n", "softdep a pre: ../escape\n"]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                modules.soft_dependencies(text)

    def test_symlink_module_and_ambiguous_normalized_names_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture(root)
            (root / "kernel/drm.ko.xz").unlink()
            (root / "kernel/drm.ko.xz").symlink_to("helper.ko.xz")
            with self.assertRaises(ValueError):
                modules.closure(root)
            (root / "kernel/drm.ko.xz").unlink()
            (root / "kernel/drm.ko.xz").write_bytes(b"drm")
            (root / "kernel/virtio_gpu.ko.xz").write_bytes(b"duplicate")
            with (root / "modules.dep").open("a") as stream:
                stream.write("kernel/virtio_gpu.ko.xz:\n")
            with self.assertRaises(ValueError):
                modules.closure(root)

    def test_first_failed_command_and_streams_are_preserved(self):
        with tempfile.TemporaryDirectory() as temporary:
            logs = Path(temporary) / "logs"
            commands = Commands(logs)
            with self.assertRaisesRegex(ValueError, "first failure exit 7"):
                commands.run([sys.executable, "-c", "import sys;print('partial');sys.exit(7)"], "first")
            self.assertEqual((logs / "first.stdout").read_bytes(), b"partial\n")
            self.assertEqual(commands.rows[0]["exit_status"], 7)
            self.assertTrue((logs / "first.json").is_file())
            with self.assertRaises(ValueError):
                Commands(logs)

    def test_container_has_no_network_privileges_or_host_root_mount(self):
        argv = container(BUILDER, "/usr/sbin/depmod")
        for option in ["--pull=never", "--network=none", "--read-only", "--cap-drop=ALL", "--security-opt=no-new-privileges"]:
            self.assertIn(option, argv)
        self.assertIn("--tmpfs", argv)
        self.assertNotIn("--mount", argv)
        self.assertNotIn("--privileged", argv)

    def test_timeout_retains_first_status_and_removes_only_its_private_container(self):
        with tempfile.TemporaryDirectory() as temporary:
            commands = Commands(Path(temporary) / "logs")
            failure = subprocess.TimeoutExpired("docker", 120, output=b"partial", stderr=b"slow")
            cleanup = subprocess.CompletedProcess("docker", 0, b"removed\n", b"")
            with mock.patch("kernel_tools.subprocess.run", side_effect=[failure, cleanup]) as run:
                with self.assertRaisesRegex(ValueError, "first failure exit 124"):
                    commands.run(container(BUILDER, "/usr/sbin/depmod"), "timeout")
            row = commands.rows[0]
            self.assertEqual(row["exit_status"], 124)
            self.assertEqual(row["cleanup"]["exit_status"], 0)
            owned = row["argv"][row["argv"].index("--name") + 1]
            self.assertTrue(owned.startswith("webboxvm-i01-kernel-"))
            self.assertEqual(run.call_args_list[1].args[0], ["docker", "rm", "--force", owned])
            self.assertEqual((commands.logs / "timeout.stdout").read_bytes(), b"partial")


if __name__ == "__main__":
    unittest.main()
