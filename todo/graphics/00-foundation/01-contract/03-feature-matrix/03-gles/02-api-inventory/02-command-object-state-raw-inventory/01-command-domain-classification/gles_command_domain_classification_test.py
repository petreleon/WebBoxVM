#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.2.1."""

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
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "gles_command_domain_classification.py"
LIVE_ROOT = Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f033221_test_map", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


MAP = load()


class GlesCommandDomainClassificationTests(unittest.TestCase):
    def live(self) -> Path:
        cache = MAP.CACHE.cache_file(LIVE_ROOT, MAP.CACHE.SOURCE)
        if not cache.is_file():
            self.fail("retained external F03.3.2.1 cache is unavailable")
        return LIVE_ROOT

    def copied(self):
        temporary, root = tempfile.TemporaryDirectory(), None
        root = Path(temporary.name)
        catalog = root / MAP.CATALOG.name
        shutil.copyfile(MAP.CATALOG, catalog)
        chunks = {}
        for key, source in MAP.CHUNKS.items():
            target = root / source.name
            shutil.copyfile(source, target)
            chunks[key] = target
        return temporary, catalog, chunks

    def write_chunk(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        value["families_sha256"] = hashlib.sha256(MAP.canonical(value["families"])).hexdigest()
        body = {key: item for key, item in value.items() if key != "chunk_sha256"}
        value["chunk_sha256"] = hashlib.sha256(MAP.canonical(body)).hexdigest()
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_exact_closed_map_and_routes(self) -> None:
        value = MAP.validate(self.live())
        self.assertEqual((value["profile"], value["source_class"], value["source_family_count"]),
                         ("gles-3.2", "command-state", 44))
        self.assertEqual((value["source"]["record_id"], value["physical_pdf_pages"]), ("gles-32-spec", 601))
        self.assertEqual([(item["id"], item["family_count"]) for item in value["chunks"]],
                         [("declarations", 5), ("object-declarations", 7), ("execution", 8),
                          ("state-lifecycle", 9), ("state-sources", 5), ("exclusions", 10)])
        self.assertEqual({item["id"] for item in value["routes"]}, set(MAP.RULES.ROUTES))
        index = MAP.ARTIFACT.document(MAP.INDEX)
        self.assertEqual((index["index_omission_crosscheck"]["physical_page_range"],
                          index["index_omission_crosscheck"]["checked_page_count"]), ([571, 601], 31))
        self.assertEqual(value["index_crosscheck_receipt"]["index_sha256"], index["index_sha256"])
        self.assertTrue(value["raw_only"])
        self.assertFalse(value["promotion_allowed"])

    def test_rehashed_missing_reordered_rerouted_and_typed_chunks_fail(self) -> None:
        edits = (
            lambda value: value["families"].reverse(), lambda value: value["families"].pop(),
            lambda value: value["families"][0].update(route="buffer-commands"),
            lambda value: value["families"][0].update(source_order=4.0),
        )
        for edit in edits:
            temporary, catalog, chunks = self.copied()
            with temporary, self.assertRaises(MAP.ClassificationError):
                self.write_chunk(chunks["declarations"], edit)
                MAP.validate(self.live(), catalog, chunks)

    def test_duplicate_catalog_promotion_and_pdf_anchor_loss_fail(self) -> None:
        temporary, catalog, chunks = self.copied()
        with temporary, self.assertRaisesRegex(MAP.ClassificationError, "duplicate"):
            chunks["execution"].write_text('{"schema":1,"schema":1}', encoding="utf-8")
            MAP.validate(self.live(), catalog, chunks)
        temporary, catalog, chunks = self.copied()
        with temporary, self.assertRaises(MAP.ClassificationError):
            value = json.loads(catalog.read_text(encoding="utf-8"))
            value["source_family_count"] = 40.0
            body = {key: item for key, item in value.items() if key != "classification_sha256"}
            value["classification_sha256"] = hashlib.sha256(MAP.canonical(body)).hexdigest()
            catalog.write_text(json.dumps(value), encoding="utf-8")
            MAP.validate(self.live(), catalog, chunks)
        source, *_ = MAP.source_input(self.live())
        raw = MAP.CACHE.pdf_bytes(MAP.CACHE.external_root(self.live()), source)
        original = MAP.RULES.page_text
        with patch.object(MAP.RULES, "page_text", side_effect=lambda data, page: original(data, page).replace("FenceSync", "absent", 1)):
            with self.assertRaisesRegex(MAP.ClassificationError, "exact anchor"):
                MAP.rendered(self.live())

    def test_private_paths_bad_cache_promotion_and_cli_fail_closed(self) -> None:
        with self.assertRaises(MAP.ClassificationError):
            MAP.source_input(Path("/not-an-external-cache"))
        with self.assertRaisesRegex(MAP.ClassificationError, "qualification row"):
            MAP.reject_promotion(self.live())
        code = "\n".join((
            "import importlib.util,sys,types", "decoy=types.ModuleType('gles_command_domain_rules')",
            "sys.modules['gles_command_domain_rules']=decoy",
            f"spec=importlib.util.spec_from_file_location('probe',{str(PROBE)!r})",
            "module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
            "assert module.RULES is not decoy",
        ))
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        result = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, env=environment)
        self.assertEqual(result.returncode, 0, result.stderr)
        passed = subprocess.run([sys.executable, str(PROBE), "--cache-root", str(self.live())], capture_output=True, text=True, env=environment)
        self.assertEqual((passed.returncode, passed.stdout.strip()), (0, "PASS: 44 closed GLES source families; source-routing-only"))


if __name__ == "__main__":
    unittest.main()
