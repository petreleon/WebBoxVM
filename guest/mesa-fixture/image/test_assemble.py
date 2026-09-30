"""Atomic publication, deterministic bytes, and missing inputs fail closed."""

import hashlib
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import assemble
import cpio
from test_cpio import decode


class AssemblyTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.root = self.base / "root"
        self.modules = self.base / "modules"
        self.root.mkdir()
        self.modules.mkdir()
        self.kernel = self.base / "kernel"
        self.kernel.write_bytes(bytes(56) + b"ARM\x64" + bytes(4))
        for name in assemble.REQUIRED:
            directory = self.modules if name.startswith("usr/lib/modules/") else self.root
            path = directory / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"fixture-" + name.encode())
            path.chmod(0o755)
        self.output = self.base / "output"

    def run_assembly(self, output=None):
        return assemble.assemble(self.root, self.modules, self.kernel, output or self.output)

    def test_clean_rebuild_bytes_manifest_and_init_devices(self):
        first = self.run_assembly()
        for path in self.root.rglob("*"):
            os.utime(path, (123, 123))
        second_dir = self.base / "second"
        second = self.run_assembly(second_dir)
        self.assertEqual(first, second)
        for name in ("Image", "initrd.cpio", "manifest.json"):
            self.assertEqual((self.output / name).read_bytes(), (second_dir / name).read_bytes())
        entries = {name: (fields, data) for name, fields, data in decode((self.output / "initrd.cpio").read_bytes())}
        self.assertEqual(entries["dev/console"][0][9:11], [5, 1])
        self.assertEqual(entries["dev/null"][0][9:11], [1, 3])
        self.assertEqual(entries["init"][1], (assemble.HERE / "init.sh").read_bytes())
        recorded = json.loads((self.output / "manifest.json").read_text())
        recorded_hash = recorded.pop("manifest_sha256")
        self.assertEqual(recorded_hash, hashlib.sha256(assemble.canonical(recorded)).hexdigest())
        self.assertFalse(recorded["software_fallback_allowed"])

    def test_missing_driver_tool_or_module_rejects_publication(self):
        for name in ("usr/bin/eglinfo", "opt/mesa-f02/lib/libgallium-25.3.6.so",
                     "lib/aarch64-linux-gnu/libvulkan.so.1",
                     "usr/lib/modules/6.12.94+deb13-arm64/kernel/drivers/virtio/virtio_mmio.ko.xz",
                     "usr/lib/modules/6.12.94+deb13-arm64/modules.dep"):
            directory = self.modules if name.startswith("usr/lib/modules/") else self.root
            path = directory / name
            content = path.read_bytes()
            path.unlink()
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.run_assembly()
            self.assertFalse((self.output / "manifest.json").exists())
            path.write_bytes(content)

    def test_non_executable_shell_tool_or_probe_rejects_publication(self):
        for name in ("bin/sh", "usr/sbin/modprobe", "usr/bin/webboxvm-mesa-vulkan"):
            path = self.root / name
            path.chmod(0o644)
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, "executable"):
                self.run_assembly()
            self.assertFalse(self.output.exists())
            path.chmod(0o755)

    def test_empty_required_driver_rejects_publication(self):
        (self.root / "opt/mesa-f02/lib/libgallium-25.3.6.so").write_bytes(b"")
        with self.assertRaisesRegex(ValueError, "nonempty"):
            self.run_assembly()

    def test_invalid_or_symlink_kernel_rejected(self):
        for content in (b"short", bytes(64)):
            self.kernel.write_bytes(content)
            with self.assertRaisesRegex(ValueError, "ARM64"):
                self.run_assembly()
        actual = self.base / "actual"
        self.kernel.rename(actual)
        self.kernel.symlink_to(actual)
        with self.assertRaisesRegex(ValueError, "regular file"):
            self.run_assembly()

    def test_changed_input_aborts_without_replacing_previous_artifacts(self):
        self.run_assembly()
        fresh = self.base / "fresh-output"
        previous = {name: path.read_bytes() for name, path in
                    ((name, self.output / name) for name in ("Image", "initrd.cpio", "manifest.json"))}
        original = cpio.write

        def change(path, nodes):
            result = original(path, nodes)
            (self.root / "usr/bin/eglinfo").write_bytes(b"changed")
            return result

        with patch.object(cpio, "write", side_effect=change), self.assertRaisesRegex(ValueError, "changed"):
            self.run_assembly(fresh)
        self.assertFalse(fresh.exists())
        for name, content in previous.items():
            self.assertEqual((self.output / name).read_bytes(), content)

    def test_existing_fixture_is_preserved_and_promotion_failure_publishes_nothing(self):
        self.run_assembly()
        previous = (self.output / "manifest.json").read_bytes()
        with self.assertRaisesRegex(ValueError, "fresh directory"):
            self.run_assembly()
        fresh = self.base / "fresh"
        with patch.object(Path, "rename", side_effect=OSError("promotion failed")), self.assertRaises(OSError):
            self.run_assembly(fresh)
        self.assertFalse(fresh.exists())
        self.assertEqual((self.output / "manifest.json").read_bytes(), previous)

    def test_conflicting_input_tree_rejected(self):
        (self.modules / "usr/bin").mkdir(parents=True)
        (self.modules / "usr/bin/eglinfo").write_bytes(b"different")
        with self.assertRaisesRegex(ValueError, "conflict"):
            self.run_assembly()

    def test_probe_tree_is_merged_without_overwriting_modules(self):
        probes = self.base / "probes"
        (probes / "usr/bin").mkdir(parents=True)
        (probes / "usr/bin/probe-extra").write_bytes(b"probe")
        manifest = assemble.assemble(self.root, self.modules, self.kernel, self.output, probes)
        self.assertIn("usr/bin/probe-extra", [row["path"] for row in manifest["files"]])
        (probes / "usr/lib/modules/6.12.94+deb13-arm64").mkdir(parents=True)
        (probes / "usr/lib/modules/6.12.94+deb13-arm64/modules.dep").write_bytes(b"conflict")
        with self.assertRaisesRegex(ValueError, "conflict"):
            assemble.assemble(self.root, self.modules, self.kernel, self.base / "fresh", probes)

    def test_output_symlinks_cannot_redirect_artifacts(self):
        actual = self.base / "actual-output"
        actual.mkdir()
        self.output.symlink_to(actual, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "fresh directory"):
            self.run_assembly()
        self.output.unlink()
        self.output.mkdir()
        sentinel = self.base / "sentinel"
        sentinel.write_bytes(b"keep")
        (self.output / "Image").symlink_to(sentinel)
        with self.assertRaisesRegex(ValueError, "fresh directory"):
            self.run_assembly()
        self.assertEqual(sentinel.read_bytes(), b"keep")
        self.assertFalse((self.output / "manifest.json").exists())


if __name__ == "__main__":
    unittest.main()
