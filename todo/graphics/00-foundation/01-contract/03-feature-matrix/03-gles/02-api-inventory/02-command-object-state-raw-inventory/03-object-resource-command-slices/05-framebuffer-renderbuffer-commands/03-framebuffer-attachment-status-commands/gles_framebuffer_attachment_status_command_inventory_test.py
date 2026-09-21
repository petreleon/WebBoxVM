#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.2.3.5.3."""

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
PROBE = HERE / "gles_framebuffer_attachment_status_command_inventory.py"


def load():
    spec = importlib.util.spec_from_file_location("f03322353_gles_framebuffer_attachment_status_test", PROBE)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module


MAP = load()


class FramebufferAttachmentStatusCommandTests(unittest.TestCase):
    def live(self) -> Path:
        if not MAP.CACHE.cache_file(CACHE, MAP.CACHE.SOURCE).is_file(): self.fail("retained external F03.3.2.1 cache is unavailable")
        return CACHE

    def rehash(self, value: dict[str, object]) -> None:
        value["raw_entries_sha256"] = hashlib.sha256(MAP.canonical(value["raw_entries"])).hexdigest()
        body = {key: item for key, item in value.items() if key != "inventory_sha256"}; value["inventory_sha256"] = hashlib.sha256(MAP.canonical(body)).hexdigest()

    def test_exact_bound_framebuffer_attachment_status_forms(self) -> None:
        value = MAP.validate(self.live()); entries = MAP.records(value)
        expected = ["FramebufferRenderbuffer", "FramebufferTexture", "FramebufferTexture2D", "FramebufferTextureLayer", "CheckFramebufferStatus"]
        self.assertEqual((value["profile"], value["source_class"], value["raw_entry_count"]), ("gles-3.2", "command-state", 5))
        self.assertEqual(value["domain_families"], [{"id": "framebuffer-renderbuffer", "source_order": 20}])
        self.assertEqual([row["unprefixed_name"] for row in entries], expected); self.assertEqual([row["c_name"] for row in entries], ["gl" + name for name in expected])
        self.assertEqual([(row["physical_page"], row["section"]) for row in entries], [(256, "9.2.7"), (258, "9.2.8"), (259, "9.2.8"), (260, "9.2.8"), (269, "9.4.2")])
        self.assertEqual(entries[4]["declaration"], "enum CheckFramebufferStatus( enum target );")
        self.assertTrue(value["raw_only"]); self.assertFalse(value["promotion_allowed"])

    def test_rejects_changed_window_renderbuffer_intrusion_and_normalizer_drift(self) -> None:
        windows = (tuple(MAP.CATALOG.DECLARATIONS[:-1]), tuple(reversed(MAP.CATALOG.DECLARATIONS)))
        for changed in windows:
            with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        changed = list(MAP.CATALOG.DECLARATIONS); changed[0] = (256, "9.2.7", "GetRenderbufferParameteriv", "void GetRenderbufferParameteriv( enum target, enum pname, int *params );"); changed = tuple(changed)
        with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.CATALOG, "SOURCE_PAGES", (256, 258, 259, 260)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.GRAMMAR, "normalize", lambda forms: ["gl" + forms[0][1] + "Unexpected"]), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rejects_fragment_marker_loss_and_pre_marker_intrusion(self) -> None:
        original_text = MAP.CATALOG.page_text
        for page, (_, marker) in MAP.CATALOG.PAGE_FRAGMENTS.items():
            def missing_marker(raw: bytes, source_page: int, target=page, text=marker) -> str:
                value = original_text(raw, source_page)
                return value.replace(text, "source fragment absent") if source_page == target else value
            with self.subTest(page=page), patch.object(MAP.CATALOG, "page_text", missing_marker), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        original_fragment = MAP.CATALOG.page_fragment
        def leaking_fragment(page: int, value: str) -> str:
            return value if page == 256 else original_fragment(page, value)
        with patch.object(MAP.CATALOG, "page_fragment", leaking_fragment), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        marker = "9.2.8 Attaching Texture Images to a Framebuffer"
        def injected_p258(raw: bytes, page: int) -> str:
            value = original_text(raw, page)
            return value.replace(marker, "void FramebufferTexture3D( enum target ); " + marker) if page == 258 else value
        def leaking_p258(page: int, value: str) -> str:
            return value if page == 258 else original_fragment(page, value)
        with patch.object(MAP.CATALOG, "page_text", injected_p258), patch.object(MAP.CATALOG, "page_fragment", leaking_p258), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rejects_section_witness_return_profile_route_and_family_drift(self) -> None:
        for index, section in ((0, "9.2.8"), (1, "9.2.7"), (4, "9.4")):
            changed = list(MAP.CATALOG.DECLARATIONS); page, _, name, declaration = changed[index]; changed[index] = (page, section, name, declaration); changed = tuple(changed)
            with self.subTest(name=name, section=section), patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        changed = list(MAP.CATALOG.DECLARATIONS); changed[4] = (269, "9.4.2", "CheckFramebufferStatus", "void CheckFramebufferStatus( enum target );"); changed = tuple(changed)
        with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        original = MAP.CATALOG.page_text
        def missing_witness(raw: bytes, page: int) -> str:
            value = original(raw, page); return value.replace("9.4.2 Whole Framebuffer Completeness", "source witness absent") if page == 267 else value
        with patch.object(MAP.CATALOG, "page_text", missing_witness), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        family = list(MAP.CATALOG.FAMILY); family[3] = 21
        with patch.object(MAP.CATALOG, "FAMILY", tuple(family)), self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)
        with patch.object(MAP.CATALOG, "ROUTE", "wrong-route"), self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)
        with patch.object(MAP.CATALOG, "PROFILE", "gles-3.1"), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rehashed_promoted_malformed_and_symlinked_artifacts_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, item_file = Path(temporary), Path(temporary) / "inventory.json"
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["raw_entries"].reverse(); self.rehash(value); item_file.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["profile_support"] = True; self.rehash(value); item_file.write_text(json.dumps(value), encoding="utf-8")
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
            with self.assertRaises(MAP.InventoryError): MAP.private(link, "f03322353_bad_link")
            with self.assertRaises(MAP.InventoryError): MAP.private(bad, "f03322353_bad_file")
        with tempfile.TemporaryDirectory() as temporary, self.assertRaises(MAP.InventoryError): MAP.validate(Path(temporary))

    def test_fixed_loaders_ignore_and_restore_ambient_aliases(self) -> None:
        aliases = ("f03322353_gles_framebuffer_attachment_status_catalog", "f03322353_gles_artifact", "f03322353_gles_domain", "f03322353_gles_grammar", "f03322353_gles_cache", "f03322353_gles_ledger")
        code = "\n".join(("import importlib.util, sys, types", "from pathlib import Path", f"aliases = {aliases!r}", "decoys = {name: types.ModuleType(name) for name in aliases}; sys.modules.update(decoys)", f"spec = importlib.util.spec_from_file_location('probe', {str(PROBE)!r})", "module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)", f"assert module.validate(Path({str(CACHE)!r}))['raw_entry_count'] == 5", "assert all(sys.modules[name] is decoys[name] for name in aliases)"))
        result = subprocess.run([sys.executable, "-B", "-c", code], cwd=tempfile.gettempdir(), text=True, capture_output=True, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        self.assertEqual(result.returncode, 0, msg=result.stderr)


if __name__ == "__main__": unittest.main()
