#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.2.2.5.3.2."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE, LIVE_ROOT = HERE / "opengl_buffer_command_inventory.py", Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f0322532_test", PROBE)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


MAP = load()


class OpenGlBufferCommandInventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        if not MAP.CACHE.cache_file(LIVE_ROOT, MAP.CACHE.SOURCE).is_file(): raise RuntimeError("retained external F03.2.2.1 cache is required")
        cls.bundle = MAP.rendered(LIVE_ROOT)
        cls.value = MAP.ARTIFACTS.validate(MAP.INVENTORY, cls.bundle, MAP.reject); MAP.fences(cls.value)

    def copied(self):
        temporary = tempfile.TemporaryDirectory(); root = Path(temporary.name)
        for name in (MAP.ARTIFACTS.ROOT_NAME,) + tuple(name for _, name in MAP.ARTIFACTS.FRAGMENTS): shutil.copyfile(HERE / name, root / name)
        return temporary, root, root / MAP.ARTIFACTS.ROOT_NAME

    def read(self, path: Path): return json.loads(path.read_text(encoding="utf-8"))

    def write(self, path: Path, value: dict[str, object]) -> None:
        path.write_text(MAP.ARTIFACTS.serialized(value), encoding="utf-8")

    def rehash(self, value: dict[str, object], key: str) -> None:
        value[key] = MAP.ARTIFACTS.digest({name: item for name, item in value.items() if name != key})

    def checked(self, path: Path) -> dict[str, object]:
        with patch.object(MAP, "rendered", return_value=self.bundle): return MAP.validate(LIVE_ROOT, path)

    def bind(self, root: Path, child: Path) -> None:
        value, part = self.read(root), self.read(child)
        receipt = next(item for item in value["artifact_receipts"] if item["file"] == child.name)
        receipt.update(artifact_sha256=part["artifact_sha256"], serialized_sha256=hashlib.sha256(child.read_bytes()).hexdigest())
        self.rehash(value, "raw_inventory_sha256"); self.write(root, value)

    def test_exact_bound_source_only_slice(self) -> None:
        rows = self.value["declarations"]
        self.assertEqual((self.value["profile"], len(rows), self.value["formal_source_declaration_count"]), ("opengl-4.6-core", 38, 39))
        self.assertEqual([row["source_order"] for row in rows], list(range(1, 39)))
        self.assertEqual([(row["numeric_section"], row["formal_declaration_count"]) for row in self.value["source_windows"]], [("6.1", 9), ("6.2", 10), ("6.3", 8), ("6.4", 0), ("6.5", 2), ("6.6", 2), ("6.7", 8)])
        self.assertEqual([row["document_name"] for row in rows[:8]], ["GenBuffers", "DeleteBuffers", "IsBuffer", "BindBuffer", "BindBufferRange", "BindBufferBase", "BindBuffersBase", "BindBuffersRange"])
        self.assertEqual((rows[6]["source_page_span"], rows[6]["source_locator"], rows[18]["declaration"], rows[-1]["c_name"]), ([85, 86], "opengl46-core-pdf-v1:page=85;section=6.1", "void *MapBufferRange( enum target, intptr offset, sizeiptr length, bitfield acesss );", "glGetNamedBufferPointerv"))
        self.assertEqual({key: sum(row["fragment"] == key for row in rows) for key, _ in MAP.ARTIFACTS.FRAGMENTS}, {"binding": 8, "storage": 10, "mapping": 8, "transfer": 4, "query": 8})
        self.assertEqual(self.value["baseline_exclusions"][0]["fact_id"], "command:glCreateBuffers")
        self.assertFalse(any(self.value["claims"].values()))

    def test_rehashed_promoted_or_partial_artifacts_fail(self) -> None:
        edits = (lambda value: value.update(profile="gles-3.2"), lambda value: value.update(formal_source_declaration_count=38),
                 lambda value: value["claims"].update(api_support=True), lambda value: value["deferred_non_declaration_routes"].update(numeric_properties_and_formats="supported"))
        for edit in edits:
            temporary, _, inventory = self.copied()
            with temporary, self.assertRaises(MAP.InventoryError):
                value = self.read(inventory); edit(value); self.rehash(value, "raw_inventory_sha256"); self.write(inventory, value); self.checked(inventory)
        temporary, root, inventory = self.copied()
        with temporary, self.assertRaises(MAP.InventoryError):
            child = root / dict(MAP.ARTIFACTS.FRAGMENTS)["storage"]; value = self.read(child)
            value["declarations"][0]["status"] = "supported"; self.rehash(value, "artifact_sha256"); self.write(child, value); self.bind(inventory, child); self.checked(inventory)

    def test_closed_pdf_anchors_and_dependency_routes_reject_tampering(self) -> None:
        source = self.bundle[MAP.ARTIFACTS.ROOT_NAME]["source"]
        raw = MAP.CACHE.pdf_bytes(MAP.CACHE.external_root(LIVE_ROOT), source)
        altered = list(MAP.RULES.DECLARATIONS); altered[0] = (*altered[0][:4], "void GenBuffersBogus( sizei n, uint *buffers );", False)
        with patch.object(MAP.RULES, "DECLARATIONS", tuple(altered)), self.assertRaises(MAP.RULES.RuleError): MAP.RULES.rows(raw, MAP.GRAMMAR.RULES)
        original = MAP.RULES.page_text
        def missing(data, page): return original(data, page).replace(MAP.RULES.SPAN_END, "missing-span") if page == 86 else original(data, page)
        with patch.object(MAP.RULES, "page_text", side_effect=missing), self.assertRaises(MAP.RULES.RuleError): MAP.RULES.rows(raw, MAP.GRAMMAR.RULES)
        with patch.object(MAP.RULES, "locator", return_value="opengl46-core-pdf-v1:page=85-86;section=6.1"):
            declarations, exclusions, _ = MAP.RULES.rows(raw, MAP.GRAMMAR.RULES)
            with self.assertRaises(MAP.InventoryError): MAP.bound_locators(source, declarations, exclusions)
        with self.assertRaises(MAP.InventoryError): MAP.classifier_binding({})
        with self.assertRaises(MAP.InventoryError): MAP.state_binding({})

    def test_duplicate_json_and_symlink_artifacts_fail_closed(self) -> None:
        temporary, root, inventory = self.copied()
        with temporary, self.assertRaises(MAP.InventoryError):
            inventory.write_text('{"schema":1,"schema":1}', encoding="utf-8"); self.checked(inventory)
        temporary, root, inventory = self.copied()
        with temporary, self.assertRaises(MAP.InventoryError):
            inventory.unlink(); inventory.symlink_to(MAP.INVENTORY); self.checked(inventory)

    def test_fixed_private_loaders_ignore_and_restore_ambient_aliases(self) -> None:
        aliases = ("f0322532_cache", "f0322532_classifier", "f0322532_grammar", "f0322532_baseline", "f0322532_state", "f0322532_rules", "f0322532_artifacts")
        code = "\n".join(("import importlib.util,sys,types", "from pathlib import Path", f"aliases={aliases!r}",
            "decoys={name:types.ModuleType(name) for name in aliases}", "sys.modules.update(decoys)",
            f"spec=importlib.util.spec_from_file_location('probe',{str(PROBE)!r})", "module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)",
            f"assert module.validate(Path({str(LIVE_ROOT)!r}))['profile']=='opengl-4.6-core'", "assert all(sys.modules[name] is decoys[name] for name in aliases)"))
        result = subprocess.run([sys.executable, "-B", "-c", code], capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)
        with tempfile.TemporaryDirectory() as temporary:
            link = Path(temporary) / "rules.py"
            link.symlink_to(MAP.RULES_PATH)
            with self.assertRaises(MAP.InventoryError): MAP.private(link, "rejected_rules")


if __name__ == "__main__": unittest.main()
