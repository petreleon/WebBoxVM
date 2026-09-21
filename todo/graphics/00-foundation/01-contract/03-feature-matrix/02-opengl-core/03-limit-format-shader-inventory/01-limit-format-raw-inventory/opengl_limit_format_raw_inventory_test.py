#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.2.3.1."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "opengl_limit_format_raw_inventory.py"
LIVE_ROOT = Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f03231_inventory_test", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


RAW = load()


class OpenGlLimitFormatRawInventoryTests(unittest.TestCase):
    def live(self) -> Path:
        cache = RAW.CATALOG.cache_api()
        if not cache.cache_file(LIVE_ROOT, cache.SOURCE).is_file():
            self.skipTest("retained external F03.2.2.1 cache is unavailable")
        return LIVE_ROOT

    def copied(self):
        temporary = tempfile.TemporaryDirectory()
        target = Path(temporary.name) / RAW.INVENTORY.name
        shutil.copyfile(RAW.INVENTORY, target)
        return temporary, target

    def mutate(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        if isinstance(value.get("facts"), list):
            value["facts_sha256"] = hashlib.sha256(RAW.canonical(value["facts"])).hexdigest()
        body = {key: item for key, item in value.items() if key != "inventory_sha256"}
        value["inventory_sha256"] = hashlib.sha256(RAW.canonical(body)).hexdigest()
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_exact_bounded_facts_and_provenance(self) -> None:
        value = RAW.validate(self.live())
        self.assertEqual((value["profile"], value["source_class"], value["raw_fact_count"]),
                         ("opengl-4.6-core", "limit-format", 33))
        self.assertEqual(value["source"]["record_id"], "opengl-46-core-spec")
        self.assertEqual(value["cache_provenance"]["physical_pdf_pages"], 851)
        self.assertFalse(value["coverage_manifest"]["complete"])
        self.assertEqual([row["source_order"] for row in value["facts"]], list(range(1, 34)))
        self.assertEqual({row["physical_page"] for row in value["facts"]}, {657, 658, 659, 675})
        self.assertTrue(all(set(row) == RAW.FACT_KEYS for row in value["facts"]))
        exclusions = {row["name"] for table in value["coverage_manifest"]["reviewed_tables"]
                      for row in table["excluded_rows"]}
        self.assertTrue({"MAX TEXTURE MAX ANISOTROPY", "SHADER COMPILER"} <= exclusions)
        self.assertEqual(value["coverage_manifest"]["unadmitted_inputs"], list(RAW.CATALOG.UNADMITTED_INPUTS))

    def test_unadmitted_sources_and_stale_cache_cannot_enter(self) -> None:
        authority = RAW.CATALOG.cache_api().SOURCE_API.authority()
        for identifier in ("shader", "extension", "gl.xml", "desktop-glsl", "glsl-4.60"):
            with self.subTest(identifier=identifier), self.assertRaises(authority.BoundaryError):
                authority.consume(identifier, RAW.CATALOG.LOCATOR)
            with self.subTest(identifier=identifier), self.assertRaises(RAW.CATALOG.InventoryError):
                RAW.CATALOG.source_input(Path("/not-used"), identifier)
        with self.assertRaises(authority.BoundaryError):
            authority.consume("limit-format", "opengl45-core-pdf-v1:page=657;section=23.53")
        cache = RAW.CATALOG.cache_api()
        with tempfile.TemporaryDirectory() as temporary:
            target = cache.cache_file(Path(temporary), cache.SOURCE)
            target.parent.mkdir(parents=True); target.write_bytes(b"stale")
            with self.assertRaisesRegex(RAW.CATALOG.InventoryError, "stale, mixed"):
                RAW.CATALOG.source_input(Path(temporary))

    def test_missing_or_ambiguous_table_and_row_anchors_fail(self) -> None:
        input_data, original = RAW.CATALOG.source_input(self.live()), RAW.CATALOG.page_text
        def missing(raw, page):
            text = original(raw, page)
            return text.replace("Table 23.53:", "Table absent:", 1) if page == 657 else text
        def ambiguous(raw, page):
            text = original(raw, page)
            return text + "\n MAX CLIP DISTANCES\n" if page == 657 else text
        with patch.object(RAW.CATALOG, "page_text", side_effect=missing), self.assertRaisesRegex(
                RAW.CATALOG.InventoryError, "missing or ambiguous table"):
            RAW.CATALOG.facts(input_data["raw"], input_data["physical_pdf_pages"])
        with patch.object(RAW.CATALOG, "page_text", side_effect=ambiguous), self.assertRaisesRegex(
                RAW.CATALOG.InventoryError, "missing, ambiguous, or reordered table row"):
            RAW.CATALOG.facts(input_data["raw"], input_data["physical_pdf_pages"])

    def test_rehashed_partial_reordered_duplicate_cross_profile_anchor_and_promotion_fail(self) -> None:
        def partial(value):
            value["facts"].pop(); value["raw_fact_count"] = len(value["facts"])
        edits = (
            lambda value: value.update(profile="gles-3.2"), lambda value: value["facts"].reverse(),
            lambda value: value["facts"].append(copy.deepcopy(value["facts"][0])), partial,
            lambda value: value["coverage_manifest"].update(complete=True),
            lambda value: value["facts"][0].update(source_locator="opengl46-core-pdf-v1:page=999;section=13.7"),
            lambda value: value.update(status="supported"),
        )
        for edit in edits:
            temporary, path = self.copied()
            with temporary, self.assertRaises(RAW.InventoryError):
                self.mutate(path, edit); RAW.validate(self.live(), path)

    def test_private_catalog_ignores_ambient_alias(self) -> None:
        decoy = object()
        import sys
        prior = sys.modules.get("opengl_limit_format_raw_catalog")
        sys.modules["opengl_limit_format_raw_catalog"] = decoy
        try:
            self.assertEqual(RAW.validate(self.live())["profile"], "opengl-4.6-core")
            self.assertIs(sys.modules["opengl_limit_format_raw_catalog"], decoy)
        finally:
            if prior is None:
                sys.modules.pop("opengl_limit_format_raw_catalog", None)
            else:
                sys.modules["opengl_limit_format_raw_catalog"] = prior


if __name__ == "__main__":
    unittest.main()
