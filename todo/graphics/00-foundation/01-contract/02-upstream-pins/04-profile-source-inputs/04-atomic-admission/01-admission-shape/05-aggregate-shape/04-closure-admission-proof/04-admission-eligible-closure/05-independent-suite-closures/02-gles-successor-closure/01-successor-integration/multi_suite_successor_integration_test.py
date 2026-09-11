#!/usr/bin/env python3
"""Hermetic positive and hostile checks for the multi-suite future integration."""

import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


INTEGRATION = module("f025_multi_suite_integration", HERE / "multi_suite_successor_integration.py")


class IntegrationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def copy(self, name: str) -> Path:
        path = Path(self.temporary.name) / name
        path.write_bytes(INTEGRATION.RECORD.read_bytes())
        return path

    def value(self, path: Path) -> dict[str, object]:
        return json.loads(path.read_text(encoding="utf-8"))

    def seal(self, value: dict[str, object]) -> None:
        value["integration_sha256"] = INTEGRATION.digest(value)

    def write(self, path: Path, value: dict[str, object]) -> None:
        path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")

    def reject(self, edit) -> None:
        path = self.copy("mutated.json")
        value = self.value(path)
        edit(value)
        self.seal(value)
        self.write(path, value)
        with self.assertRaisesRegex(INTEGRATION.IntegrationError, "exact unadmitted multi-suite boundary"):
            INTEGRATION.validate(path)

    def test_committed_integration_is_read_only_and_design_only(self) -> None:
        watched = (INTEGRATION.RECORD, INTEGRATION.DOCS.RECORD, INTEGRATION.BOUNDARY.RECORD,
                   INTEGRATION.DOCS.INVENTORY_DIR / "manifest.toml",
                   INTEGRATION.DOCS.INVENTORY_DIR / "inventory.lock", INTEGRATION.BOUNDARY.CLOSURE)
        before = {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in watched}
        value = INTEGRATION.validate()
        self.assertEqual(value["status"], "design-only-unadmitted")
        self.assertEqual(value["aggregate"]["wrapper_families"], ["vulkan-docs", "gles-cts"])
        self.assertEqual(value["effects"], {name: False for name in INTEGRATION.EFFECTS})
        result = subprocess.run([sys.executable, "-B", str(HERE / "multi_suite_successor_integration.py")],
                                capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), f"INTEGRATION: design-only-unadmitted {value['integration_sha256']}")
        self.assertEqual(before, {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in watched})

    def test_resealed_alias_omission_scope_and_promotion_are_rejected(self) -> None:
        cases = (
            lambda value: value["predecessor"]["active_families"].pop(),
            lambda value: value["docs"].__setitem__("transition_sha256", "0" * 64),
            lambda value: value["docs"].__setitem__("independently_admitted", True),
            lambda value: value["gles"].__setitem__("family", "vulkan-docs"),
            lambda value: value["gles"].__setitem__("successor_closure_ready", True),
            lambda value: value["gles"].__setitem__("per_member_max_bytes", 8 * 1024 * 1024 + 1),
            lambda value: value["aggregate"]["wrapper_families"].pop(),
            lambda value: value["aggregate"]["wrapper_families"].__setitem__(1, "vulkan-docs"),
            lambda value: value["aggregate"].__setitem__("cross_wrapper_substitution_allowed", True),
            lambda value: value["aggregate"].__setitem__("active_mutation_permitted", True),
            lambda value: value["effects"].__setitem__("admitted", True),
            lambda value: value["effects"].__setitem__("conformant", 0),
        )
        for edit in cases:
            with self.subTest(edit=edit):
                self.reject(edit)

    def test_self_hash_duplicate_oversize_fifo_and_symlink_are_rejected(self) -> None:
        path = self.copy("selfhash.json")
        value = self.value(path)
        value["integration_sha256"] = "0" * 64
        self.write(path, value)
        with self.assertRaisesRegex(INTEGRATION.IntegrationError, "self-hash"):
            INTEGRATION.validate(path)
        root = Path(self.temporary.name)
        duplicate = root / "duplicate.json"
        duplicate.write_text('{"schema":4,"schema":4}', encoding="utf-8")
        with self.assertRaisesRegex(INTEGRATION.IntegrationError, "duplicate"):
            INTEGRATION.validate(duplicate)
        oversized = root / "oversized.json"
        oversized.write_bytes(b"x" * (INTEGRATION.MAX_BYTES + 1))
        with self.assertRaisesRegex(INTEGRATION.IntegrationError, "bounded size"):
            INTEGRATION.validate(oversized)
        fifo = root / "integration.fifo"
        os.mkfifo(fifo)
        with self.assertRaisesRegex(INTEGRATION.IntegrationError, "regular file"):
            INTEGRATION.validate(fifo)
        link = root / "integration-link.json"
        link.symlink_to(INTEGRATION.RECORD)
        with self.assertRaisesRegex(INTEGRATION.IntegrationError, "regular file"):
            INTEGRATION.validate(link)


if __name__ == "__main__":
    unittest.main(verbosity=2)
