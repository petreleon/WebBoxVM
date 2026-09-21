#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.2.3.4.1.1."""

from __future__ import annotations

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
PROBE = HERE / "opengl_final_table_anchor_catalog.py"
LIVE_ROOT = Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f032341_catalog_test", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


CATALOG = load()


class OpenGlFinalTableAnchorCatalogTests(unittest.TestCase):
    def live(self) -> Path:
        cache = CATALOG.SOURCE.private(CATALOG.SOURCE.CACHE, "f032341_test_cache")
        if not cache.cache_file(LIVE_ROOT, cache.SOURCE).is_file():
            raise AssertionError("retained external F03.2.2.1 cache is required")
        return LIVE_ROOT

    def changed_modules(self, part: int, index: int, edit):
        modules = CATALOG.modules()
        cells = list(modules[part].ROWS)
        cells[index] = edit(cells[index])
        fake = types.SimpleNamespace(FRAGMENT_ID=modules[part].FRAGMENT_ID, ROWS=tuple(cells))
        return [fake if item == part else module for item, module in enumerate(modules)]

    def copied(self):
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / CATALOG.RECEIPT.name
        shutil.copyfile(CATALOG.RECEIPT, target)
        return temporary, target

    def rehash(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        body = {key: item for key, item in value.items() if key != "catalog_sha256"}
        value["catalog_sha256"] = CATALOG.SOURCE.sha256(body)
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_exact_source_cells_routes_and_boundaries(self) -> None:
        value = CATALOG.validate(self.live())
        receipt, rows = CATALOG.built(self.live())
        self.assertEqual(receipt, value)
        self.assertEqual((value["profile"], value["candidate_count"], value["semantic_fact_count"]),
                         ("opengl-4.6-core", 159, 0))
        self.assertEqual([part["candidate_count"] for part in value["fragments"]], [111, 48])
        self.assertEqual(value["route_counts"], {"covered": 0, "eligible-unreviewed": 124, "route-to-state": 28,
                                                  "shader-unadmitted": 3, "extension-unadmitted": 4, "out-of-domain": 0})
        self.assertEqual([row["source_order"] for row in rows], list(range(1, 160)))
        self.assertEqual({row["physical_page"] for row in rows}, set(range(660, 675)) | {676, 677, 678})
        self.assertNotIn("23.71", {row["table"] for row in rows})
        self.assertEqual([row for row in rows if row["table"] == "23.68"][0]["route"], "eligible-unreviewed")
        by_cell = {(row["table"], row["cell_text"]): row for row in rows}
        self.assertEqual(by_cell[("23.67", "DEBUG OUTPUT")]["route"], "route-to-state")
        self.assertEqual(by_cell[("23.73", "IMPLEMENTATION COLOR READ FORMAT")]["destination"], "F03.2.3.4.5.2")
        self.assertEqual(sum(row["table_column"] == "Description" for row in rows), 2)
        self.assertFalse(any(value["claims"].values()))

    def test_rehashed_partial_reordered_cross_profile_and_promoted_receipts_fail(self) -> None:
        edits = (
            lambda value: value.update(profile="gles-3.2"), lambda value: value["fragments"].reverse(),
            lambda value: value["fragments"][0].update(candidate_count=1),
            lambda value: value["route_counts"].update({"eligible-unreviewed": 125}),
            lambda value: value["claims"].update(api_support=True), lambda value: value.update(matrix_row_count=1),
            lambda value: value.update(status="supported"),
        )
        for edit in edits:
            temporary, path = self.copied()
            with temporary, self.assertRaises(CATALOG.SOURCE.CatalogError):
                self.rehash(path, edit); CATALOG.validate(self.live(), path)
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "bad.json"
            for text in ('{"schema":1,"schema":1}', '{"n":NaN}', '[]', '{bad json'):
                path.write_text(text, encoding="utf-8")
                with self.subTest(text=text), self.assertRaises(CATALOG.SOURCE.CatalogError):
                    CATALOG.validate(self.live(), path)

    def test_missing_duplicate_and_mutated_source_cells_fail_before_receipt_use(self) -> None:
        original = CATALOG.SOURCE.page_text
        def missing(raw, page):
            text = original(raw, page)
            return text.replace("Table 23.56:", "Table absent:", 1) if page == 660 else text
        def duplicate(raw, page):
            text = original(raw, page)
            return text.replace("Table 23.57:", "MAX VERTEX ATTRIBS Z+\nGetIntegerv 0\n10.2\nTable 23.57:", 1) if page == 661 else text
        for change in (missing, duplicate):
            with patch.object(CATALOG.SOURCE, "page_text", side_effect=change), self.assertRaises(CATALOG.SOURCE.CatalogError):
                CATALOG.built(self.live())
        for cell in ("FAKE CELL", "MAX VERTEX ATTRIBS"):
            modules = self.changed_modules(0, 0, lambda item: (*item[:3], cell, *item[4:]))
            with patch.object(CATALOG, "modules", return_value=modules), self.assertRaises(CATALOG.SOURCE.CatalogError):
                CATALOG.built(self.live())

    def test_rehashed_fragment_mutations_need_exact_pdf_rows(self) -> None:
        cases = ((0, 0, lambda item: (*item[:3], "MAJOR", *item[4:]), True),
                 (0, 8, lambda item: (*item[:4], "GetString", item[5]), False),
                 (1, 0, lambda item: (*item[:5], "20"), False))
        baseline = CATALOG.SOURCE.document(CATALOG.RECEIPT)["fragments"]
        for part, index, edit, altered_cell in cases:
            expected = dict(CATALOG.EXPECTED_POLICY)
            if altered_cell:
                expected[("23.56", "MAJOR")] = expected.pop(("23.56", "MAJOR VERSION"))
            modules = self.changed_modules(part, index, edit)
            with patch.object(CATALOG, "modules", return_value=modules), patch.object(CATALOG, "EXPECTED_POLICY", expected), \
                 patch.object(CATALOG, "POLICY", side_effect=lambda table, cell: expected[(table, cell)]):
                _, fragments = CATALOG.records()
                self.assertNotEqual(fragments[part]["cells_sha256"], baseline[part]["cells_sha256"])
                with self.assertRaises(CATALOG.SOURCE.CatalogError):
                    CATALOG.built(self.live())

    def test_fixed_dependencies_and_invalid_policy_outcomes_fail(self) -> None:
        with patch.object(CATALOG.SOURCE, "AUTHORITY", HERE / "missing-authority.py"), self.assertRaises(CATALOG.SOURCE.CatalogError):
            CATALOG.built(self.live())
        for outcome in (("invented", "reason", "F03.2.3.4.3"), ("route-to-state", "", "F03.2.2.3.2"),
                        ("route-to-state", "reason", ""), ("eligible-unreviewed", "fake", "F03.2.3.4.3")):
            with patch.object(CATALOG, "POLICY", return_value=outcome), self.assertRaises(CATALOG.SOURCE.CatalogError):
                CATALOG.records()

    def test_fixed_source_loader_ignores_ambient_alias(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "opengl_final_table_anchor_source.py").write_text("raise RuntimeError('ambient loaded')\n", encoding="utf-8")
            code = "\n".join(("import importlib.util,sys,types", "decoy=types.ModuleType('decoy')",
                                "sys.modules['opengl_final_table_anchor_source']=decoy",
                                f"spec=importlib.util.spec_from_file_location('probe',{str(PROBE)!r})",
                                "module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
                                f"assert module.validate(__import__('pathlib').Path({str(LIVE_ROOT)!r}))['candidate_count']==159",
                                "assert sys.modules['opengl_final_table_anchor_source'] is decoy"))
            result = subprocess.run([sys.executable, "-c", code], cwd=root, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"),
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
