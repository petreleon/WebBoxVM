#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.2.3.1."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
CACHE = Path("/private/tmp/webboxvm-f0341.cqT6ZX")
PROBE = HERE / "gles_generic_sync_query_raw_inventory.py"


def load():
    spec = importlib.util.spec_from_file_location("f0332231_gles_generic_test", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


MAP = load()


class GenericSyncQueryRawInventoryTests(unittest.TestCase):
    def live(self) -> Path:
        cache = MAP.CACHE.cache_file(CACHE, MAP.CACHE.SOURCE)
        if not cache.is_file():
            self.fail("retained external F03.3.2.1 cache is unavailable")
        return CACHE

    def rehash(self, value: dict[str, object]) -> None:
        value["raw_entries_sha256"] = hashlib.sha256(MAP.canonical(value["raw_entries"])).hexdigest()
        body = {key: item for key, item in value.items() if key != "inventory_sha256"}
        value["inventory_sha256"] = hashlib.sha256(MAP.canonical(body)).hexdigest()

    def test_exact_bound_literal_slice(self) -> None:
        value = MAP.validate(self.live())
        entries = value["raw_entries"]
        self.assertEqual((value["profile"], value["source_class"], value["raw_entry_count"]), ("gles-3.2", "command-state", 19))
        self.assertEqual([(row["id"], row["source_order"]) for row in value["domain_families"]],
                         [("generic-context", 8), ("sync", 12), ("query", 13), ("memory-barrier", 16)])
        self.assertEqual([row["c_name"] for row in entries[:5]], ["glGetError", "glGetGraphicsResetStatus", "glFlush", "glFinish", "glFenceSync"])
        self.assertEqual([row["c_name"] for row in entries[-2:]], ["glMemoryBarrier", "glMemoryBarrierByRegion"])
        self.assertEqual({family: sum(row["family_id"] == family for row in entries) for family in {row["family_id"] for row in entries}},
                         {"generic-context": 4, "sync": 6, "query": 7, "memory-barrier": 2})
        self.assertTrue(value["raw_only"])
        self.assertFalse(value["promotion_allowed"])

    def test_rejects_guessed_literal_and_rerouted_domain_binding(self) -> None:
        declarations = list(MAP.CATALOG.DECLARATIONS)
        declarations[0] = (*declarations[0][:4], "enum GetErrorBogus( void );")
        with patch.object(MAP.CATALOG, "DECLARATIONS", tuple(declarations)), self.assertRaises(MAP.InventoryError):
            MAP.rendered(self.live())
        bindings = list(MAP.CATALOG.FAMILY_BINDINGS)
        bindings[-1] = (*bindings[-1][:4], 142, bindings[-1][-1])
        with patch.object(MAP.CATALOG, "FAMILY_BINDINGS", tuple(bindings)), self.assertRaises(MAP.CATALOG.CatalogError):
            MAP.CATALOG.bound_families(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)

    def test_rehashed_reordered_and_promoted_artifacts_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "inventory.json"
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8"))
            value["raw_entries"].reverse()
            self.rehash(value)
            path.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError):
                MAP.validate(self.live(), path)
            value = json.loads(MAP.INVENTORY.read_text(encoding="utf-8"))
            value["profile_support"] = True
            self.rehash(value)
            path.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.InventoryError):
                MAP.artifact(MAP.ARTIFACT.forbidden, value)

    def test_private_cache_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary, self.assertRaises(MAP.InventoryError):
            MAP.validate(Path(temporary))


if __name__ == "__main__":
    unittest.main()
