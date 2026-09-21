#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.2.3.4.2."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
CACHE = Path("/private/tmp/webboxvm-f0341.cqT6ZX")
PROBE = HERE / "gles_texture_image_copy_subimage_inventory.py"


def load():
    spec = importlib.util.spec_from_file_location("f03322342_gles_texture_image_test", PROBE)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module


MAP = load()


class TextureImageCopySubimageTests(unittest.TestCase):
    def live(self) -> Path:
        if not MAP.CACHE.cache_file(CACHE, MAP.CACHE.SOURCE).is_file(): self.fail("retained external F03.3.2.1 cache is unavailable")
        return CACHE

    def rehash(self, value: dict[str, object]) -> None:
        value["raw_entries_sha256"] = hashlib.sha256(MAP.canonical(value["raw_entries"])).hexdigest()
        body = {key: item for key, item in value.items() if key != "inventory_sha256"}; value["inventory_sha256"] = hashlib.sha256(MAP.canonical(body)).hexdigest()

    def test_exact_bound_image_copy_and_subimage_slice(self) -> None:
        value = MAP.validate(self.live()); entries = MAP.records(value)
        expected = ["TexImage3D", "TexImage2D", "CopyTexImage2D", "TexSubImage3D", "TexSubImage2D", "CopyTexSubImage3D", "CopyTexSubImage2D"]
        self.assertEqual((value["profile"], value["source_class"], value["raw_entry_count"]), ("gles-3.2", "command-state", 7))
        self.assertEqual(value["domain_families"], [{"id": "texture", "source_order": 17}])
        self.assertEqual([row["unprefixed_name"] for row in entries], expected); self.assertEqual([row["c_name"] for row in entries], ["gl" + name for name in expected])
        self.assertEqual([row["source_page_span"] for row in entries], [[175], [184], [185, 187], [191, 192], [192], [192], [192]])
        self.assertEqual([(row["physical_page"], row["section"]) for row in entries[2:4]], [(185, "8.6"), (191, "8.6")])
        self.assertTrue(value["raw_only"]); self.assertFalse(value["promotion_allowed"])

    def test_rejects_discontinuous_fragment_gap_and_source_order_drift(self) -> None:
        changed = list(MAP.CATALOG.SPLIT_FRAGMENTS); name, pieces = changed[0]; changed[0] = (name, (pieces[0], (187, "enum internalformat, int x, int y, sizei width, sizei height, int border, int fabricated );")))
        with patch.object(MAP.CATALOG, "SPLIT_FRAGMENTS", tuple(changed)), patch.object(MAP.CATALOG, "SEALED_FRAGMENTS_SHA256", MAP.CATALOG.sha256(tuple(changed))), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.CATALOG, "GAP_WITNESS", (185, 186, 187, "not-the-figure")), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.CATALOG, "SOURCE_PAGES", (175, 184, 185, 187, 191, 192)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        for changed in (tuple(MAP.CATALOG.FORMS[:-1]), tuple(reversed(MAP.CATALOG.FORMS))):
            with patch.object(MAP.CATALOG, "FORMS", changed), patch.object(MAP.CATALOG, "SEALED_FORMS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        forms = list(MAP.CATALOG.FORMS); forms[2] = ((185, 186), *forms[2][1:]); changed = tuple(forms)
        with patch.object(MAP.CATALOG, "FORMS", changed), patch.object(MAP.CATALOG, "SEALED_FORMS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.GRAMMAR, "normalize", lambda forms: ["glTexImage3D"]), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rejects_cross_family_and_domain_route_drift(self) -> None:
        forms = list(MAP.CATALOG.FORMS); forms[0] = ((195,), "8.7", "CompressedTexImage2D", "void CompressedTexImage2D( enum target, int level, enum internalformat, sizei width, sizei height, int border, sizei imageSize, const void *data );"); changed = tuple(forms)
        with patch.object(MAP.CATALOG, "FORMS", changed), patch.object(MAP.CATALOG, "SEALED_FORMS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        family = list(MAP.CATALOG.FAMILY); family[3] = 18
        with patch.object(MAP.CATALOG, "FAMILY", tuple(family)), self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)
        with patch.object(MAP.CATALOG, "ROUTE", "wrong-route"), self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)

    def test_rehashed_promoted_span_duplicate_and_symlinked_artifacts_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, item_file = Path(temporary), Path(temporary) / "inventory.json"
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["raw_entries"][2][5] = [185, 186]; self.rehash(value); item_file.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["profile_support"] = True; self.rehash(value); item_file.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["pixel_layout"] = "fabricated"; self.rehash(value); item_file.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            item_file.write_text('{"schema":1,"schema":2}', encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            item_file.write_text('{"schema":NaN}', encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            link = root / "inventory-link.json"; link.symlink_to(item_file)
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), link)

    def test_unavailable_substitutes_and_private_dependencies_fail_closed(self) -> None:
        for identifier, candidate in (("shader", "essl-320-spec"), ("precision", "gl.xml"), ("extension", "desktop-glsl")):
            with self.subTest(identifier=identifier), self.assertRaisesRegex(MAP.InventoryError, "separate F02 admission"): MAP.reject_unadmitted(identifier, candidate)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); bad = root / "bad.py"; bad.write_text("__file__ = 'unexpected.py'\n", encoding="utf-8"); link = root / "link.py"; link.symlink_to(bad)
            with self.assertRaises(MAP.InventoryError): MAP.private(link, "f03322342_bad_link")
            with self.assertRaises(MAP.InventoryError): MAP.private(bad, "f03322342_bad_file")
        with tempfile.TemporaryDirectory() as temporary, self.assertRaises(MAP.InventoryError): MAP.validate(Path(temporary))

    def test_fixed_loaders_ignore_and_restore_ambient_aliases(self) -> None:
        aliases = ("f03322342_gles_texture_image_catalog", "f03322342_gles_artifact", "f03322342_gles_domain", "f03322342_gles_grammar", "f03322342_gles_cache", "f03322342_gles_ledger")
        code = "\n".join(("import importlib.util, sys, types", "from pathlib import Path", f"aliases = {aliases!r}", "decoys = {name: types.ModuleType(name) for name in aliases}; sys.modules.update(decoys)", f"spec = importlib.util.spec_from_file_location('probe', {str(PROBE)!r})", "module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)", f"assert module.validate(Path({str(CACHE)!r}))['raw_entry_count'] == 7", "assert all(sys.modules[name] is decoys[name] for name in aliases)"))
        result = subprocess.run([sys.executable, "-B", "-c", code], cwd=tempfile.gettempdir(), text=True, capture_output=True, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        self.assertEqual(result.returncode, 0, msg=result.stderr)


if __name__ == "__main__": unittest.main()
