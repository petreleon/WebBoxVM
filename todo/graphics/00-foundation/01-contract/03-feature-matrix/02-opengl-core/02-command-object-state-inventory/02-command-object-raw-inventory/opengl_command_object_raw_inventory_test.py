#!/usr/bin/env python3
"""Focused hostile checks for the F03.2.2.2 bounded raw inventory."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "opengl_command_object_raw_inventory.py"
LIVE_ROOT = Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f03222_test_inventory", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


INVENTORY = load()


class OpenGlCommandObjectRawInventoryTests(unittest.TestCase):
    def copied(self):
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / "inventory.json"
        shutil.copyfile(INVENTORY.INVENTORY, target)
        return temporary, target

    def rewrite(self, path: Path, edit, rehash: bool = True) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        if rehash:
            body = {key: item for key, item in value.items() if key != "inventory_sha256"}
            value["inventory_sha256"] = hashlib.sha256(INVENTORY.canonical(body)).hexdigest()
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_exact_bounded_source_order_and_no_claim_state(self) -> None:
        value = INVENTORY.validate(LIVE_ROOT)
        facts = value["facts"]
        self.assertEqual((value["source"]["record_id"], value["physical_pdf_pages"]), ("opengl-46-core-spec", 851))
        self.assertEqual((len(facts), [row["source_order"] for row in facts]), (24, list(range(1, 25))))
        self.assertEqual([row["fact_kind"] for row in facts].count("object-taxonomy"), 12)
        self.assertEqual([row["fact_kind"] for row in facts].count("command-declaration"), 12)
        self.assertEqual(value["coverage_manifest"]["coverage_decision"], "complete-for-listed-object-families-only")
        self.assertIn("all-openGL-command-universe-not-inferred", value["coverage_manifest"]["excluded"])
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["cts_executions"], value["matrix_row_count"]), (0, 0))

    def test_missing_reordered_duplicate_cross_profile_and_invalid_anchor_fail(self) -> None:
        edits = (
            lambda value: value["facts"].pop(),
            lambda value: value["facts"].reverse(),
            lambda value: value["facts"].append(copy.deepcopy(value["facts"][0])),
            lambda value: value.update(profile="gles-3.2"),
            lambda value: value["facts"][0]["condition"].update(row="Not Buffer Objects"),
        )
        for edit in edits:
            temporary, path = self.copied()
            with temporary, self.assertRaises(INVENTORY.InventoryError):
                self.rewrite(path, edit)
                INVENTORY.validate(LIVE_ROOT, path)
        temporary, path = self.copied()
        with temporary, self.assertRaisesRegex(INVENTORY.InventoryError, "duplicate"):
            path.write_text('{"schema":1,"schema":1}', encoding="utf-8")
            INVENTORY.validate(LIVE_ROOT, path)

    def test_stale_mixed_cache_and_unanchored_pdf_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, target = Path(temporary), None
            target = INVENTORY.CACHE.cache_file(root, INVENTORY.CACHE.SOURCE)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(b"stale")
            with self.assertRaises(INVENTORY.InventoryError):
                INVENTORY.rendered(root)
            target.write_bytes(b"\0" * INVENTORY.CACHE.SOURCE["bytes"])
            with self.assertRaises(INVENTORY.InventoryError):
                INVENTORY.rendered(root)
        with patch.object(INVENTORY.RULES, "page_text", return_value=""):
            with self.assertRaisesRegex(INVENTORY.InventoryError, "prefix rule"):
                INVENTORY.rendered(LIVE_ROOT)

    def test_matrix_shape_and_promotion_are_refused(self) -> None:
        for edit in (lambda value: value["facts"][0].update(status="supported"),
                     lambda value: value.update(matrix_row_count=1)):
            temporary, path = self.copied()
            with temporary, self.assertRaisesRegex(INVENTORY.InventoryError, "Matrix|promotes"):
                self.rewrite(path, edit)
                INVENTORY.validate(LIVE_ROOT, path)
        with self.assertRaisesRegex(INVENTORY.InventoryError, "cannot create a Matrix v2 row"):
            INVENTORY.reject_matrix_row({"requirement_kind": "command"}, LIVE_ROOT)

    def test_fixed_private_cache_alias_and_cli_are_isolated(self) -> None:
        code = "\n".join((
            "import importlib.util, sys, types",
            "from pathlib import Path",
            "for name in ('opengl_normative_pdf_cache','opengl_command_object_raw_rules'):",
            "    sys.modules[name] = types.ModuleType(name)",
            f"spec=importlib.util.spec_from_file_location('inventory',{str(PROBE)!r})",
            "module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
            f"assert module.validate(Path({str(LIVE_ROOT)!r}))['profile'] == 'opengl-4.6-core'",
            "assert all(isinstance(sys.modules[name], types.ModuleType) for name in ('opengl_normative_pdf_cache','opengl_command_object_raw_rules'))",
        ))
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        isolated = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, env=environment)
        self.assertEqual(isolated.returncode, 0, isolated.stderr)
        passed = subprocess.run([sys.executable, str(PROBE), "--cache-root", str(LIVE_ROOT)],
                                 capture_output=True, text=True, env=environment)
        self.assertEqual((passed.returncode, passed.stdout.strip()), (0, "PASS: 24 bounded raw command/object facts; matrix-incomplete"))


if __name__ == "__main__":
    unittest.main()
