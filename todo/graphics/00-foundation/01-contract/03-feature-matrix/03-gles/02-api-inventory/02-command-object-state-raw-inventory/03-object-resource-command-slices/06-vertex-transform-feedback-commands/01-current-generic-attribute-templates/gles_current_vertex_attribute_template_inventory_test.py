#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.2.3.6.1."""

from __future__ import annotations

import copy
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
PROBE = HERE / "gles_current_vertex_attribute_template_inventory.py"


def load():
    spec = importlib.util.spec_from_file_location("f03322361_gles_current_vertex_test", PROBE)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


MAP = load()


class CurrentVertexAttributeTemplateTests(unittest.TestCase):
    def live(self) -> Path:
        if not MAP.CACHE.cache_file(CACHE, MAP.CACHE.SOURCE).is_file(): self.fail("retained external F03.3.2.1 cache is unavailable")
        return CACHE

    def rehash(self, value: dict[str, object]) -> None:
        value["raw_entries_sha256"] = hashlib.sha256(MAP.canonical(value["raw_entries"])).hexdigest()
        body = {key: item for key, item in value.items() if key != "inventory_sha256"}
        value["inventory_sha256"] = hashlib.sha256(MAP.canonical(body)).hexdigest()

    def test_exact_bound_template_slice(self) -> None:
        value, entries = MAP.validate(self.live()), None
        entries = MAP.records(value); expected = [name for group in MAP.CATALOG.EXPANDED_NAMES for name in group]
        self.assertEqual((value["profile"], value["source_class"], value["raw_entry_count"]), ("gles-3.2", "command-state", 12))
        self.assertEqual(value["domain_families"], [{"id": "vertex-array", "source_order": 22}])
        self.assertEqual([row["unprefixed_name"] for row in entries], expected)
        self.assertEqual([row["c_name"] for row in entries], ["gl" + name for name in expected])
        self.assertEqual([(row["physical_page"], row["section"]) for row in entries], [(283, "10.2.1")] * 12)
        self.assertEqual([row["declaration"] for row in entries], sum(([item] * len(names) for _, item, names in MAP.CATALOG.TEMPLATES), []))
        self.assertTrue(value["raw_only"]); self.assertFalse(value["promotion_allowed"])

    def test_rejects_changed_template_window_source_and_normalizer(self) -> None:
        changed = tuple(reversed(MAP.CATALOG.FORMAL_WINDOW))
        with patch.object(MAP.CATALOG, "FORMAL_WINDOW", changed), patch.object(MAP.CATALOG, "SEALED_WINDOW_SHA256", MAP.CATALOG.template_sha256(changed)):
            with self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        templates = list(MAP.CATALOG.TEMPLATES); templates[2] = (25, "void VertexAttrib{5}f( uint index,float values );", ("VertexAttrib5f",))
        with patch.object(MAP.CATALOG, "TEMPLATES", tuple(templates)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        original = MAP.CATALOG.page_text
        def replaced(raw, page): return original(raw, page).replace("VertexAttrib{1234}f", "VertexAttrib{1235}f")
        with patch.object(MAP.CATALOG, "page_text", replaced), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.GRAMMAR, "normalize", lambda forms: ["glVertexAttribI4i"]), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rejects_heading_page_grammar_and_locator_drift(self) -> None:
        original = MAP.CATALOG.page_text
        def missing_heading(raw, page): return original(raw, page).replace(MAP.CATALOG.SECTION_WITNESS, "missing heading")
        with patch.object(MAP.CATALOG, "page_text", missing_heading), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.CATALOG, "SOURCE_PAGE", 282), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        grammar = copy.deepcopy(MAP.GRAMMAR.validate(self.live())); grammar["grammar"]["formal_templates"][22]["source_order"] = 99
        with self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.grammar_window(grammar)
        grammar = copy.deepcopy(MAP.GRAMMAR.validate(self.live())); grammar["grammar"]["formal_templates"][25]["expansion_count"] = 3
        with self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.grammar_window(grammar)
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

    def test_rejects_family_route_and_profile_drift(self) -> None:
        family = list(MAP.CATALOG.FAMILY); family[3] = 21
        with patch.object(MAP.CATALOG, "FAMILY", tuple(family)), self.assertRaises(MAP.CATALOG.CatalogError):
            MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)
        with patch.object(MAP.CATALOG, "ROUTE", "wrong-route"), self.assertRaises(MAP.CATALOG.CatalogError):
            MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)
        with patch.object(MAP.CATALOG, "PROFILE", "gles-3.1"), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rehashed_promoted_duplicate_and_symlinked_artifacts_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, item_file = Path(temporary), Path(temporary) / "inventory.json"
            for key, item in (("reverse", None), ("profile_support", True), ("current_attribute_value", "fabricated"), ("raw_only", False)):
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
            with self.assertRaises(MAP.InventoryError): MAP.private(link, "f03322361_bad_link")
            with self.assertRaises(MAP.InventoryError): MAP.private(bad, "f03322361_bad_file")
        with tempfile.TemporaryDirectory() as temporary, self.assertRaises(MAP.InventoryError): MAP.validate(Path(temporary))

    def test_fixed_loaders_ignore_and_restore_ambient_aliases(self) -> None:
        aliases = ("f03322361_gles_current_vertex_catalog", "f03322361_gles_artifact", "f03322361_gles_domain", "f03322361_gles_grammar", "f03322361_gles_cache", "f03322361_gles_ledger")
        code = "\n".join(("import importlib.util, sys, types", "from pathlib import Path", f"aliases = {aliases!r}",
            "decoys = {name: types.ModuleType(name) for name in aliases}; sys.modules.update(decoys)",
            f"spec = importlib.util.spec_from_file_location('probe', {str(PROBE)!r})", "module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
            f"assert module.validate(Path({str(CACHE)!r}))['raw_entry_count'] == 12", "assert all(sys.modules[name] is decoys[name] for name in aliases)"))
        result = subprocess.run([sys.executable, "-B", "-c", code], cwd=tempfile.gettempdir(), text=True, capture_output=True, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        with tempfile.TemporaryDirectory() as temporary:
            fake = Path(temporary) / "pdftotext"; fake.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8"); fake.chmod(0o755)
            raw = MAP.CACHE.pdf_bytes(MAP.CACHE.external_root(self.live()), MAP.CACHE.SOURCE)
            with patch.dict(os.environ, {"PATH": str(fake.parent)}, clear=False): self.assertIn(MAP.CATALOG.FORMAL_WINDOW[0], MAP.CATALOG.page_text(raw, 283))


if __name__ == "__main__": unittest.main()
