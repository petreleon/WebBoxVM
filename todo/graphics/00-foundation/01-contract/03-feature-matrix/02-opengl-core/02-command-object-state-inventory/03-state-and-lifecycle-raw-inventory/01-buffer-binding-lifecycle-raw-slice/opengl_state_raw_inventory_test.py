"""Hostile checks for the F03.2.2.3.1 buffer-binding raw slice."""

import copy
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
PROBE = HERE / "opengl_state_raw_inventory.py"
SPEC = importlib.util.spec_from_file_location("f032231_test", PROBE)
API = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(API)
ROOT = Path(os.environ.get("WEBBOXVM_GRAPHICS_CACHE_ROOT", "/private/tmp/webboxvm-f0341.cqT6ZX"))


class StateSliceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.value = API.validate(ROOT)
        raw = API.CACHE.pdf_bytes(API.CACHE.external_root(ROOT), cls.value["source"])
        cls.pages = API.ANCHORS.extract(raw)

    def altered(self, edit, rehash=True):
        value = copy.deepcopy(self.value)
        edit(value)
        if rehash:
            value = API.seal({key: item for key, item in value.items() if key != "inventory_sha256"})
        with tempfile.TemporaryDirectory() as temporary:
            target = Path(temporary) / "inventory.json"
            target.write_text(json.dumps(value))
            with self.assertRaises(API.InventoryError):
                API.validate(ROOT, target)

    def test_exact_live_pdf_and_three_distinct_raw_facts(self):
        self.assertEqual(self.value["source"]["record_id"], "opengl-46-core-spec")
        self.assertEqual(self.value["physical_pdf_pages"], 851)
        facts = self.value["facts"]
        self.assertEqual([row["source_order"] for row in facts], [1, 2, 3])
        self.assertEqual([row["id"] for row in facts], ["buffer-existing-rebind",
                         "buffer-delete-current-context", "array-buffer-binding-state"])
        self.assertEqual((facts[2]["table"], facts[2]["table_row"], facts[2]["initial_value"]), ("23.5", 1, "0"))
        self.assertFalse(self.value["complete"])
        self.assertFalse(self.value["coverage_manifest"]["full_state_inventory_complete"])
        self.assertFalse(any(self.value["claims"].values()))
        self.assertEqual((self.value["matrix_rows"], self.value["cts_executions"]), (0, 0))

    def test_missing_duplicate_reorder_cross_profile_and_stale_hash(self):
        for edit in (lambda v: v["facts"].pop(), lambda v: v["facts"].reverse(),
                     lambda v: v["facts"].append(v["facts"][0]),
                     lambda v: v.update(profile="gles-3.2"),
                     lambda v: v["coverage_manifest"]["ordered_sources"].reverse(),
                     lambda v: v["facts"][2].update(table_row=2)):
            self.altered(edit)
        self.altered(lambda v: v.update(profile="gles-3.2"), rehash=False)

    def test_no_support_owner_matrix_or_complete_promotion(self):
        for edit in (lambda v: v["facts"][0].update(status="supported"),
                     lambda v: v["facts"][0].update(implementation_owner="F08"),
                     lambda v: v["claims"].update(api_support=True),
                     lambda v: v.update(matrix_rows=1), lambda v: v.update(cts_executions=1),
                     lambda v: v.update(complete=True)):
            self.altered(edit)
        with self.assertRaisesRegex(API.InventoryError, "cannot create a Matrix"):
            API.reject_matrix_row({"status": "blocked"}, ROOT)

    def test_missing_ambiguous_and_reordered_prose_or_table_anchors(self):
        for page, anchor in ((84, API.ANCHORS.REBIND), (84, API.ANCHORS.DELETE), (609, API.ANCHORS.ROW)):
            for replacement in ("", anchor + " " + anchor):
                pages = dict(self.pages)
                pages[page] = pages[page].replace(anchor, replacement)
                with self.assertRaises(API.ANCHORS.AnchorError):
                    API.ANCHORS.facts_from_pages(pages)
        pages = dict(self.pages)
        pages[84] = pages[84].replace(API.ANCHORS.REBIND, "SWAP").replace(
            API.ANCHORS.DELETE, API.ANCHORS.REBIND).replace("SWAP", API.ANCHORS.DELETE)
        with self.assertRaisesRegex(API.ANCHORS.AnchorError, "reordered"):
            API.ANCHORS.facts_from_pages(pages)
        pages = dict(self.pages)
        pages[609] = API.ANCHORS.ROW + " " + pages[609].replace(API.ANCHORS.ROW, "")
        with self.assertRaisesRegex(API.ANCHORS.AnchorError, "precedes"):
            API.ANCHORS.facts_from_pages(pages)

    def test_stale_mixed_missing_and_symlink_cache(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target = API.CACHE.cache_file(root, self.value["source"])
            target.parent.mkdir(parents=True)
            for raw in (None, b"stale", b"\0" * self.value["source"]["bytes"]):
                if raw is not None:
                    target.write_bytes(raw)
                with self.assertRaises(API.InventoryError):
                    API.rendered(root)
            target.unlink()
            target.symlink_to(API.CACHE.cache_file(ROOT, self.value["source"]))
            with self.assertRaises(API.InventoryError):
                API.rendered(root)
            link = root / "linked-root"
            link.symlink_to(ROOT, target_is_directory=True)
            for invalid in (link, API.CACHE.REPO, Path("relative")):
                with self.assertRaises(API.InventoryError):
                    API.rendered(invalid)

    def test_extractor_failure_and_truncated_physical_pages(self):
        for result in (subprocess.CompletedProcess([], 1, b"", b"error"),
                       subprocess.CompletedProcess([], 0, b"one page\f", b"")):
            with patch.object(API.ANCHORS.subprocess, "run", return_value=result):
                with self.assertRaises(API.ANCHORS.AnchorError):
                    API.ANCHORS.extract(b"already verified by caller")

    def test_duplicate_json_keys_and_oversized_manifest(self):
        with tempfile.TemporaryDirectory() as temporary:
            target = Path(temporary) / "inventory.json"
            for raw in ('{"schema":1,"schema":1}', " " * (API.LIMIT + 1)):
                target.write_text(raw)
                with self.assertRaises(API.InventoryError):
                    API.validate(ROOT, target)

    def test_fixed_private_imports_from_foreign_cwd_with_decoys(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in ("opengl_state_raw_anchors", "opengl_normative_pdf_cache",
                         "opengl_normative_pdf_cache_source", "opengl_source_authority"):
                (root / (name + ".py")).write_text("raise RuntimeError('ambient decoy imported')\n")
            code = ("import importlib.util,sys;sys.path.insert(0,sys.argv[1]);"
                    "s=importlib.util.spec_from_file_location('probe',sys.argv[2]);"
                    "m=importlib.util.module_from_spec(s);s.loader.exec_module(m);"
                    "assert m.validate(m.Path(sys.argv[3]))['complete'] is False")
            result = subprocess.run([sys.executable, "-B", "-c", code, str(root), str(PROBE), str(ROOT)],
                                    cwd=root, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            decoy = root / "helper.py"
            decoy.symlink_to(API.ANCHOR_PATH)
            with self.assertRaisesRegex(API.InventoryError, "nonsymlink"):
                API.private(decoy, "rejected_helper")


if __name__ == "__main__":
    unittest.main()
