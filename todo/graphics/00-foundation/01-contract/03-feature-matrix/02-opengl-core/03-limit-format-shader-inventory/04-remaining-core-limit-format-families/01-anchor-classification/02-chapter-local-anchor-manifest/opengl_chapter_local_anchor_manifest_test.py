#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.2.3.4.1.2."""

from __future__ import annotations

import importlib.util
import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "opengl_chapter_local_anchor_manifest.py"
LIVE_ROOT = Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f0323412_manifest_test", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


MAP = load()


class OpenGlChapterLocalAnchorManifestTests(unittest.TestCase):
    def live(self) -> Path:
        cache = MAP.SOURCE.private(MAP.SOURCE.CACHE, "f0323412_test_cache")
        if not cache.cache_file(LIVE_ROOT, cache.SOURCE).is_file():
            self.fail("retained external F03.2.2.1 cache is required")
        return LIVE_ROOT

    def copied(self):
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        manifest = root / MAP.MANIFEST.name
        shutil.copyfile(MAP.MANIFEST, manifest)
        for filename in MAP.FRAGMENT_FILES.values():
            shutil.copyfile(HERE / filename, root / filename)
        return temporary, manifest, root

    def rehash(self, path: Path, key: str, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8")); edit(value)
        body = {name: item for name, item in value.items() if name != key}
        value[key] = MAP.SOURCE.sha256(body)
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_exact_closed_no_claim_source_universe(self) -> None:
        value = MAP.validate(self.live()); expected, fragments = MAP.rendered(self.live())
        rows, rendered_fragments = MAP.records()
        self.assertEqual(value, expected)
        self.assertEqual((value["profile"], value["candidate_count"], value["semantic_fact_count"]),
                         ("opengl-4.6-core", 38, 0))
        self.assertEqual([part["candidate_count"] for part in fragments], [5, 5, 5, 5, 5, 5, 8])
        self.assertEqual([row["source_order"] for row in rows], list(range(1, 39)))
        self.assertEqual([row["candidate_id"] for row in rows], [f"opengl46-local-{item}" for item in MAP.CLOSED_IDS])
        self.assertEqual(rows[-2]["anchor_kind"], "section-heading")
        self.assertEqual(rows[-1]["table"], "22.2")
        self.assertEqual(rendered_fragments, fragments)
        self.assertEqual(value["classification"], "unclassified")
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["matrix_row_count"], value["cts_executions"]), (0, 0))

    def test_rehashed_stale_cross_profile_and_promoted_artifacts_fail(self) -> None:
        edits = (lambda value: value.update(profile="gles-3.2"), lambda value: value.update(candidate_count=37),
                 lambda value: value.update(classification="route-to-state"), lambda value: value["claims"].update(api_support=True),
                 lambda value: value.update(matrix_row_count=1), lambda value: value.update(status="supported"))
        for edit in edits:
            temporary, manifest, root = self.copied()
            with temporary, self.assertRaises(MAP.SOURCE.ManifestError):
                self.rehash(manifest, "manifest_sha256", edit); MAP.validate(self.live(), manifest, root)
        temporary, manifest, root = self.copied()
        with temporary, self.assertRaises(MAP.SOURCE.ManifestError):
            fragment = root / MAP.FRAGMENT_FILES["buffer"]
            self.rehash(fragment, "fragment_sha256", lambda value: value["candidates"][0].update(row_scope={"first": "*", "last": "*"}))
            MAP.validate(self.live(), manifest, root)

    def test_missing_reordered_wildcard_and_cross_profile_rows_fail_before_artifacts(self) -> None:
        original = MAP.CELLS.FRAGMENTS
        for changed in (original[1:], original + (original[0],), tuple(reversed(original))):
            with self.subTest(changed=changed), patch.object(MAP.CELLS, "FRAGMENTS", changed), self.assertRaises(MAP.SOURCE.ManifestError):
                MAP.records()
        rows, _ = MAP.records()
        for field, value in (("row_scope", {"first": "*", "last": "*"}),
                             ("source_locator", "gles32-core-pdf-v1:page=107;section=6.8")):
            altered = [dict(row) for row in rows]; altered[0][field] = value
            with self.subTest(field=field), self.assertRaises(MAP.SOURCE.ManifestError):
                MAP.fences(altered)

    def test_missing_or_wrong_exact_pdf_anchors_and_dependency_fail(self) -> None:
        original = MAP.SOURCE.page_text
        missing = lambda raw, page: original(raw, page).replace("Table 8.2:", "Table absent:", 1) if page == 215 else original(raw, page)
        wrong_section = lambda raw, page: original(raw, page).replace("8.4. PIXEL RECTANGLES", "8.40. PIXEL RECTANGLES", 1) if page == 215 else original(raw, page)
        for reader in (missing, wrong_section):
            with self.subTest(reader=reader), patch.object(MAP.SOURCE, "page_text", side_effect=reader), self.assertRaises(MAP.SOURCE.ManifestError):
                MAP.rendered(self.live())
        with patch.object(MAP.SOURCE, "AUTHORITY", HERE / "missing-authority.py"), self.assertRaises(MAP.SOURCE.ManifestError):
            MAP.rendered(self.live())


if __name__ == "__main__":
    unittest.main()
