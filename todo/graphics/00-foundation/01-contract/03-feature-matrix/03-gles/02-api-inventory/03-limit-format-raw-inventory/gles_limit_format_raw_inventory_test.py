#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.3."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "gles_limit_format_raw_inventory.py"
LIVE_ROOT = Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f03323_inventory_test", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


RAW = load()


class GlesLimitFormatRawInventoryTests(unittest.TestCase):
    def live(self) -> Path:
        cache = RAW.CATALOG.cache_api()
        if not cache.cache_file(LIVE_ROOT, cache.SOURCE).is_file():
            self.skipTest("retained external F03.3.2.1 cache is unavailable")
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

    def test_exact_slice_pins_routes_and_no_claim_boundary(self) -> None:
        value = RAW.validate(self.live())
        self.assertEqual((value["profile"], value["source_class"], value["raw_fact_count"]), ("gles-3.2", "limit-format", 34))
        self.assertEqual((value["source"]["record_id"], value["source"]["sha256"]),
                         ("gles-32-spec", "5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c"))
        self.assertEqual(value["cache_provenance"]["physical_pdf_pages"], 601)
        self.assertEqual([row["source_order"] for row in value["facts"]], list(range(1, 35)))
        self.assertEqual({row["physical_page"] for row in value["facts"]}, {504, 505, 506})
        self.assertTrue(all(set(row) == RAW.FACT_KEYS for row in value["facts"]))
        tables = value["coverage_manifest"]["reviewed_tables"]
        self.assertEqual([(row["table"], len(row["included_rows"]), len(row["excluded_rows"])) for row in tables],
                         [("21.40", 18, 0), ("21.41", 7, 5), ("21.42", 9, 7)])
        self.assertFalse(value["coverage_manifest"]["complete"])

    def test_source_class_admission_cache_and_immutable_pins_fail_closed(self) -> None:
        for source_class in ("command-state", "shader", "precision", "extension"):
            with self.subTest(source_class=source_class), self.assertRaises(RAW.CATALOG.InventoryError):
                RAW.CATALOG.source_input(Path("/not-used"), source_class)
        cache = RAW.CATALOG.cache_api()
        with tempfile.TemporaryDirectory() as temporary:
            target = cache.cache_file(Path(temporary), cache.SOURCE)
            target.parent.mkdir(parents=True); target.write_bytes(b"stale")
            with self.assertRaisesRegex(RAW.CATALOG.InventoryError, "stale, mixed"):
                RAW.CATALOG.source_input(Path(temporary))

    def test_missing_ambiguous_table_and_row_anchors_fail(self) -> None:
        input_data, original = RAW.CATALOG.source_input(self.live()), RAW.CATALOG.page_text
        def missing(raw, page):
            text = original(raw, page)
            return text.replace("Table 21.40:", "Table absent:", 1) if page == 504 else text
        def ambiguous(raw, page):
            text = original(raw, page)
            return text + "\nMAX SAMPLES\n" if page == 505 else text
        with patch.object(RAW.CATALOG, "page_text", side_effect=missing), self.assertRaisesRegex(
                RAW.CATALOG.InventoryError, "missing or ambiguous implementation"):
            RAW.CATALOG.facts(input_data["raw"], input_data["physical_pdf_pages"])
        with patch.object(RAW.CATALOG, "page_text", side_effect=ambiguous), self.assertRaisesRegex(
                RAW.CATALOG.InventoryError, "missing, ambiguous, or reordered"):
            RAW.CATALOG.facts(input_data["raw"], input_data["physical_pdf_pages"])

    def test_rehashed_order_profile_identity_and_type_promotions_fail(self) -> None:
        edits = (
            lambda value: value.update(profile="opengl-4.6-core"), lambda value: value["facts"].reverse(),
            lambda value: value["facts"].append(copy.deepcopy(value["facts"][0])),
            lambda value: value["facts"].pop(), lambda value: value["coverage_manifest"].update(complete=True),
            lambda value: value["facts"][0].update(source_locator="gles32-pdf-v1:page=999;section=13"),
            lambda value: value.update(status="supported"), lambda value: value.update(promotion_allowed=1),
            lambda value: value.update(raw_fact_count=34.0), lambda value: value["source"].update(bytes=2198754.0),
        )
        for edit in edits:
            temporary, path = self.copied()
            with temporary, self.assertRaises(RAW.InventoryError):
                self.mutate(path, edit); RAW.validate(self.live(), path)

    def test_private_catalog_ignores_an_ambient_alias(self) -> None:
        decoy, prior = object(), sys.modules.get("gles_limit_format_raw_catalog")
        sys.modules["gles_limit_format_raw_catalog"] = decoy
        try:
            self.assertEqual(RAW.validate(self.live())["profile"], "gles-3.2")
            self.assertIs(sys.modules["gles_limit_format_raw_catalog"], decoy)
        finally:
            if prior is None:
                sys.modules.pop("gles_limit_format_raw_catalog", None)
            else:
                sys.modules["gles_limit_format_raw_catalog"] = prior


if __name__ == "__main__":
    unittest.main()
