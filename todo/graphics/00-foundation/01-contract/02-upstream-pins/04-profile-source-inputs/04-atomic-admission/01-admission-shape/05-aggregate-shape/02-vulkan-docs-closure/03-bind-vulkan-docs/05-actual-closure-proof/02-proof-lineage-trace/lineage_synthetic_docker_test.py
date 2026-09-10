#!/usr/bin/env python3
"""Lock the fixture collector to its unprivileged, no-network container shape."""

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from lineage_model import LineageError
from lineage_synthetic_docker import IMAGE, SOURCE, command, run


def roots(directory: str) -> tuple[Path, Path]:
    root = Path(directory); vulkan, work = root / "vulkan", root / "work"; vulkan.mkdir(); (work / "temporary").mkdir(parents=True); (work / "generated").mkdir()
    return vulkan, work


class SyntheticDockerTest(unittest.TestCase):
    def test_command_mounts_only_fixture_roots_without_privilege_or_network(self):
        with tempfile.TemporaryDirectory() as temporary:
            vulkan, work = roots(temporary); value = command("docker", SOURCE, vulkan, work); script = value[-1]
        self.assertEqual(value[:9], ["docker", "run", "--rm", "--pull=never", "--network", "none", "--platform", "linux/amd64", "--user"])
        self.assertEqual(value[value.index("--user") + 1], "501:20"); self.assertIn(IMAGE, value)
        self.assertIn("dst=/source,readonly", " ".join(value)); self.assertIn("dst=/vulkan,readonly", " ".join(value)); self.assertIn("dst=/work", " ".join(value))
        self.assertIn("gcc -std=c11 -O2 -Wall -Werror", script); self.assertIn("/work/collector /work/fixture", script)
        self.assertNotIn("LD_PRELOAD", script)
        self.assertFalse(any(flag == "--privileged" or flag.startswith(("--cap-add", "--security-opt")) for flag in value))

    def test_rejects_missing_directories_and_a_nonzero_collector(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); root.mkdir(exist_ok=True)
            with self.assertRaises(LineageError): command("docker", SOURCE, root, root)
        with tempfile.TemporaryDirectory() as temporary:
            vulkan, work = roots(temporary); failed = subprocess.CompletedProcess([], 77, b'{"kind":"terminal","status":"blocked"}\n', b"")
            with patch("lineage_synthetic_docker.shutil.which", return_value="docker"), patch("lineage_synthetic_docker.subprocess.run", return_value=failed):
                with self.assertRaises(LineageError): run(vulkan, work)


if __name__ == "__main__": unittest.main()
