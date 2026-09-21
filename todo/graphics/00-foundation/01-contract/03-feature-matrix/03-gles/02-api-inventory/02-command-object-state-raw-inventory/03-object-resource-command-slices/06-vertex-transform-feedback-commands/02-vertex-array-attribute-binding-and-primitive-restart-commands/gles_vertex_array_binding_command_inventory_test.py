#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.2.3.6.2."""

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
PROBE = HERE / "gles_vertex_array_binding_command_inventory.py"


def load():
    spec = importlib.util.spec_from_file_location("f03322362_gles_vertex_array_test", PROBE)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module


MAP = load()


class VertexArrayBindingCommandTests(unittest.TestCase):
    def live(self) -> Path:
        if not MAP.CACHE.cache_file(CACHE, MAP.CACHE.SOURCE).is_file(): self.fail("retained external F03.3.2.1 cache is unavailable")
        return CACHE

    def rehash(self, value: dict[str, object]) -> None:
        value["raw_entries_sha256"] = hashlib.sha256(MAP.canonical(value["raw_entries"])).hexdigest()
        body = {key: item for key, item in value.items() if key != "inventory_sha256"}
        value["inventory_sha256"] = hashlib.sha256(MAP.canonical(body)).hexdigest()

    def test_exact_bound_vertex_array_binding_forms(self) -> None:
        value, entries = MAP.validate(self.live()), None
        entries = MAP.records(value); expected = [row[2] for row in MAP.CATALOG.DECLARATIONS]
        locations = [(row[0], row[1]) for row in MAP.CATALOG.DECLARATIONS]
        self.assertEqual((value["profile"], value["source_class"], value["raw_entry_count"]), ("gles-3.2", "command-state", 12))
        self.assertEqual(value["domain_families"], [{"id": "vertex-array", "source_order": 22}])
        self.assertEqual([row["unprefixed_name"] for row in entries], expected)
        self.assertEqual([row["c_name"] for row in entries], ["gl" + name for name in expected])
        self.assertEqual([(row["physical_page"], row["section"]) for row in entries], locations)
        self.assertEqual((entries[10]["unprefixed_name"], entries[10]["section"]), ("Enable", "10.3.4"))
        self.assertTrue(value["raw_only"]); self.assertFalse(value["promotion_allowed"])

    def test_rejects_changed_window_cross_family_and_normalizer_drift(self) -> None:
        windows = (tuple(MAP.CATALOG.DECLARATIONS[:-1]), tuple(reversed(MAP.CATALOG.DECLARATIONS)))
        for changed in windows:
            with self.subTest(changed=changed), patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        changed = list(MAP.CATALOG.DECLARATIONS); changed[0] = (285, "10.3.1", "VertexAttrib1f", "void VertexAttrib1f( uint index, float x );"); changed = tuple(changed)
        with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.GRAMMAR, "normalize", lambda forms: ["gl" + forms[0][1] + "Unexpected"]), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rejects_section_and_primitive_restart_fences(self) -> None:
        for index, section in ((6, "10.3.2"), (8, "10.3.1")):
            changed = list(MAP.CATALOG.DECLARATIONS); page, _, name, declaration = changed[index]; changed[index] = (page, section, name, declaration); changed = tuple(changed)
            with self.subTest(index=index), patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        original = MAP.CATALOG.page_text
        for page, marker in ((289, "10.3.2 Vertex Attribute Divisors"), (290, "10.3.3 Transferring Array Elements"), (291, "10.3.5 Robust Buffer Access"), (291, "PRIMITIVE_RESTART_FIXED_INDEX")):
            def missing(raw, item_page, page=page, marker=marker): return original(raw, item_page).replace(marker, "missing marker") if item_page == page else original(raw, item_page)
            with self.subTest(page=page, marker=marker), patch.object(MAP.CATALOG, "page_text", missing), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rejects_family_route_profile_and_locator_drift(self) -> None:
        family = list(MAP.CATALOG.FAMILY); family[3] = 21
        with patch.object(MAP.CATALOG, "FAMILY", tuple(family)), self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)
        with patch.object(MAP.CATALOG, "ROUTE", "wrong-route"), self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)
        with patch.object(MAP.CATALOG, "PROFILE", "gles-3.1"), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        original_receipt = MAP.CACHE.inspect
        def relocated_receipt(*args, **kwargs):
            value = original_receipt(*args, **kwargs); value["locator"] = "gles32-pdf-v1:page=1;section=1"; return value
        with patch.object(MAP.CACHE, "inspect", relocated_receipt), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        authority, original_admission = MAP.CACHE.SOURCE_API.authority(), None
        original_admission = authority.consume
        def relocated_admission(*args, **kwargs):
            value = original_admission(*args, **kwargs); value["locator"] = "gles32-pdf-v1:page=1;section=1"; return value
        with patch.object(authority, "consume", relocated_admission), patch.object(MAP.CACHE.SOURCE_API, "authority", lambda: authority):
            with self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rehashed_promoted_duplicate_and_symlinked_artifacts_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, item_file = Path(temporary), Path(temporary) / "inventory.json"
            for key, item in (("reverse", None), ("profile_support", True), ("primitive_restart_state", "fabricated"), ("raw_only", False)):
                value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8"))
                if key == "reverse": value["raw_entries"].reverse()
                else: value[key] = item
                self.rehash(value); item_file.write_text(json.dumps(value), encoding="utf-8")
                with self.subTest(key=key), self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            item_file.write_text('{"schema":1,"schema":2}', encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            item_file.write_text('{"schema":NaN}', encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            link = root / "inventory-link.json"; link.symlink_to(item_file)
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), link)

    def test_unavailable_substitutes_and_private_dependencies_fail_closed(self) -> None:
        for identifier, candidate in (("shader", "essl-320-spec"), ("precision", "gl.xml"), ("extension", "desktop-glsl")):
            with self.subTest(identifier=identifier), self.assertRaisesRegex(MAP.InventoryError, "separate F02 admission"):
                MAP.reject_unadmitted(identifier, candidate)
        with tempfile.TemporaryDirectory() as temporary:
            root, bad = Path(temporary), Path(temporary) / "bad.py"; bad.write_text("__file__ = 'unexpected.py'\n", encoding="utf-8")
            link = root / "link.py"; link.symlink_to(bad)
            with self.assertRaises(MAP.InventoryError): MAP.private(link, "f03322362_bad_link")
            with self.assertRaises(MAP.InventoryError): MAP.private(bad, "f03322362_bad_file")
        with tempfile.TemporaryDirectory() as temporary, self.assertRaises(MAP.InventoryError): MAP.validate(Path(temporary))

    def test_fixed_loaders_ignore_aliases_and_ambient_path_extractor(self) -> None:
        aliases = ("f03322362_gles_vertex_array_catalog", "f03322362_gles_artifact", "f03322362_gles_domain", "f03322362_gles_grammar", "f03322362_gles_cache", "f03322362_gles_ledger")
        code = "\n".join(("import importlib.util, sys, types", "from pathlib import Path", f"aliases = {aliases!r}",
            "decoys = {name: types.ModuleType(name) for name in aliases}; sys.modules.update(decoys)",
            f"spec = importlib.util.spec_from_file_location('probe', {str(PROBE)!r})", "module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
            f"assert module.validate(Path({str(CACHE)!r}))['raw_entry_count'] == 12", "assert all(sys.modules[name] is decoys[name] for name in aliases)"))
        result = subprocess.run([sys.executable, "-B", "-c", code], cwd=tempfile.gettempdir(), text=True, capture_output=True, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        with tempfile.TemporaryDirectory() as temporary:
            fake = Path(temporary) / "pdftotext"; fake.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8"); fake.chmod(0o755)
            raw = MAP.CACHE.pdf_bytes(MAP.CACHE.external_root(self.live()), MAP.CACHE.SOURCE)
            with patch.dict(os.environ, {"PATH": str(fake.parent)}, clear=False): self.assertIn(MAP.CATALOG.DECLARATIONS[0][3], MAP.CATALOG.page_text(raw, 285))


if __name__ == "__main__": unittest.main()
