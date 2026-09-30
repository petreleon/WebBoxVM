#!/usr/bin/env python3
"""Focused extraction and cache regressions for shared graphics PDF tooling."""

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from scripts.graphics.inventories import pdf

REPO = Path(__file__).resolve().parents[1]
GLES = REPO / "todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory"
RAW = GLES / "02-command-object-state-raw-inventory"
VERTEX = RAW / "03-object-resource-command-slices/06-vertex-transform-feedback-commands"
CATALOGS = sorted(path for path in VERTEX.glob("0[1-4]-*/*catalog.py"))


def load(path: Path):
    spec = importlib.util.spec_from_file_location(path.stem, path)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def one_page_pdf() -> bytes:
    stream = b"BT /F1 12 Tf 72 720 Td (Shared extraction witness) Tj ET\n"
    objects = [b"<< /Type /Catalog /Pages 2 0 R >>", b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
               b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>",
               b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
               b"<< /Length " + str(len(stream)).encode() + b" >>\nstream\n" + stream + b"endstream"]
    value, offsets = bytearray(b"%PDF-1.4\n"), []
    for number, item in enumerate(objects, 1):
        offsets.append(len(value)); value.extend(f"{number} 0 obj\n".encode() + item + b"\nendobj\n")
    start = len(value)
    value.extend(f"xref\n0 {len(objects) + 1}\n0000000000 65535 f \n".encode())
    for offset in offsets: value.extend(f"{offset:010d} 00000 n \n".encode())
    value.extend(f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\nstartxref\n{start}\n%%EOF\n".encode())
    return bytes(value)


class PdfTests(unittest.TestCase):
    def test_fixed_command_environment_and_whitespace(self) -> None:
        result = subprocess.CompletedProcess([], 0, b" A\tB\n\f C ", b"")
        with patch.object(pdf, "extractor", return_value="/fixed/pdftotext"), patch.object(pdf.subprocess, "run", return_value=result) as run:
            self.assertEqual(pdf.sealed_pdf_page(b"source bytes", 357), "A B C")
        self.assertEqual(run.call_args.args[0], ["/fixed/pdftotext", "-f", "357", "-l", "357", "-raw", "-", "-"])
        self.assertEqual(run.call_args.kwargs, {"input": b"source bytes", "capture_output": True, "check": False,
                                             "timeout": 30, "env": {"LC_ALL": "C", "PATH": "/usr/bin:/bin"}})

    def test_rejects_invalid_input_before_launching(self) -> None:
        cases = [(b"", 1), ("bytes", 1), (bytearray(b"a"), 1), (b"a", 0), (b"a", -1), (b"a", True), (b"a", "1")]
        with patch.object(pdf.subprocess, "run") as run:
            for raw, page in cases:
                with self.subTest(raw=raw, page=page), self.assertRaises(pdf.PdfError): pdf.sealed_pdf_page(raw, page)
        run.assert_not_called()

    def test_rejects_nonzero_status_and_malformed_utf8(self) -> None:
        for result in (subprocess.CompletedProcess([], 7, b"partial output", b"failure"),
                       subprocess.CompletedProcess([], 0, b"\xff", b"")):
            with self.subTest(result=result), patch.object(pdf, "extractor", return_value="/fixed/tool"), patch.object(pdf.subprocess, "run", return_value=result):
                with self.assertRaises(pdf.PdfError): pdf.sealed_pdf_page(b"source bytes", 1)

    def test_retains_first_exit_status(self) -> None:
        result = subprocess.CompletedProcess([], 19, b"partial", b"")
        with patch.object(pdf, "extractor", return_value="/fixed/tool"), patch.object(pdf.subprocess, "run", return_value=result):
            with self.assertRaisesRegex(pdf.PdfError, "exit 19"): pdf.sealed_pdf_page(b"source bytes", 1)

    def test_rejects_os_error_and_timeout(self) -> None:
        for error in (OSError("launch failed"), subprocess.TimeoutExpired("tool", 30)):
            with self.subTest(error=error), patch.object(pdf, "extractor", return_value="/fixed/tool"), patch.object(pdf.subprocess, "run", side_effect=error):
                with self.assertRaises(pdf.PdfError): pdf.sealed_pdf_page(b"source bytes", 1)

    def test_does_not_use_ambient_path_to_find_executable(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fake = Path(temporary) / "pdftotext"
            fake.write_text("#!/bin/sh\nexit 0\n"); fake.chmod(0o755)
            with patch.object(pdf, "TRUSTED_EXTRACTORS", (Path(temporary) / "missing",)), patch.dict(os.environ, {"PATH": temporary}):
                with self.assertRaisesRegex(pdf.PdfError, "unavailable"): pdf.extractor()

    def test_real_poppler_extracts_fixture_and_rejects_missing_page(self) -> None:
        raw = one_page_pdf()
        self.assertEqual(pdf.sealed_pdf_page(raw, 1), "Shared extraction witness")
        with self.assertRaises(pdf.PdfError): pdf.sealed_pdf_page(raw, 2)

    def test_catalogs_extract_each_physical_page_once(self) -> None:
        grammar = json.loads((RAW / "02-template-declaration-grammar/gles_declaration_grammar.json").read_text())
        normalize = load(RAW / "02-template-declaration-grammar/gles_declaration_grammar_rules.py").normalize
        starts = {283: "10.2.1 Current Generic Attributes", 285: "10.3.1 Specifying Arrays for Generic Vertex Attributes", 291: "10.3.4 Primitive Restart",
                  294: "10.4 Vertex Array Objects", 354: "12.2.1 Transform Feedback Objects"}
        ends = {289: "10.3.2 Vertex Attribute Divisors", 290: "10.3.3 Transferring Array Elements",
                291: "10.3.5 Robust Buffer Access with target PRIMITIVE_RESTART_FIXED_INDEX.",
                295: "10.5 Drawing Commands Using Vertex Arrays", 357: "12.2.2 Transform Feedback Primitive Capture"}
        expected_reads, expected_rows = (1, 7, 2, 4), (12, 12, 4, 4)
        self.assertEqual(len(CATALOGS), 4)
        for position, catalog_path in enumerate(CATALOGS):
            catalog = load(catalog_path)
            inventory = json.loads(next(catalog_path.parent.glob("*inventory.json")).read_text())
            declarations = {}
            for row in inventory["raw_entries"]:
                declarations.setdefault(row[4], []).append(row[3])
            corpus = {page: " ".join((starts.get(page, ""), *dict.fromkeys(items), ends.get(page, "")))
                      for page, items in declarations.items()}
            if position == 3: corpus[357] = ends[357]
            with self.subTest(catalog=catalog.__name__), patch.object(catalog, "page_text", side_effect=lambda raw, page: corpus[page]) as read:
                args = (b"fixture", 601, 22, grammar, normalize) if position == 0 else (b"fixture", 601, catalog.FAMILY[3], normalize)
                self.assertEqual(len(catalog.facts(*args)), expected_rows[position])
                self.assertEqual(read.call_count, expected_reads[position])
                self.assertEqual(len({call.args[1] for call in read.call_args_list}), expected_reads[position])

    def test_catalog_preserves_error_type_and_mockable_wrapper(self) -> None:
        for catalog_path in CATALOGS:
            catalog = load(catalog_path)
            with self.subTest(catalog=catalog.__name__), patch.object(catalog.PDF, "sealed_pdf_page", side_effect=catalog.PDF.PdfError("fixture failure")):
                with self.assertRaisesRegex(catalog.CatalogError, "fixture failure"): catalog.page_text(b"fixture", 1)


if __name__ == "__main__": unittest.main()
