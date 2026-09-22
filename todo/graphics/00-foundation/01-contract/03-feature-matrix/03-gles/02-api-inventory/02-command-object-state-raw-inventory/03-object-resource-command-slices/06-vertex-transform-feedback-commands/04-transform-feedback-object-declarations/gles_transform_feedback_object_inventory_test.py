#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.2.3.6.4."""

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
PROBE = HERE / "gles_transform_feedback_object_inventory.py"


def load():
    spec = importlib.util.spec_from_file_location("f03322364_gles_transform_feedback_object_test", PROBE)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module


MAP = load()


class TransformFeedbackObjectTests(unittest.TestCase):
    def live(self) -> Path:
        if not MAP.CACHE.cache_file(CACHE, MAP.CACHE.SOURCE).is_file(): self.fail("retained external F03.3.2.1 cache is unavailable")
        return CACHE

    def rehash(self, value: dict[str, object]) -> None:
        value["raw_entries_sha256"] = hashlib.sha256(MAP.canonical(value["raw_entries"])).hexdigest()
        body = {key: item for key, item in value.items() if key != "inventory_sha256"}
        value["inventory_sha256"] = hashlib.sha256(MAP.canonical(body)).hexdigest()

    def test_exact_bound_transform_feedback_object_forms(self) -> None:
        value, entries = MAP.validate(self.live()), None
        entries = MAP.records(value); expected = [row[2] for row in MAP.CATALOG.DECLARATIONS]
        self.assertEqual((value["profile"], value["source_class"], value["raw_entry_count"]), ("gles-3.2", "command-state", 4))
        self.assertEqual(value["domain_families"], [{"id": "transform-feedback", "source_order": 26}])
        self.assertEqual([row["unprefixed_name"] for row in entries], expected)
        self.assertEqual([row["c_name"] for row in entries], ["gl" + name for name in expected])
        self.assertEqual([(row["physical_page"], row["section"]) for row in entries], [(354, "12.2.1"), (355, "12.2.1"), (355, "12.2.1"), (356, "12.2.1")])
        self.assertEqual(entries[2]["declaration"], "boolean IsTransformFeedback( uint id );")
        self.assertTrue(value["raw_only"]); self.assertFalse(value["promotion_allowed"])

    def test_rejects_changed_window_cross_family_return_const_and_normalizer_drift(self) -> None:
        windows = (tuple(MAP.CATALOG.DECLARATIONS[:-1]), tuple(reversed(MAP.CATALOG.DECLARATIONS)))
        for changed in windows:
            with self.subTest(changed=changed), patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        changed = list(MAP.CATALOG.DECLARATIONS); changed[0] = (354, "12.2.1", "BeginTransformFeedback", "void BeginTransformFeedback( enum primitiveMode );"); changed = tuple(changed)
        with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        changed = list(MAP.CATALOG.DECLARATIONS); changed[1] = (355, "12.2.1", "DeleteTransformFeedbacks", "void DeleteTransformFeedbacks( sizei n, uint *ids );"); changed = tuple(changed)
        with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        changed = list(MAP.CATALOG.DECLARATIONS); changed[2] = (355, "12.2.1", "IsTransformFeedback", "void IsTransformFeedback( uint id );"); changed = tuple(changed)
        with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.GRAMMAR, "normalize", lambda forms: ["gl" + forms[0][1] + "Unexpected"]), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rejects_section_fences_and_heading_drift(self) -> None:
        original = MAP.CATALOG.source_slice
        with patch.object(MAP.CATALOG, "source_slice", lambda text: original(text) + ((357, "void BeginTransformFeedback( enum primitiveMode );"),)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.CATALOG, "SOURCE_PAGES", (354, 355, 356, 357)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        page_text = MAP.CATALOG.page_text
        def before_heading(raw, page):
            value = page_text(raw, page)
            if page != 354: return value
            declaration, heading = MAP.CATALOG.DECLARATIONS[0][3], "12.2.1 Transform Feedback Objects"
            return value.replace(declaration, "").replace(heading, declaration + " " + heading)
        with patch.object(MAP.CATALOG, "page_text", before_heading), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        def missing_marker(raw, page):
            marker = "12.2.1 Transform Feedback Objects" if page == 354 else "12.2.2 Transform Feedback Primitive Capture"
            return page_text(raw, page).replace(marker, "missing marker")
        for page in (354, 357):
            with self.subTest(page=page), patch.object(MAP.CATALOG, "page_text", missing_marker), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rejects_family_route_profile_and_locator_drift(self) -> None:
        family = list(MAP.CATALOG.FAMILY); family[3] = 25
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
            for key, item in (("reverse", None), ("profile_support", True), ("transform_feedback_state", "fabricated"), ("raw_only", False)):
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
            with self.assertRaises(MAP.InventoryError): MAP.private(link, "f03322364_bad_link")
            with self.assertRaises(MAP.InventoryError): MAP.private(bad, "f03322364_bad_file")
        with tempfile.TemporaryDirectory() as temporary, self.assertRaises(MAP.InventoryError): MAP.validate(Path(temporary))

    def test_fixed_loaders_ignore_aliases_and_ambient_path_extractor(self) -> None:
        aliases = ("f03322364_gles_transform_feedback_object_catalog", "f03322364_gles_artifact", "f03322364_gles_domain", "f03322364_gles_grammar", "f03322364_gles_cache", "f03322364_gles_ledger")
        code = "\n".join(("import importlib.util, sys, types", "from pathlib import Path", f"aliases = {aliases!r}",
            "decoys = {name: types.ModuleType(name) for name in aliases}; sys.modules.update(decoys)",
            f"spec = importlib.util.spec_from_file_location('probe', {str(PROBE)!r})", "module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
            f"assert module.validate(Path({str(CACHE)!r}))['raw_entry_count'] == 4", "assert all(sys.modules[name] is decoys[name] for name in aliases)"))
        result = subprocess.run([sys.executable, "-B", "-c", code], cwd=tempfile.gettempdir(), text=True, capture_output=True, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        with tempfile.TemporaryDirectory() as temporary:
            fake = Path(temporary) / "pdftotext"; fake.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8"); fake.chmod(0o755)
            raw = MAP.CACHE.pdf_bytes(MAP.CACHE.external_root(self.live()), MAP.CACHE.SOURCE)
            with patch.dict(os.environ, {"PATH": str(fake.parent)}, clear=False): self.assertIn(MAP.CATALOG.DECLARATIONS[0][3], MAP.CATALOG.page_text(raw, 354))


if __name__ == "__main__": unittest.main()
