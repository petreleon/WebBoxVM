#!/usr/bin/env python3
"""Reusing a source proof never accepts changed bytes, inputs, or escaped cache paths."""

import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from scripts.graphics.inventories.snapshot import SourceSnapshot
from scripts.graphics.inventories.source import InventoryError


class SnapshotTests(unittest.TestCase):
    def fixture(self, root):
        source = root / "todo/graphics/00-foundation/01-contract"
        source.mkdir(parents=True)
        catalog = source / "catalog.json"
        catalog.write_text('{"source": "fixed"}')
        cache = root / "cache"
        cache.mkdir()
        (cache / "source.pdf").write_bytes(b"sealed PDF bytes")
        return catalog, cache

    def test_exact_proof_is_reused_as_an_independent_copy(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _, cache = self.fixture(root)
            snapshot = SourceSnapshot(root, cache)
            snapshot.remember({"sha256": "fixed"}, b"fixed", {"families": [1]}, {"templates": [2]}, {"forbidden": [3]})
            engine = SimpleNamespace(exact=lambda a, b: a == b)
            first = snapshot.reuse(engine, {"sha256": "fixed"}, b"fixed")
            first[0]["families"].clear()
            self.assertEqual(snapshot.reuse(engine, {"sha256": "fixed"}, b"fixed")[0], {"families": [1]})
            snapshot.finish()

    def test_changed_admission_or_pdf_bytes_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _, cache = self.fixture(root)
            snapshot = SourceSnapshot(root, cache)
            snapshot.remember({"sha256": "fixed"}, b"fixed", {}, {}, {})
            engine = SimpleNamespace(exact=lambda a, b: a == b)
            for source, raw in (({"sha256": "other"}, b"fixed"), ({"sha256": "fixed"}, b"changed")):
                with self.subTest(source=source, raw=raw), self.assertRaises(InventoryError):
                    snapshot.reuse(engine, source, raw)

    def test_edit_addition_deletion_and_cache_change_invalidate_batch(self):
        for mutation in ("edit", "add", "delete", "pdf", "lock", "manifest"):
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                catalog, cache = self.fixture(root)
                snapshot = SourceSnapshot(root, cache)
                if mutation == "edit":
                    catalog.write_text('{"source": "new"}')
                elif mutation == "add":
                    (catalog.parent / "new.py").write_text("unverified = True")
                elif mutation == "delete":
                    catalog.unlink()
                elif mutation == "pdf":
                    (cache / "source.pdf").write_bytes(b"changed")
                elif mutation == "lock":
                    (catalog.parent / "source_contract.lock").write_text("changed")
                else:
                    (catalog.parent / "manifest.toml").write_text("changed")
                with self.subTest(mutation=mutation), self.assertRaises(InventoryError):
                    snapshot.finish()

    def test_symlinked_input_or_root_cannot_reuse_proofs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            catalog, cache = self.fixture(root)
            (catalog.parent / "escaped.json").symlink_to(cache / "source.pdf")
            with self.assertRaises(InventoryError):
                SourceSnapshot(root, cache)
            (catalog.parent / "escaped.json").unlink()
            link = root / "linked-cache"
            link.symlink_to(cache, target_is_directory=True)
            with self.assertRaises(InventoryError):
                SourceSnapshot(root, link)


if __name__ == "__main__":
    unittest.main()
