"""Reject altered runtime artifacts and mismatched installed Mesa inputs."""

import json
from pathlib import Path
import tempfile
import unittest

from image import tree
import verify_outputs as outputs


class OutputTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.build = Path(self.temporary.name)
        self.runtime = self.build / "runtime-rootfs"
        self.install = self.build / "mesa-destdir"
        for directory in (self.runtime, self.install):
            path = directory / "opt/mesa-f02/lib/driver.so"
            path.parent.mkdir(parents=True)
            path.write_bytes(b"compiled-driver")
            path.chmod(0o644)
            (path.parent / "link.so").symlink_to("driver.so")
        tool = self.runtime / "usr/bin/tool"
        tool.parent.mkdir(parents=True)
        tool.write_bytes(b"tool")
        tool.chmod(0o755)
        nodes = tree.collect(self.runtime)
        value = {"schema": 1, "drivers": ["virgl"], "legacy_dri_aliases": [],
                 "icds": ["opt/mesa-f02/share/vulkan/icd.d/virtio_icd.aarch64.json"],
                 "tools": {"sh": "/usr/bin/tool"},
                 "files": outputs.records(nodes), "bytes": len(b"compiled-driver") + 4}
        (self.build / "manifests").mkdir()
        self.manifest = self.build / "manifests/runtime.json"
        self.manifest.write_text(json.dumps(value))

    def test_exact_bytes_links_and_installed_prefix_pass(self):
        value = outputs.verify(self.build)
        self.assertEqual((value["result"], value["files"]), ("PASS", 3))

    def test_unchanged_tool_bytes_with_removed_execute_bit_fail(self):
        (self.runtime / "usr/bin/tool").chmod(0o644)
        with self.assertRaisesRegex(ValueError, "not executable"):
            outputs.verify(self.build)
        (self.runtime / "usr/bin/tool").chmod(0o755)
        (self.runtime / "usr/bin").chmod(0o744)
        with self.assertRaisesRegex(ValueError, "search permissions"):
            outputs.verify(self.build)

    def test_changed_missing_and_added_runtime_files_fail(self):
        path = self.runtime / "opt/mesa-f02/lib/driver.so"
        path.write_bytes(b"modified")
        with self.assertRaisesRegex(ValueError, "compiled manifest"):
            outputs.verify(self.build)
        path.unlink()
        with self.assertRaises(ValueError):
            outputs.verify(self.build)
        path.write_bytes(b"compiled-driver")
        (path.parent / "extra.so").write_bytes(b"unexpected")
        with self.assertRaisesRegex(ValueError, "compiled manifest"):
            outputs.verify(self.build)

    def test_installed_prefix_bytes_and_modes_must_match_runtime(self):
        path = self.install / "opt/mesa-f02/lib/driver.so"
        path.write_bytes(b"modified")
        with self.assertRaisesRegex(ValueError, "installed Mesa"):
            outputs.verify(self.build)
        path.write_bytes(b"compiled-driver")
        path.chmod(0o755)
        with self.assertRaisesRegex(ValueError, "installed Mesa"):
            outputs.verify(self.build)

    def test_redirected_link_and_software_driver_contract_fail(self):
        path = self.runtime / "opt/mesa-f02/lib/link.so"
        path.unlink()
        path.symlink_to("/outside/missing")
        with self.assertRaises(ValueError):
            outputs.verify(self.build)
        path.unlink()
        path.symlink_to("driver.so")
        value = json.loads(self.manifest.read_text())
        value["drivers"] = ["swrast"]
        self.manifest.write_text(json.dumps(value))
        with self.assertRaisesRegex(ValueError, "driver contract"):
            outputs.verify(self.build)


if __name__ == "__main__":
    unittest.main()
