#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.2.2.5.4.1."""

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
PROBE, LIVE_ROOT = HERE / "opengl_global_execution_sync.py", Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f0322541_test_inventory", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


INVENTORY = load()


class OpenGlGlobalExecutionSyncTests(unittest.TestCase):
    def copied(self):
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / "inventory.json"
        shutil.copyfile(INVENTORY.ARTIFACT, target)
        return temporary, target

    def rewrite(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        body = {key: item for key, item in value.items() if key != "inventory_sha256"}
        value["inventory_sha256"] = hashlib.sha256(INVENTORY.canonical(body)).hexdigest()
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_exact_source_order_bindings_and_no_claim_boundary(self) -> None:
        value = INVENTORY.validate(LIVE_ROOT)
        rows = value["declarations"]
        self.assertEqual([row["document_name"] for row in rows], ["GetError", "GetGraphicsResetStatus", "Flush", "Finish", "MemoryBarrier", "MemoryBarrierByRegion"])
        self.assertEqual([row["c_name"] for row in rows], ["glGetError", "glGetGraphicsResetStatus", "glFlush", "glFinish", "glMemoryBarrier", "glMemoryBarrierByRegion"])
        self.assertEqual([(row["physical_page"], row["section"]) for row in rows], [(38, "2.3.1"), (41, "2.3.2"), (42, "2.3.3"), (43, "2.3.3"), (183, "7.13.2"), (187, "7.13.2")])
        self.assertEqual(value["classification_binding"]["family_ids"], ["global-execution", "shader-memory-sync"])
        self.assertEqual(value["declaration_grammar_binding"]["c_command_prefix"], "gl")
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["cts_executions"], value["matrix_row_count"]), (0, 0))

    def test_source_anchor_and_dependency_swaps_are_refused(self) -> None:
        source, _, _, raw, classification, grammar = INVENTORY.source_input(LIVE_ROOT)
        with self.assertRaises(INVENTORY.RULES.RuleError):
            INVENTORY.RULES.declaration_row(raw, 1, ("guess", 38, "2.3.1", "void Guessed( void );"))
        original = INVENTORY.RULES.page_text
        def missing(source_bytes, page):
            text = original(source_bytes, page)
            return text.replace("void Finish( void );", "void Missing( void );") if page == 43 else text
        with patch.object(INVENTORY.RULES, "page_text", side_effect=missing), self.assertRaises(INVENTORY.InventoryError):
            INVENTORY.rendered(LIVE_ROOT)
        broken_classification = copy.deepcopy(classification)
        for row in broken_classification["command_families"]:
            if row.get("id") == "global-execution":
                row["anchor"]["physical_page"] = 43
        with self.assertRaisesRegex(INVENTORY.InventoryError, "exact two"):
            INVENTORY.classifier_binding(broken_classification, source)
        broken_grammar = copy.deepcopy(grammar)
        broken_grammar["rules"]["c_binding_prefix"]["c_command_prefix"] = "bad"
        with self.assertRaisesRegex(INVENTORY.InventoryError, "C-name normalization"):
            INVENTORY.grammar_binding(broken_grammar, source)

    def test_rehashed_promotion_and_row_tampering_fail(self) -> None:
        temporary, target = self.copied()
        with temporary:
            self.rewrite(target, lambda value: value["claims"].update(api_support=True))
            with self.assertRaisesRegex(INVENTORY.InventoryError, "promotes"):
                INVENTORY.validate(LIVE_ROOT, target)
        temporary, target = self.copied()
        with temporary:
            self.rewrite(target, lambda value: value["declarations"].__setitem__(0, {**value["declarations"][0], "c_name": "glGuess"}))
            with self.assertRaises(INVENTORY.InventoryError):
                INVENTORY.validate(LIVE_ROOT, target)
        temporary, target = self.copied()
        with temporary:
            target.write_text('{"schema":1,"schema":1}', encoding="utf-8")
            with self.assertRaisesRegex(INVENTORY.InventoryError, "duplicate"):
                INVENTORY.validate(LIVE_ROOT, target)

    def test_fixed_private_load_and_cli_remain_isolated(self) -> None:
        code = "\n".join((
            "import importlib.util,sys,types", "from pathlib import Path",
            "sys.modules['opengl_global_execution_sync_rules']=types.ModuleType('opengl_global_execution_sync_rules')",
            f"spec=importlib.util.spec_from_file_location('probe',{str(PROBE)!r})",
            "module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)",
            f"assert module.validate(Path({str(LIVE_ROOT)!r}))['profile']=='opengl-4.6-core'",
        ))
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        isolated = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, env=environment)
        self.assertEqual(isolated.returncode, 0, isolated.stderr)
        passed = subprocess.run([sys.executable, str(PROBE), "--cache-root", str(LIVE_ROOT)], capture_output=True, text=True, env=environment)
        self.assertEqual((passed.returncode, passed.stdout.strip()), (0, "PASS: 6 source-only global execution/synchronization declarations; matrix-incomplete"))


if __name__ == "__main__":
    unittest.main()
