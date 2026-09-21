#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.2.3.4.1."""

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
PROBE = HERE / "gles_texture_sampler_object_parameter_inventory.py"


def load():
    spec = importlib.util.spec_from_file_location("f03322341_gles_texture_sampler_test", PROBE)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module


MAP = load()


class TextureSamplerObjectParameterTests(unittest.TestCase):
    def live(self) -> Path:
        if not MAP.CACHE.cache_file(CACHE, MAP.CACHE.SOURCE).is_file(): self.fail("retained external F03.3.2.1 cache is unavailable")
        return CACHE

    def rehash(self, value: dict[str, object]) -> None:
        value["raw_entries_sha256"] = hashlib.sha256(MAP.canonical(value["raw_entries"])).hexdigest()
        body = {key: item for key, item in value.items() if key != "inventory_sha256"}
        value["inventory_sha256"] = hashlib.sha256(MAP.canonical(body)).hexdigest()

    def test_exact_bound_object_and_parameter_slice(self) -> None:
        value = MAP.validate(self.live()); entries = MAP.records(value)
        expected = ("GenTextures", "BindTexture", "DeleteTextures", "IsTexture", "GenSamplers", "BindSampler",
                    *MAP.CATALOG.EXPANDED_NAMES, "DeleteSamplers", "IsSampler")
        self.assertEqual((value["profile"], value["source_class"], value["raw_entry_count"]), ("gles-3.2", "command-state", 14))
        self.assertEqual(value["domain_families"], [{"id": "texture", "source_order": 17}, {"id": "sampler", "source_order": 18}])
        self.assertEqual([row["unprefixed_name"] for row in entries], list(expected))
        self.assertEqual([row["c_name"] for row in entries], ["gl" + name for name in expected])
        self.assertEqual([(row["physical_page"], row["section"]) for row in entries[:4]], [(157, "8.1"), (157, "8.1"), (158, "8.1"), (158, "8.1")])
        self.assertEqual(entries[0]["declaration"], "void GenTextures( sizei n, uint *textures );;")
        self.assertEqual([row["derivation_class"] for row in entries[6:12]], ["template-expansion"] * 6)
        self.assertTrue(value["raw_only"]); self.assertFalse(value["promotion_allowed"])

    def test_rejects_source_scope_boundaries_and_grammar_expansion(self) -> None:
        changed = list(MAP.CATALOG.FORMS); changed[0] = (*changed[0][:-1], "void GenTextures( sizei n, uint *textures );")
        with patch.object(MAP.CATALOG, "FORMS", tuple(changed)), patch.object(MAP.CATALOG, "SEALED_FORMS_SHA256", MAP.CATALOG.sha256(tuple(changed))), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        changed = tuple(reversed(MAP.CATALOG.FORMS))
        with patch.object(MAP.CATALOG, "FORMS", changed), patch.object(MAP.CATALOG, "SEALED_FORMS_SHA256", MAP.CATALOG.sha256(changed)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.CATALOG, "SOURCE_PAGES", (156, 157, 158, 159, 160, 161)), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        with patch.object(MAP.CATALOG, "SECTION_BOUNDARIES", {}), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        grammar = copy.deepcopy(MAP.GRAMMAR.validate(self.live())); grammar["grammar"]["formal_templates"][11]["source_order"] = 13
        with self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.grammar_window(grammar)
        grammar = copy.deepcopy(MAP.GRAMMAR.validate(self.live())); grammar["grammar"]["formal_templates"][13]["expansion_count"] = 3
        with self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.grammar_window(grammar)
        with patch.object(MAP.GRAMMAR, "normalize", lambda forms: ["glSamplerParameteri"]), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_rejects_sampler_query_and_domain_cross_route(self) -> None:
        intruder = list(MAP.CATALOG.FORMS); intruder[-1] = ("sampler", 162, "8.3", "template", "GetSamplerParameter{if}v", "void GetSamplerParameter{if}v( uint sampler, enum pname, T *params );")
        with patch.object(MAP.CATALOG, "FORMS", tuple(intruder)), patch.object(MAP.CATALOG, "SEALED_FORMS_SHA256", MAP.CATALOG.sha256(tuple(intruder))), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        families = list(MAP.CATALOG.FAMILIES); altered = list(families[0]); altered[3] = 16; families[0] = tuple(altered)
        with patch.object(MAP.CATALOG, "FAMILIES", tuple(families)), self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.bound_families(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)
        with patch.object(MAP.CATALOG, "ROUTE", "wrong-route"), self.assertRaises(MAP.CATALOG.CatalogError): MAP.CATALOG.bound_families(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)

    def test_rehashed_promoted_semantic_duplicate_and_symlinked_artifacts_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, item_file = Path(temporary), Path(temporary) / "inventory.json"
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["raw_entries"].reverse(); self.rehash(value); item_file.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["profile_support"] = True; self.rehash(value); item_file.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["sampler_state"] = "fabricated"; self.rehash(value); item_file.write_text(json.dumps(value), encoding="utf-8")
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
            root = Path(temporary); bad = root / "bad.py"; bad.write_text("__file__ = 'unexpected.py'\n", encoding="utf-8")
            link = root / "link.py"; link.symlink_to(bad)
            with self.assertRaises(MAP.InventoryError): MAP.private(link, "f03322341_bad_link")
            with self.assertRaises(MAP.InventoryError): MAP.private(bad, "f03322341_bad_file")
        with tempfile.TemporaryDirectory() as temporary, self.assertRaises(MAP.InventoryError): MAP.validate(Path(temporary))

    def test_fixed_loaders_ignore_and_restore_ambient_aliases(self) -> None:
        aliases = ("f03322341_gles_texture_sampler_catalog", "f03322341_gles_artifact", "f03322341_gles_domain", "f03322341_gles_grammar", "f03322341_gles_cache", "f03322341_gles_ledger")
        code = "\n".join(("import importlib.util, sys, types", "from pathlib import Path", f"aliases = {aliases!r}",
                            "decoys = {name: types.ModuleType(name) for name in aliases}; sys.modules.update(decoys)",
                            f"spec = importlib.util.spec_from_file_location('probe', {str(PROBE)!r})", "module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
                            f"assert module.validate(Path({str(CACHE)!r}))['raw_entry_count'] == 14", "assert all(sys.modules[name] is decoys[name] for name in aliases)"))
        result = subprocess.run([sys.executable, "-B", "-c", code], cwd=tempfile.gettempdir(), text=True, capture_output=True,
                                env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        self.assertEqual(result.returncode, 0, msg=result.stderr)


if __name__ == "__main__": unittest.main()
