#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.2.2."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE = Path("/private/tmp/webboxvm-f0341.cqT6ZX")
TARGET_PATH = HERE / "gles_declaration_grammar.py"


def target():
    spec = importlib.util.spec_from_file_location("f033222_gles_grammar_test_target", TARGET_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load target")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


TARGET = target()


class GlesDeclarationGrammarTest(unittest.TestCase):
    def test_sealed_catalog_binds_all_formal_source_templates(self) -> None:
        value = TARGET.validate(CACHE)
        grammar = value["grammar"]
        forms = {item["formal_name"]: item for item in grammar["formal_templates"]}
        self.assertEqual((value["profile"], value["source_class"], value["physical_pdf_pages"]), ("gles-3.2", "command-state", 601))
        self.assertEqual((len(value["anchors"]), len(forms), grammar["c_binding"]), (5, 27, {"document_command_prefix": "", "c_command_prefix": "gl", "input": "unprefixed-command-name"}))
        self.assertEqual((forms["Uniform{1234}{if ui}"]["expansion_count"], forms["ProgramUniformMatrix{2x3,3x2,2x4,4x2,3x4,4x3}{f}v"]["expansion_count"], forms["ClearBuffer{if ui}v"]["expansion_count"]), (12, 6, 3))
        self.assertFalse(value["promotion_allowed"])

    def test_normalizes_ordered_literals_and_all_source_anchored_template_shapes(self) -> None:
        uniform = TARGET.normalize([("template", "Uniform{1234}{if ui}")])
        self.assertEqual(uniform, ["glUniform1i", "glUniform1f", "glUniform1ui", "glUniform2i", "glUniform2f", "glUniform2ui", "glUniform3i", "glUniform3f", "glUniform3ui", "glUniform4i", "glUniform4f", "glUniform4ui"])
        matrix = TARGET.normalize([("template", "ProgramUniformMatrix{2x3,3x2,2x4,4x2,3x4,4x3}{f}v")])
        self.assertEqual(matrix, ["glProgramUniformMatrix2x3fv", "glProgramUniformMatrix3x2fv", "glProgramUniformMatrix2x4fv", "glProgramUniformMatrix4x2fv", "glProgramUniformMatrix3x4fv", "glProgramUniformMatrix4x3fv"])
        self.assertEqual(TARGET.normalize([("literal", "GetError"), ("template", "ClearBuffer{if ui}v")])[-3:], ["glClearBufferiv", "glClearBufferfv", "glClearBufferuiv"])

    def test_rejects_guessed_prefixed_unexpanded_and_duplicate_forms(self) -> None:
        cases = (
            [("literal", "glGetError")], [("literal", "GL_FOO")], [("literal", "getError")],
            [("template", "Uniform{1234}{if d}")], [("template", "Unknown{1234}{if}")],
            [("template", "Uniform", ("1", "2", "3", "4"), ("i", "f"))],
            [("template", "Uniform{1234}{if}"), ("literal", "Uniform1i")], [("registry", "glGetError")],
        )
        for forms in cases:
            with self.subTest(forms=forms), self.assertRaises(TARGET.GrammarValidationError):
                TARGET.normalize(forms)

    def test_rejects_rehashed_promotion_anchor_drift_and_missing_cache(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "grammar.json"
            value = json.loads((HERE / "gles_declaration_grammar.json").read_text())
            value["api_support"] = True
            body = {key: item for key, item in value.items() if key != "grammar_sha256"}
            value["grammar_sha256"] = hashlib.sha256(TARGET.canonical(body)).hexdigest()
            path.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(TARGET.GrammarValidationError):
                TARGET.validate(CACHE, path)
            with self.assertRaises(TARGET.GrammarValidationError):
                TARGET.validate(Path(temporary))
        original = TARGET.RULES.FORMAL_TEMPLATES
        try:
            TARGET.RULES.FORMAL_TEMPLATES = original[:-1] + ((420, "999.999", original[-1][2]),)
            with self.assertRaises(TARGET.GrammarValidationError):
                TARGET.rendered(CACHE)
        finally:
            TARGET.RULES.FORMAL_TEMPLATES = original


if __name__ == "__main__":
    unittest.main()
