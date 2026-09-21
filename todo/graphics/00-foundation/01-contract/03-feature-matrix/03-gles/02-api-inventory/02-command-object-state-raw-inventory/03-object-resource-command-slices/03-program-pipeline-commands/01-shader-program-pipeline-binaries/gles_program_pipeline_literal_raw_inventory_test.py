#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.2.3.3.1."""

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
PROBE = HERE / "gles_program_pipeline_literal_raw_inventory.py"


def load():
    spec = importlib.util.spec_from_file_location("f03322331_gles_program_test", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


MAP = load()


class ProgramPipelineLiteralInventoryTests(unittest.TestCase):
    def live(self) -> Path:
        if not MAP.CACHE.cache_file(CACHE, MAP.CACHE.SOURCE).is_file():
            self.fail("retained external F03.3.2.1 cache is unavailable")
        return CACHE

    def rehash(self, value: dict[str, object]) -> None:
        value["raw_entries_sha256"] = hashlib.sha256(MAP.canonical(value["raw_entries"])).hexdigest()
        body = {key: item for key, item in value.items() if key != "inventory_sha256"}
        value["inventory_sha256"] = hashlib.sha256(MAP.canonical(body)).hexdigest()

    def test_exact_bound_literal_slice(self) -> None:
        value = MAP.validate(self.live()); entries = MAP.records(value)
        expected = ["CreateShader", "ShaderSource", "CompileShader", "ReleaseShaderCompiler", "DeleteShader",
                    "IsShader", "ShaderBinary", "CreateProgram", "AttachShader", "DetachShader", "LinkProgram",
                    "UseProgram", "ProgramParameteri", "DeleteProgram", "IsProgram", "CreateShaderProgramv",
                    "GetProgramInterfaceiv", "GetProgramResourceIndex", "GetProgramResourceName", "GetProgramResourceiv",
                    "GetProgramResourceLocation", "GenProgramPipelines", "DeleteProgramPipelines", "IsProgramPipeline",
                    "BindProgramPipeline", "UseProgramStages", "ActiveShaderProgram", "GetProgramBinary", "ProgramBinary"]
        self.assertEqual((value["profile"], value["source_class"], value["raw_entry_count"]), ("gles-3.2", "command-state", 29))
        self.assertEqual(value["domain_families"], [{"id": "program-shader", "source_order": 15}])
        self.assertEqual([row["unprefixed_name"] for row in entries], expected)
        self.assertEqual([(row["physical_page"], row["section"]) for row in entries[4:7]], [(88, "7.1"), (88, "7.1"), (88, "7.2")])
        self.assertEqual([row["c_name"] for row in entries], ["gl" + name for name in expected])
        self.assertTrue(value["raw_only"]); self.assertFalse(value["promotion_allowed"])

    def test_rejects_changed_set_order_scope_and_cross_family_source(self) -> None:
        declarations = list(MAP.CATALOG.DECLARATIONS)
        declarations[0] = (*declarations[0][:3], "uint CreateShaderBogus( enum type );")
        with patch.object(MAP.CATALOG, "DECLARATIONS", tuple(declarations)), self.assertRaises(MAP.InventoryError):
            MAP.rendered(self.live())
        for changed in (tuple(declarations[:-1]), tuple(reversed(MAP.CATALOG.DECLARATIONS))):
            with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.declaration_sha256(changed)), self.assertRaises(MAP.InventoryError):
                MAP.rendered(self.live())
        declarations = list(MAP.CATALOG.DECLARATIONS); declarations[4] = (88, "7.2", *declarations[4][2:])
        changed = tuple(declarations)
        with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.declaration_sha256(changed)), self.assertRaises(MAP.InventoryError):
            MAP.rendered(self.live())
        declarations = list(MAP.CATALOG.DECLARATIONS); declarations[6] = (88, "7.1", *declarations[6][2:])
        changed = tuple(declarations)
        with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.declaration_sha256(changed)), self.assertRaises(MAP.InventoryError):
            MAP.rendered(self.live())
        declarations = list(MAP.CATALOG.DECLARATIONS)
        declarations[0] = (141, "7.11.2", "MemoryBarrier", "void MemoryBarrier( bitfield barriers );")
        changed = tuple(declarations)
        with patch.object(MAP.CATALOG, "DECLARATIONS", changed), patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.declaration_sha256(changed)), self.assertRaises(MAP.InventoryError):
            MAP.rendered(self.live())

    def test_rejects_stale_domain_order_and_route(self) -> None:
        family = list(MAP.CATALOG.FAMILY); family[3] = 14
        with patch.object(MAP.CATALOG, "FAMILY", tuple(family)), self.assertRaises(MAP.CATALOG.CatalogError):
            MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)
        with patch.object(MAP.CATALOG, "ROUTE", "wrong-route"), self.assertRaises(MAP.CATALOG.CatalogError):
            MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)

    def test_rehashed_promoted_stale_duplicate_and_symlinked_artifacts_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, item_file = Path(temporary), Path(temporary) / "inventory.json"
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["raw_entries"].reverse(); self.rehash(value)
            item_file.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["profile_support"] = True; self.rehash(value)
            item_file.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8")); value["raw_only"] = False
            item_file.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            item_file.write_text('{"schema":1,"schema":2}', encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            item_file.write_text('{"schema":NaN}', encoding="utf-8")
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), item_file)
            link = root / "inventory-link.json"; link.symlink_to(item_file)
            with self.assertRaises(MAP.InventoryError): MAP.validate(self.live(), link)

    def test_unavailable_shader_substitute_and_private_dependencies_fail_closed(self) -> None:
        with self.assertRaisesRegex(MAP.InventoryError, "separate F02 admission"):
            MAP.reject_unadmitted("essl-320-spec")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); bad = root / "bad.py"; bad.write_text("__file__ = 'unexpected.py'\n", encoding="utf-8")
            link = root / "link.py"; link.symlink_to(bad)
            with self.assertRaises(MAP.InventoryError): MAP.private(link, "f03322331_bad_link")
            with self.assertRaises(MAP.InventoryError): MAP.private(bad, "f03322331_bad_file")
        with tempfile.TemporaryDirectory() as temporary, self.assertRaises(MAP.InventoryError):
            MAP.validate(Path(temporary))

    def test_fixed_loaders_ignore_and_restore_ambient_aliases(self) -> None:
        aliases = ("f03322331_gles_program_catalog", "f03322331_gles_artifact", "f03322331_gles_domain",
                   "f03322331_gles_grammar", "f03322331_gles_cache", "f03322331_gles_ledger")
        code = "\n".join((
            "import importlib.util, sys, types", "from pathlib import Path", f"aliases = {aliases!r}",
            "decoys = {name: types.ModuleType(name) for name in aliases}; sys.modules.update(decoys)",
            f"spec = importlib.util.spec_from_file_location('probe', {str(PROBE)!r})",
            "module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
            f"assert module.validate(Path({str(CACHE)!r}))['raw_entry_count'] == 29",
            "assert all(sys.modules[name] is decoys[name] for name in aliases)"))
        result = subprocess.run([sys.executable, "-B", "-c", code], cwd=tempfile.gettempdir(), text=True,
                                capture_output=True, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        self.assertEqual(result.returncode, 0, msg=result.stderr)


if __name__ == "__main__":
    unittest.main()
