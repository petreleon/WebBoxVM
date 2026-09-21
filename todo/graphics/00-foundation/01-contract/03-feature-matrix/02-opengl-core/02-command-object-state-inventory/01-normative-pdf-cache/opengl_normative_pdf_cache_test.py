#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.2.2.1."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import shutil
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "opengl_normative_pdf_cache.py"
LIVE_ROOT = Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f03221_cache_test", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


CACHE = load()
LOCATOR = "opengl46-core-pdf-v1:page=1;section=1"


class OpenGlNormativePdfCacheTests(unittest.TestCase):
    def target(self, root: Path) -> Path:
        path = CACHE.cache_file(root, CACHE.SOURCE)
        path.parent.mkdir(parents=True, exist_ok=True)
        return path

    def copied_manifest(self):
        temporary = tempfile.TemporaryDirectory()
        path = Path(temporary.name) / "manifest.json"
        shutil.copyfile(CACHE.MANIFEST, path)
        return temporary, path

    def rehash(self, value: dict[str, object]) -> None:
        body = {key: item for key, item in value.items() if key != "cache_boundary_sha256"}
        value["cache_boundary_sha256"] = hashlib.sha256(CACHE.canonical(body)).hexdigest()

    def test_manifest_retains_no_claim_boundary(self) -> None:
        value = CACHE.manifest()
        self.assertEqual(value["source"], CACHE.SOURCE)
        self.assertEqual((value["physical_pdf_pages"], value["cts_executions"], value["matrix_rows"]), (851, 0, 0))
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["states"]["profile_status"], value["states"]["blocker"]), ("blocked", "matrix-incomplete"))

    def test_retained_external_pdf_and_page_bounds(self) -> None:
        if not CACHE.cache_file(LIVE_ROOT, CACHE.SOURCE).is_file():
            self.skipTest("retained external F03.4.1 cache is unavailable")
        result = CACHE.inspect(LIVE_ROOT, "opengl46-core-pdf-v1:page=851;section=1")
        self.assertEqual((result["source"]["record_id"], result["physical_pdf_pages"]), ("opengl-46-core-spec", 851))
        with self.assertRaisesRegex(CACHE.CacheError, "exceeds"):
            CACHE.inspect(LIVE_ROOT, "opengl46-core-pdf-v1:page=852;section=1")
        with self.assertRaises(CACHE.CacheError):
            CACHE.inspect(LIVE_ROOT, "opengl46-core-pdf-v1:page=0;section=1")

    def test_missing_repository_and_symlink_roots_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, link = Path(temporary) / "root", Path(temporary) / "link"
            root.mkdir(); link.symlink_to(root, target_is_directory=True)
            with self.assertRaises(CACHE.CacheError):
                CACHE.external_root(Path(temporary) / "missing")
            with self.assertRaises(CACHE.CacheError):
                CACHE.external_root(CACHE.REPO)
            with self.assertRaises(CACHE.CacheError):
                CACHE.external_root(link)

    def test_stale_mixed_symlinked_and_oversized_payloads_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, target = Path(temporary), self.target(Path(temporary))
            target.write_bytes(b"stale")
            with self.assertRaisesRegex(CACHE.CacheError, "byte count"):
                CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)
            target.unlink(); target.symlink_to(Path(temporary) / "outside")
            with self.assertRaises(CACHE.CacheError):
                CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)
            target.unlink(); target.write_bytes(b"\0" * CACHE.SOURCE["bytes"])
            with self.assertRaisesRegex(CACHE.CacheError, "identity"):
                CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)
            target.write_bytes(b"")
            with target.open("r+b") as stream:
                stream.truncate(CACHE.SOURCE["bytes"] + 1)
            with self.assertRaisesRegex(CACHE.CacheError, "oversized"):
                CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)
            with patch.object(CACHE.SOURCE_API.os, "O_NOFOLLOW", 0), self.assertRaisesRegex(CACHE.CacheError, "O_NOFOLLOW"):
                CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)

    def test_aliases_nonadmitted_classes_and_wrong_profile_fail(self) -> None:
        for identifier in ("shader", "limit-format", "opengl-gles-registry"):
            with self.subTest(identifier=identifier), self.assertRaisesRegex(CACHE.CacheError, "only the admitted"):
                CACHE.admitted_source(identifier, LOCATOR)
        original = CACHE.SOURCE_API.authority
        wrong = {**CACHE.SOURCE, "profile": "gles-3.2"}
        CACHE.SOURCE_API.authority = lambda: types.SimpleNamespace(consume=lambda *_: {"source": wrong}, BoundaryError=ValueError)
        try:
            with self.assertRaisesRegex(CACHE.CacheError, "exact sealed"):
                CACHE.admitted_source(CACHE.SOURCE_CLASS, LOCATOR)
        finally:
            CACHE.SOURCE_API.authority = original

    def test_stale_manifest_ambient_alias_and_bad_page_count_fail(self) -> None:
        temporary, path = self.copied_manifest()
        with temporary:
            value = json.loads(path.read_text(encoding="utf-8"))
            value["profile"] = "gles-3.2"; self.rehash(value)
            path.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaisesRegex(CACHE.CacheError, "stale"):
                CACHE.manifest(path)
        decoy, prior = types.ModuleType("opengl_source_authority"), sys.modules.get("opengl_source_authority")
        sys.modules["opengl_source_authority"] = decoy
        try:
            self.assertEqual(Path(CACHE.authority().__file__).resolve(), CACHE.AUTHORITY.resolve())
            self.assertIs(sys.modules["opengl_source_authority"], decoy)
        finally:
            if prior is None:
                sys.modules.pop("opengl_source_authority", None)
            else:
                sys.modules["opengl_source_authority"] = prior
        bad = types.SimpleNamespace(returncode=0, stdout=b"Pages: 850\n")
        with patch.object(CACHE.SOURCE_API.subprocess, "run", return_value=bad), self.assertRaisesRegex(CACHE.CacheError, "page count"):
            CACHE.physical_pages(b"not a PDF")


if __name__ == "__main__":
    unittest.main()
