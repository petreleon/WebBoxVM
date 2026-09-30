"""Exercise image authentication and subprocess failure/log preservation."""

import contextlib
import hashlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest

import run
from verify_image import canonical, verify


class FixtureRunTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.image = self.root / "image"
        self.image.mkdir()
        manifest = {"schema": 1, "kind": "webboxvm-stock-mesa-initramfs", "software_fallback_allowed": False}
        for name, field in (("Image", "kernel"), ("initrd.cpio", "initrd")):
            content = name.encode()
            (self.image / name).write_bytes(content)
            manifest[field] = {"bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()}
        manifest["manifest_sha256"] = hashlib.sha256(canonical(manifest)).hexdigest()
        self.manifest = manifest
        self.save_manifest()
        self.runner = self.root / "runner"
        self.result = self.root / "result.json"

    def save_manifest(self):
        (self.image / "manifest.json").write_text(json.dumps(self.manifest))

    def program(self, code):
        self.runner.write_text("#!" + sys.executable + "\n" + code)
        self.runner.chmod(0o755)

    def execute(self, **kwargs):
        with contextlib.redirect_stdout(io.StringIO()):
            return run.execute(self.image, self.runner, "tools", self.result, **kwargs)

    def test_manifest_and_bytes_are_authenticated(self):
        self.assertEqual(verify(self.image), self.manifest)
        self.manifest["extra"] = "tampered"
        self.save_manifest()
        with self.assertRaisesRegex(ValueError, "self hash"):
            verify(self.image)
        self.manifest.pop("extra")
        self.save_manifest()
        (self.image / "Image").write_bytes(b"tampered")
        with self.assertRaisesRegex(ValueError, "bytes differ"):
            verify(self.image)

    def test_symlink_image_artifact_is_rejected(self):
        (self.image / "Image").rename(self.root / "kernel-bytes")
        (self.image / "Image").symlink_to(self.root / "kernel-bytes")
        with self.assertRaisesRegex(ValueError, "redirected"):
            verify(self.image)

    def test_failed_child_preserves_first_exit_and_raw_uart(self):
        self.program("import pathlib,sys\np=pathlib.Path(sys.argv[sys.argv.index('--uart-log')+1])\n"
                     "p.write_bytes(b'first failure\\x00\\xff')\nprint('before failure')\nsys.exit(17)\n")
        self.assertEqual(self.execute(), 17)
        result = json.loads(self.result.read_text())
        self.assertEqual((result["result"], result["exit_status"]), ("FAIL", 17))
        uart = Path(result["artifacts"][2]["path"])
        self.assertEqual(uart.read_bytes(), b"first failure\x00\xff")

    def test_repeated_runs_preserve_unique_logs(self):
        self.program("import pathlib,sys\npathlib.Path(sys.argv[sys.argv.index('--uart-log')+1]).write_bytes(b'raw')\n")
        self.assertEqual(self.execute(), 0)
        first = json.loads(self.result.read_text())
        self.assertEqual(self.execute(), 0)
        second = json.loads(self.result.read_text())
        self.assertNotEqual(first["artifacts"][2]["path"], second["artifacts"][2]["path"])
        self.assertEqual(Path(first["artifacts"][2]["path"]).read_bytes(), b"raw")
        self.assertEqual(json.loads((Path(first["artifacts"][2]["path"]).parent / "result.json").read_text()), first)
        self.assertEqual(json.loads((Path(second["artifacts"][2]["path"]).parent / "result.json").read_text()), second)
        self.assertFalse(second["conformance_claimed"])

    def test_zero_exit_without_uart_is_failure(self):
        self.program("pass\n")
        self.assertEqual(self.execute(), 1)
        self.assertEqual(json.loads(self.result.read_text())["result"], "FAIL")

    def test_input_drift_fails_after_preserving_uart(self):
        self.program("import pathlib,sys\n"
                     "pathlib.Path(sys.argv[sys.argv.index('--uart-log')+1]).write_bytes(b'raw')\n"
                     "with pathlib.Path(sys.argv[0]).open('a') as stream: stream.write('# changed\\n')\n")
        self.assertEqual(self.execute(), 1)
        result = json.loads(self.result.read_text())
        self.assertEqual(result["result"], "FAIL")
        self.assertIn("changed during execution", result["errors"][0])
        self.assertEqual(Path(result["artifacts"][2]["path"]).read_bytes(), b"raw")

    def test_timeout_kills_child_and_preserves_failure(self):
        self.program("import time\ntime.sleep(60)\n")
        self.assertEqual(self.execute(timeout=0.05), 1)
        result = json.loads(self.result.read_text())
        self.assertTrue(result["timeout"])
        self.assertLess(result["duration_seconds"], 3)

    def test_result_symlink_cannot_overwrite_other_file(self):
        self.program("pass\n")
        sentinel = self.root / "sentinel"
        sentinel.write_bytes(b"preserve")
        self.result.symlink_to(sentinel)
        with self.assertRaisesRegex(ValueError, "symlink"):
            self.execute()
        self.assertEqual(sentinel.read_bytes(), b"preserve")


if __name__ == "__main__":
    unittest.main()
