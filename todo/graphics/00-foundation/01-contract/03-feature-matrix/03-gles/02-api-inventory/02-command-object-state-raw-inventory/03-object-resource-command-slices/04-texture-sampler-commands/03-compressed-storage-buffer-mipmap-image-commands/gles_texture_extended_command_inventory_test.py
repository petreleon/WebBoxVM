#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.2.3.4.3."""

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
PROBE = HERE / "gles_texture_extended_command_inventory.py"


def load():
    spec = importlib.util.spec_from_file_location("f03322343_gles_texture_extended_test", PROBE)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module


MAP = load()


class TextureExtendedCommandTests(unittest.TestCase):
    def live(self) -> Path:
        if not MAP.CACHE.cache_file(CACHE, MAP.CACHE.SOURCE).is_file(): self.fail("retained external F03.3.2.1 cache is unavailable")
        return CACHE

    def rehash(self, value: dict[str, object]) -> None:
        value["raw_entries_sha256"] = hashlib.sha256(MAP.canonical(value["raw_entries"])).hexdigest()
        body = {key: item for key, item in value.items() if key != "inventory_sha256"}; value["inventory_sha256"] = hashlib.sha256(MAP.canonical(body)).hexdigest()

    def test_exact_bound_extended_texture_slice(self) -> None:
        value = MAP.validate(self.live()); entries = MAP.records(value)
        expected = ["CompressedTexImage2D", "CompressedTexImage3D", "CompressedTexSubImage2D", "CompressedTexSubImage3D", "TexStorage2DMultisample", "TexStorage3DMultisample", "TexBufferRange", "TexBuffer", "GenerateMipmap", "TexStorage2D", "TexStorage3D", "BindImageTexture"]
        self.assertEqual((value["profile"], value["source_class"], value["raw_entry_count"]), ("gles-3.2", "command-state", 12))
        self.assertEqual(value["domain_families"], [{"id": "texture", "source_order": 17}])
        self.assertEqual([row["unprefixed_name"] for row in entries], expected); self.assertEqual([row["c_name"] for row in entries], ["gl" + name for name in expected])
        self.assertEqual([(row["physical_page"], row["section"]) for row in entries[8:]], [(222, "8.14.4"), (226, "8.18"), (227, "8.18"), (233, "8.23")])
        self.assertTrue(value["raw_only"]); self.assertFalse(value["promotion_allowed"])

    def test_rejects_changed_set_order_scope_and_normalization_drift(self) -> None:
        forms = list(MAP.CATALOG.DECLARATIONS); forms[0] = (184, "8.5", "TexImage2D", "void TexImage2D( enum target, int level, int internalformat, sizei width, sizei height, int border, enum format, enum type, const void *data );"); changed = tuple(forms)
        with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        for changed in (tuple(MAP.CATALOG.DECLARATIONS[:-1]), tuple(reversed(MAP.CATALOG.DECLARATIONS))):
            with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.CATALOG, "SOURCE_PAGES", (195, 199, 201, 203, 204, 222, 226, 233)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.CATALOG, "SECTION_WITNESSES", {}), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.GRAMMAR, "normalize", lambda forms: ["glCompressedTexImage2D"]), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rejects_cross_route_and_domain_order_drift(self) -> None:
        family = list(MAP.CATALOG.FAMILY); family[3] = 18
        with patch.object(MAP.CATALOG, "FAMILY", tuple(family)), self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)
        with patch.object(MAP.CATALOG, "ROUTE", "wrong-route"), self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)

    def test_rehashed_promoted_semantic_duplicate_and_symlinked_artifacts_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, item_file = Path(temporary), Path(temporary) / "inventory.json"
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["raw_entries"].reverse(); self.rehash(value); item_file.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["profile_support"] = True; self.rehash(value); item_file.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["storage_behavior"] = "fabricated"; self.rehash(value); item_file.write_text(json.dumps(value), encoding="utf-8")
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
            with self.assertRaises(MAP.InventoryError): MAP.private(link, "f03322343_bad_link")
            with self.assertRaises(MAP.InventoryError): MAP.private(bad, "f03322343_bad_file")
        with tempfile.TemporaryDirectory() as temporary, self.assertRaises(MAP.InventoryError): MAP.validate(Path(temporary))

    def test_fixed_loaders_ignore_and_restore_ambient_aliases(self) -> None:
        aliases = ("f03322343_gles_texture_extended_catalog", "f03322343_gles_artifact", "f03322343_gles_domain", "f03322343_gles_grammar", "f03322343_gles_cache", "f03322343_gles_ledger")
        code = "\n".join(("import importlib.util, sys, types", "from pathlib import Path", f"aliases = {aliases!r}", "decoys = {name: types.ModuleType(name) for name in aliases}; sys.modules.update(decoys)", f"spec = importlib.util.spec_from_file_location('probe', {str(PROBE)!r})", "module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)", f"assert module.validate(Path({str(CACHE)!r}))['raw_entry_count'] == 12", "assert all(sys.modules[name] is decoys[name] for name in aliases)"))
        result = subprocess.run([sys.executable, "-B", "-c", code], cwd=tempfile.gettempdir(), text=True, capture_output=True, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        self.assertEqual(result.returncode, 0, msg=result.stderr)


if __name__ == "__main__": unittest.main()
