#!/usr/bin/env python3
"""Ambient engine aliases must neither execute nor get discarded by fixed loaders."""

import importlib.util
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import ModuleType
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "scripts/graphics/inventories"
spec = importlib.util.spec_from_file_location("inventory_bootstrap_test_api", PACKAGE / "bootstrap.py")
API = importlib.util.module_from_spec(spec)
spec.loader.exec_module(API)


class BootstrapTests(unittest.TestCase):
    def test_driver_ignores_working_directory_and_ambient_package(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "scripts").mkdir()
            (root / "scripts/__init__.py").write_text("raise RuntimeError('ambient scripts package executed')")
            result = subprocess.run([sys.executable, "-B", str(ROOT / "scripts/graphics_inventory_batch.py"), "--help"],
                                    cwd=root, env={**os.environ, "PYTHONPATH": str(root)}, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("--output-dir", result.stdout)

    def test_preloaded_package_and_child_aliases_are_ignored_and_restored(self):
        aliases = {name: ModuleType(name) for name in ("scripts.graphics.inventories", "_test_fixed_inventory", "_test_fixed_inventory.engine")}
        for module in aliases.values():
            module.InventoryEngine = lambda *args: self.fail("ambient engine executed")
        with patch.dict(sys.modules, aliases):
            actual = API.load(PACKAGE, "_test_fixed_inventory")
            self.assertTrue(actual.InventoryEngine.__module__.startswith("_test_fixed_inventory."))
            for name, module in aliases.items():
                self.assertIs(sys.modules[name], module)
            self.assertNotIn("_test_fixed_inventory.source", sys.modules)

    def test_failed_load_restores_prior_aliases(self):
        with tempfile.TemporaryDirectory() as temporary:
            package = Path(temporary)
            (package / "__init__.py").write_text("raise ValueError('broken package')")
            prior = ModuleType("_broken_inventory")
            with patch.dict(sys.modules, {"_broken_inventory": prior}):
                with self.assertRaises(ValueError):
                    API.load(package, "_broken_inventory")
                self.assertIs(sys.modules["_broken_inventory"], prior)
            self.assertNotIn("_broken_inventory", sys.modules)

    def test_symlinked_package_or_module_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "__init__.py").write_text("")
            (root / "engine.py").symlink_to(PACKAGE / "engine.py")
            with self.assertRaises(ValueError):
                API.load(root, "_linked_inventory")
            (root / "engine.py").unlink()
            link = root / "linked-package"
            link.symlink_to(PACKAGE, target_is_directory=True)
            with self.assertRaises(ValueError):
                API.load(link, "_linked_inventory")


if __name__ == "__main__":
    unittest.main()
