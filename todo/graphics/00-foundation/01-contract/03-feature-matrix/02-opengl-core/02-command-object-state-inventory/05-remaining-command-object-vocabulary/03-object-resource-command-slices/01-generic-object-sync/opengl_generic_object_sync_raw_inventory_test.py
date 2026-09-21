#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.2.2.5.3.1."""

from __future__ import annotations

import importlib.util
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "opengl_generic_object_sync_raw_inventory.py"
LIVE_ROOT = Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f0322531_test", PROBE)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


MAP = load()


class OpenGlGenericObjectSyncRawInventoryTests(unittest.TestCase):
    def live(self) -> Path:
        if not MAP.CACHE.cache_file(LIVE_ROOT, MAP.CACHE.SOURCE).is_file(): self.fail("retained external F03.2.2.1 cache is required")
        return LIVE_ROOT

    def copied(self):
        temporary = tempfile.TemporaryDirectory(); root = Path(temporary.name); inventory = root / MAP.INVENTORY.name
        shutil.copyfile(MAP.INVENTORY, inventory)
        for _, filename in MAP.ARTIFACTS.FRAGMENTS: shutil.copyfile(HERE / filename, root / filename)
        return temporary, root, inventory

    def write(self, path: Path, value: dict[str, object]) -> None:
        path.write_text(MAP.ARTIFACTS.serialized(value), encoding="utf-8")

    def rehash(self, value: dict[str, object], key: str) -> None:
        value[key] = MAP.ARTIFACTS.digest({name: item for name, item in value.items() if name != key})

    def test_exact_closed_source_only_slice(self) -> None:
        value = MAP.validate(self.live())
        self.assertEqual((value["profile"], len(value["declarations"]), value["formal_source_declaration_count"]),
                         ("opengl-4.6-core", 23, 25))
        self.assertEqual([row["source_order"] for row in value["declarations"]], list(range(1, 24)))
        self.assertEqual([row["source_sequence_position"] for row in value["baseline_exclusions"]], [1, 8])
        self.assertEqual([row["fragment"] for row in value["declarations"]].count("sync"), 5)
        self.assertEqual([row["fragment"] for row in value["declarations"]].count("query-lifecycle"), 8)
        self.assertEqual([row["fragment"] for row in value["declarations"]].count("query-state"), 6)
        self.assertEqual([row["fragment"] for row in value["declarations"]].count("query-buffer-state"), 4)
        self.assertEqual(value["classifier_binding"]["route"], "generic-object-sync")
        self.assertEqual(value["returned_state_semantics_deferred"]["target"], "F03.2.2.3")
        self.assertFalse(any(value["claims"].values()))

    def test_rehashed_stale_cross_profile_and_promoted_artifacts_fail(self) -> None:
        edits = (lambda value: value.update(profile="gles-3.2"), lambda value: value.update(formal_source_declaration_count=24),
                 lambda value: value["claims"].update(api_support=True), lambda value: value.update(matrix_row_count=1),
                 lambda value: value["returned_state_semantics_deferred"].update(target="supported"))
        for edit in edits:
            temporary, root, inventory = self.copied()
            with temporary, self.assertRaises(MAP.InventoryError):
                value = json.loads(inventory.read_text(encoding="utf-8")); edit(value); self.rehash(value, "raw_inventory_sha256")
                self.write(inventory, value); MAP.validate(self.live(), inventory)
        temporary, root, inventory = self.copied()
        with temporary, self.assertRaises(MAP.InventoryError):
            filename = dict(MAP.ARTIFACTS.FRAGMENTS)["sync"]; fragment = root / filename
            part = json.loads(fragment.read_text(encoding="utf-8"))
            part["declarations"][0]["status"] = "supported"; self.rehash(part, "artifact_sha256"); self.write(fragment, part)
            root_value = json.loads(inventory.read_text(encoding="utf-8"))
            receipt = next(item for item in root_value["artifact_receipts"] if item["file"] == filename)
            receipt.update(artifact_sha256=part["artifact_sha256"], serialized_sha256=hashlib.sha256(MAP.ARTIFACTS.serialized(part).encode()).hexdigest())
            self.rehash(root_value, "raw_inventory_sha256"); self.write(inventory, root_value); MAP.validate(self.live(), inventory)

    def test_missing_reordered_and_non_exact_source_declarations_fail(self) -> None:
        original = MAP.RULES.DECLARATIONS
        with patch.object(MAP.RULES, "DECLARATIONS", original[1:]), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())
        reordered = tuple(reversed(original))
        with patch.object(MAP.RULES, "DECLARATIONS", reordered), patch.object(MAP.RULES, "CLOSED_DECLARATIONS_SHA256", MAP.RULES.digest(reordered)), self.assertRaises(MAP.InventoryError):
            MAP.rendered(self.live())
        reader = MAP.RULES.page_text
        def missing(raw, page):
            text = reader(raw, page)
            return text.replace("void DeleteSync( sync sync );", "void DeleteAbsent( sync sync );", 1) if page == 59 else text
        with patch.object(MAP.RULES, "page_text", side_effect=missing), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())

    def test_cross_profile_locator_and_fixed_dependency_fail_before_claims(self) -> None:
        value = MAP.validate(self.live()); altered = dict(value); rows = [dict(row) for row in value["declarations"]]
        rows[0]["source_locator"] = "gles32-core-pdf-v1:page=59;section=4.1"; altered["declarations"] = rows
        with self.assertRaises(MAP.InventoryError): MAP.fences(altered)
        with patch.object(MAP.CACHE, "inspect", side_effect=RuntimeError("missing cache")), self.assertRaises(MAP.InventoryError): MAP.rendered(self.live())


if __name__ == "__main__": unittest.main()
