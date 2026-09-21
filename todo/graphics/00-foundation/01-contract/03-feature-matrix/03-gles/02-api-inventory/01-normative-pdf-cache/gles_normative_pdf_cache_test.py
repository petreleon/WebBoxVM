#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.3.2.1."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import shutil
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "gles_normative_pdf_cache.py"
LIVE_ROOT = Path("/private/tmp/webboxvm-f0341.cqT6ZX")
LOCATOR = "gles32-pdf-v1:page=1;section=1"


def load():
    spec = importlib.util.spec_from_file_location("f03321_cache_test", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


CACHE = load()


class GlesNormativePdfCacheTests(unittest.TestCase):
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

    def test_manifest_retains_two_classes_and_no_claim_boundary(self) -> None:
        value = CACHE.manifest()
        self.assertEqual(value["source"], CACHE.SOURCE)
        self.assertEqual(value["source_classes"], ["command-state", "limit-format"])
        self.assertEqual((value["physical_pdf_pages"], value["cts_executions"], value["matrix_rows"]), (601, 0, 0))
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["states"]["profile_status"], value["states"]["blocker"]), ("blocked", "matrix-incomplete"))

    def test_retained_external_pdf_and_page_bounds(self) -> None:
        if not CACHE.cache_file(LIVE_ROOT, CACHE.SOURCE).is_file():
            self.skipTest("retained external F03.3.1 cache is unavailable")
        result = CACHE.inspect(LIVE_ROOT, "gles32-pdf-v1:page=601;section=1", "command-state")
        self.assertEqual((result["source"]["record_id"], result["physical_pdf_pages"]), ("gles-32-spec", 601))
        self.assertEqual(CACHE.admitted_source("limit-format", LOCATOR), CACHE.SOURCE)

    def test_page_bounds_reject_before_any_inventory_claim(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with patch.object(CACHE, "physical_pages", return_value=601), patch.object(CACHE, "pdf_bytes", return_value=b"pdf"):
                with self.assertRaisesRegex(CACHE.CacheError, "exceeds"):
                    CACHE.inspect(root, "gles32-pdf-v1:page=602;section=1")
            with self.assertRaises(CACHE.CacheError):
                CACHE.inspect(root, "gles32-pdf-v1:page=0;section=1")

    def test_missing_repository_and_symlink_roots_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, link = Path(temporary) / "root", Path(temporary) / "link"
            root.mkdir(); link.symlink_to(root, target_is_directory=True)
            with self.assertRaises(CACHE.CacheError):
                CACHE.external_root(Path(temporary) / "missing")
            with self.assertRaises(CACHE.CacheError):
                CACHE.external_root(CACHE.REPO)
            with self.assertRaises(CACHE.CacheError):
                CACHE.pdf_bytes(CACHE.REPO, CACHE.SOURCE)
            with self.assertRaises(CACHE.CacheError):
                CACHE.external_root(link)

    def test_stale_mixed_symlinked_oversized_and_inode_changed_payloads_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, target = Path(temporary), self.target(Path(temporary))
            target.write_bytes(b"stale")
            with self.assertRaisesRegex(CACHE.CacheError, "byte count"):
                CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)
            target.unlink(); target.symlink_to(Path(temporary) / "outside")
            with self.assertRaisesRegex(CACHE.CacheError, "regular nonsymlink"):
                CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)
            target.unlink(); target.write_bytes(b"\0" * CACHE.SOURCE["bytes"])
            before = target.lstat()
            changed = types.SimpleNamespace(st_mode=before.st_mode, st_size=before.st_size,
                                            st_dev=before.st_dev, st_ino=before.st_ino + 1)
            with patch.object(CACHE.SOURCE_API.os, "fstat", return_value=changed):
                with self.assertRaisesRegex(CACHE.CacheError, "identity during open"):
                    CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)
            with self.assertRaisesRegex(CACHE.CacheError, "identity"):
                CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)
            with target.open("r+b") as stream:
                stream.truncate(CACHE.SOURCE["bytes"] + 1)
            with self.assertRaisesRegex(CACHE.CacheError, "oversized"):
                CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)
            with patch.object(CACHE.SOURCE_API.os, "O_NOFOLLOW", 0), self.assertRaisesRegex(CACHE.CacheError, "O_NOFOLLOW"):
                CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)
            with patch.object(CACHE.SOURCE_API.os, "O_NONBLOCK", 0), self.assertRaisesRegex(CACHE.CacheError, "O_NONBLOCK"):
                CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)
            target.unlink()
            os.mkfifo(target)
            with self.assertRaisesRegex(CACHE.CacheError, "regular nonsymlink"):
                CACHE.pdf_bytes(CACHE.external_root(root), CACHE.SOURCE)

    def test_aliases_nonadmitted_classes_and_wrong_profile_fail(self) -> None:
        for identifier in ("shader", "precision", "extension", "gles-32-spec", "gl.xml", "vulkan-registry"):
            with self.subTest(identifier=identifier), self.assertRaisesRegex(CACHE.CacheError, "only admitted"):
                CACHE.admitted_source(identifier, LOCATOR)
        original = CACHE.SOURCE_API.authority
        wrong = {**CACHE.SOURCE, "profile": "opengl-4.6-core"}
        CACHE.SOURCE_API.authority = lambda: types.SimpleNamespace(consume=lambda *_: {"source": wrong}, BoundaryError=ValueError)
        try:
            with self.assertRaisesRegex(CACHE.CacheError, "exact sealed"):
                CACHE.admitted_source("command-state", LOCATOR)
        finally:
            CACHE.SOURCE_API.authority = original

    def test_stale_manifest_invalid_pages_and_ambient_alias_fail(self) -> None:
        temporary, path = self.copied_manifest()
        with temporary:
            value = json.loads(path.read_text(encoding="utf-8"))
            value["source_classes"].reverse(); self.rehash(value)
            path.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaisesRegex(CACHE.CacheError, "stale"):
                CACHE.manifest(path)
        temporary, path = self.copied_manifest()
        with temporary:
            value = json.loads(path.read_text(encoding="utf-8"))
            value["claims"]["api_support"] = True; self.rehash(value)
            path.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaisesRegex(CACHE.CacheError, "stale"):
                CACHE.manifest(path)
        for locator in ("gles32-pdf-v1:page=0;section=1", "missing"):
            with self.subTest(locator=locator), self.assertRaises(CACHE.CacheError):
                CACHE.page_number(locator)
        with self.assertRaises(CACHE.CacheError):
            CACHE.admitted_source("command-state", "gles32-pdf-v1:page=1;section=0")
        decoy, prior = types.ModuleType("gles_source_authority"), sys.modules.get("gles_source_authority")
        helper_decoy, helper_prior = types.ModuleType("gles_normative_pdf_cache_source"), sys.modules.get("gles_normative_pdf_cache_source")
        sys.modules["gles_source_authority"], sys.modules["gles_normative_pdf_cache_source"] = decoy, helper_decoy
        try:
            self.assertEqual(Path(CACHE.SOURCE_API.authority().__file__).resolve(), CACHE.AUTHORITY.resolve())
            self.assertEqual(Path(CACHE.helpers().__file__).resolve(), CACHE.HELPERS.resolve())
            self.assertIs(sys.modules["gles_source_authority"], decoy)
            self.assertIs(sys.modules["gles_normative_pdf_cache_source"], helper_decoy)
        finally:
            if prior is None:
                sys.modules.pop("gles_source_authority", None)
            else:
                sys.modules["gles_source_authority"] = prior
            if helper_prior is None:
                sys.modules.pop("gles_normative_pdf_cache_source", None)
            else:
                sys.modules["gles_normative_pdf_cache_source"] = helper_prior


if __name__ == "__main__":
    unittest.main()
