#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.2.2.5.2."""

from __future__ import annotations

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
PROBE, LIVE_ROOT = HERE / "opengl_declaration_grammar.py", Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f032252_test_grammar", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


GRAMMAR = load()


class OpenGlDeclarationGrammarTests(unittest.TestCase):
    def copied(self):
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / "grammar.json"
        shutil.copyfile(GRAMMAR.ARTIFACT, target)
        return temporary, target

    def rewrite(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        body = {key: item for key, item in value.items() if key != "normalization_sha256"}
        value["normalization_sha256"] = hashlib.sha256(GRAMMAR.canonical(body)).hexdigest()
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_exact_examples_source_pages_and_no_claim_boundary(self) -> None:
        value = GRAMMAR.validate(LIVE_ROOT)
        rules = value["rules"]
        self.assertEqual((value["source"]["record_id"], value["physical_pdf_pages"]), ("opengl-46-core-spec", 851))
        self.assertEqual([item["physical_page"] for item in rules["source_pages"]], [32, 33, 34, 163])
        self.assertEqual([item["section"] for item in rules["source_pages"]], ["2.1", "2.2", "2.2", "7.6.1"])
        self.assertEqual([item["c_name"] for item in rules["literal_examples"]], ["glUniform4f", "glGetFloatv"])
        self.assertEqual(rules["template_examples"][0]["c_names"], ["glUniform1i", "glUniform1f", "glUniform2i", "glUniform2f", "glUniform3i", "glUniform3f", "glUniform4i", "glUniform4f"])
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["cts_executions"], value["matrix_row_count"]), (0, 0))

    def test_literal_and_template_rules_refuse_guesses_duplicates_and_ambiguity(self) -> None:
        raw = GRAMMAR.source_input(LIVE_ROOT)[3]
        literal = GRAMMAR.RULES.literal(raw, 33, "2.2", GRAMMAR.RULES.LITERALS[1])
        self.assertEqual(literal["c_name"], "glGetFloatv")
        self.assertEqual(GRAMMAR.RULES.expand_template_name("UniformMatrix{2x3,3x2,2x4,4x2,3x4,4x3}{fd}v")[0], "glUniformMatrix2x3fv")
        matrix = GRAMMAR.RULES.template(raw, 163, "7.6.1", GRAMMAR.RULES.ANCHORS[-1][3])
        self.assertEqual((matrix["c_names"][0], matrix["c_names"][-1], len(matrix["c_names"])), ("glUniformMatrix2x3fv", "glUniformMatrix4x3dv", 12))
        for name in ("Uniform{12345}{if}", "Uniform{112}{if}", "Uniform{1234}{fi}", "glUniform{1234}{if}"):
            with self.subTest(name=name), self.assertRaises(GRAMMAR.RULES.RuleError):
                GRAMMAR.RULES.expand_template_name(name)
        for declaration in ("void Uniform5f( int location, float value );", "void glUniform4f( int location, float v0, float v1, float v2, float v3 );", GRAMMAR.RULES.UNIFORM):
            with self.subTest(declaration=declaration), self.assertRaises(GRAMMAR.RULES.RuleError):
                GRAMMAR.RULES.literal(raw, 33, "2.2", declaration)
        original = GRAMMAR.RULES.page_text
        duplicate = original(raw, 33) + " " + GRAMMAR.RULES.LITERALS[0]
        def duplicated(source, page):
            return duplicate if page == 33 else original(source, page)
        with patch.object(GRAMMAR.RULES, "page_text", side_effect=duplicated), self.assertRaisesRegex(GRAMMAR.RULES.RuleError, "absent or ambiguous"):
            GRAMMAR.RULES.literal(raw, 33, "2.2", GRAMMAR.RULES.LITERALS[0])

    def test_anchor_cache_and_rehashed_artifact_tampering_fail(self) -> None:
        with patch.object(GRAMMAR.RULES, "page_text", return_value="2.2. COMMAND SYNTAX"):
            with self.assertRaisesRegex(GRAMMAR.GrammarError, "source-section heading"):
                GRAMMAR.rendered(LIVE_ROOT)
        raw, original, prefix = GRAMMAR.source_input(LIVE_ROOT)[3], GRAMMAR.RULES.page_text, GRAMMAR.RULES.ANCHORS[0][3]
        def misplaced(source, page):
            text = original(source, page)
            return text.replace(prefix, "2.2 Command Syntax " + prefix) if page == 32 else text
        with patch.object(GRAMMAR.RULES, "page_text", side_effect=misplaced), self.assertRaisesRegex(GRAMMAR.GrammarError, "not before"):
            GRAMMAR.rendered(LIVE_ROOT)
        temporary, target = self.copied()
        with temporary:
            self.rewrite(target, lambda value: value["rules"]["template_examples"][0]["c_names"].pop())
            with self.assertRaises(GRAMMAR.GrammarError):
                GRAMMAR.validate(LIVE_ROOT, target)
        temporary, target = self.copied()
        with temporary:
            self.rewrite(target, lambda value: value["claims"].update(api_support=True))
            with self.assertRaisesRegex(GRAMMAR.GrammarError, "promotes"):
                GRAMMAR.validate(LIVE_ROOT, target)
        with tempfile.TemporaryDirectory() as temporary, self.assertRaises(GRAMMAR.GrammarError):
            GRAMMAR.rendered(Path(temporary))

    def test_duplicate_json_fixed_private_load_and_cli_remain_isolated(self) -> None:
        temporary, target = self.copied()
        with temporary:
            target.write_text('{"schema":1,"schema":1}', encoding="utf-8")
            with self.assertRaisesRegex(GRAMMAR.GrammarError, "duplicate"):
                GRAMMAR.validate(LIVE_ROOT, target)
        code = "\n".join((
            "import importlib.util,sys,types", "from pathlib import Path",
            "sys.modules['opengl_declaration_grammar_rules']=types.ModuleType('opengl_declaration_grammar_rules')",
            f"spec=importlib.util.spec_from_file_location('probe',{str(PROBE)!r})",
            "module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)",
            f"assert module.validate(Path({str(LIVE_ROOT)!r}))['profile']=='opengl-4.6-core'",
        ))
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        isolated = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, env=environment)
        self.assertEqual(isolated.returncode, 0, isolated.stderr)
        passed = subprocess.run([sys.executable, str(PROBE), "--cache-root", str(LIVE_ROOT)], capture_output=True, text=True, env=environment)
        self.assertEqual((passed.returncode, passed.stdout.strip()), (0, "PASS: 2 literal examples and 8 formal template expansions; matrix-incomplete"))


if __name__ == "__main__":
    unittest.main()
