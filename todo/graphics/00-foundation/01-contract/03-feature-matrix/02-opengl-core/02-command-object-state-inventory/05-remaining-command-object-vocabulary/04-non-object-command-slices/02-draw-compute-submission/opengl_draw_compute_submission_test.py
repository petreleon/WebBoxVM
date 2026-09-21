#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.2.2.5.4.2."""

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
PROBE, LIVE_ROOT = HERE / "opengl_draw_compute_submission.py", Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f0322542_test_inventory", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


INVENTORY = load()


class OpenGlDrawComputeSubmissionTests(unittest.TestCase):
    def copied(self):
        temporary = tempfile.TemporaryDirectory()
        directory = Path(temporary.name) / "inventory"
        directory.mkdir()
        names = [INVENTORY.ARTIFACTS.ROOT_NAME] + [name for _, name in INVENTORY.ARTIFACTS.FRAGMENTS]
        for name in names:
            shutil.copyfile(HERE / name, directory / name)
        return temporary, directory / INVENTORY.ARTIFACTS.ROOT_NAME

    def rewrite_root(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        body = {key: item for key, item in value.items() if key != "raw_inventory_sha256"}
        value["raw_inventory_sha256"] = INVENTORY.ARTIFACTS.digest(body)
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_exact_source_order_bindings_and_no_claim_boundary(self) -> None:
        value = INVENTORY.validate(LIVE_ROOT)
        rows = value["declarations"]
        expected = (("DrawArrays", 388, "10.4"), ("DrawArraysInstancedBaseInstance", 388, "10.4"),
                    ("DrawArraysInstanced", 388, "10.4"), ("DrawArraysIndirect", 389, "10.4"),
                    ("MultiDrawArrays", 389, "10.4"), ("MultiDrawArraysIndirect", 390, "10.4"),
                    ("MultiDrawArraysIndirectCount", 391, "10.4"), ("DrawElements", 393, "10.4"),
                    ("DrawElementsInstancedBaseInstance", 393, "10.4"), ("DrawElementsInstanced", 394, "10.4"),
                    ("MultiDrawElements", 394, "10.4"), ("DrawRangeElements", 394, "10.4"),
                    ("DrawElementsBaseVertex", 395, "10.4"), ("DrawRangeElementsBaseVertex", 395, "10.4"),
                    ("DrawElementsInstancedBaseVertex", 395, "10.4"), ("DrawElementsInstancedBaseVertexBaseInstance", 395, "10.4"),
                    ("DrawElementsIndirect", 396, "10.4"), ("MultiDrawElementsIndirect", 396, "10.4"),
                    ("MultiDrawElementsIndirectCount", 397, "10.4"), ("MultiDrawElementsBaseVertex", 398, "10.4"),
                    ("DrawTransformFeedback", 472, "13.3.3"), ("DrawTransformFeedbackInstanced", 472, "13.3.3"),
                    ("DrawTransformFeedbackStream", 472, "13.3.3"), ("DrawTransformFeedbackStreamInstanced", 472, "13.3.3"),
                    ("DispatchCompute", 568, "19"), ("DispatchComputeIndirect", 569, "19"))
        self.assertEqual([(row["document_name"], row["physical_page"], row["numeric_section"]) for row in rows], list(expected))
        self.assertEqual([row["c_name"] for row in rows], ["gl" + item[0] for item in expected])
        self.assertEqual(value["classification_binding"]["included_family_ids"], ["draw-submission", "transform-feedback-draw", "compute-submission"])
        self.assertEqual(value["classification_binding"]["out_of_scope_same_route_family_ids"], ["conditional-rendering"])
        self.assertEqual([row["document_name"] for row in value["formal_source_exclusions"]], ["DrawArraysOneInstance", "DrawElementsOneInstance"])
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["formal_source_declaration_count"], value["raw_submission_declaration_count"], value["cts_executions"], value["matrix_row_count"]), (28, 26, 0, 0))

    def test_source_anchor_and_dependency_swaps_are_refused(self) -> None:
        source, _, _, raw, classification, grammar = INVENTORY.source_input(LIVE_ROOT)
        original = INVENTORY.RULES.page_text
        def missing(source_bytes, page):
            text = original(source_bytes, page)
            return text.replace("void DispatchComputeIndirect( intptr indirect );", "void Guessed( intptr indirect );") if page == 569 else text
        with patch.object(INVENTORY.RULES, "page_text", side_effect=missing), self.assertRaises(INVENTORY.RULES.RuleError):
            INVENTORY.RULES.rows(raw, INVENTORY.GRAMMAR.RULES)
        broken_classification = copy.deepcopy(classification)
        for row in broken_classification["command_families"]:
            if row.get("id") == "compute-submission":
                row["anchor"]["physical_page"] = 569
        with self.assertRaisesRegex(INVENTORY.InventoryError, "exact included"):
            INVENTORY.classifier_binding(broken_classification, source)
        broken_grammar = copy.deepcopy(grammar)
        broken_grammar["rules"]["c_binding_prefix"]["c_command_prefix"] = "bad"
        with self.assertRaisesRegex(INVENTORY.InventoryError, "C-name normalization"):
            INVENTORY.grammar_binding(broken_grammar, source)

    def test_rehashed_promotion_duplicate_and_fragment_tampering_fail(self) -> None:
        temporary, target = self.copied()
        with temporary:
            self.rewrite_root(target, lambda value: value["claims"].update(api_support=True))
            with self.assertRaisesRegex(INVENTORY.InventoryError, "raw root artifact"):
                INVENTORY.validate(LIVE_ROOT, target)
        temporary, target = self.copied()
        with temporary:
            fragment = target.parent / "opengl_draw_compute_compute.json"
            value = json.loads(fragment.read_text(encoding="utf-8"))
            value["declarations"][0]["c_name"] = "glGuess"
            body = {key: item for key, item in value.items() if key != "artifact_sha256"}
            value["artifact_sha256"] = INVENTORY.ARTIFACTS.digest(body)
            fragment.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(INVENTORY.InventoryError):
                INVENTORY.validate(LIVE_ROOT, target)
        temporary, target = self.copied()
        with temporary:
            target.write_text('{"schema":1,"schema":1}', encoding="utf-8")
            with self.assertRaisesRegex(INVENTORY.InventoryError, "duplicate"):
                INVENTORY.validate(LIVE_ROOT, target)

    def test_private_loading_is_symlink_and_alias_resistant(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            link = Path(directory) / "rules-link.py"
            link.symlink_to(INVENTORY.RULES_PATH)
            with self.assertRaisesRegex(INVENTORY.InventoryError, "regular file"):
                INVENTORY.private(link, "f0322542_symlink_probe")
        temporary, target = self.copied()
        with temporary:
            fragment = target.parent / "opengl_draw_compute_compute.json"
            fragment.unlink()
            fragment.symlink_to(HERE / fragment.name)
            with self.assertRaisesRegex(INVENTORY.InventoryError, "regular file"):
                INVENTORY.validate(LIVE_ROOT, target)
        aliases = ("f0322542_cache", "f0322542_classifier", "f0322542_grammar", "f0322542_rules", "f0322542_artifacts")
        code = "\n".join((
            "import importlib.util,sys,types", "from pathlib import Path",
            f"aliases={aliases!r}; decoys={{name:types.ModuleType(name) for name in aliases}}; sys.modules.update(decoys)",
            f"spec=importlib.util.spec_from_file_location('probe',{str(PROBE)!r})",
            "module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)",
            f"assert module.validate(Path({str(LIVE_ROOT)!r}))['raw_submission_declaration_count']==26",
            "assert all(sys.modules[name] is decoys[name] for name in aliases)",
        ))
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        isolated = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, env=environment, cwd=tempfile.gettempdir())
        self.assertEqual(isolated.returncode, 0, isolated.stderr)
        passed = subprocess.run([sys.executable, str(PROBE), "--cache-root", str(LIVE_ROOT)], capture_output=True, text=True, env=environment)
        self.assertEqual((passed.returncode, passed.stdout.strip()), (0, "PASS: 26 source-only draw/compute submission declarations; matrix-incomplete"))


if __name__ == "__main__":
    unittest.main()
