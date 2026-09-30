#!/usr/bin/env python3
"""Family fragments cannot omit, substitute, reorder, or promote raw facts."""

import copy
import hashlib
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from scripts.graphics.inventories import storage
from scripts.graphics.inventories.source import InventoryError
from scripts.graphics.inventories.vertex_batch import STATE_TASKS, collect, select_tasks

ROOT = Path(__file__).resolve().parents[1]
ARTIFACT_PATH = ROOT / ("todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/"
                       "02-command-object-state-raw-inventory/01-command-domain-classification/gles_command_domain_artifact.py")
spec = importlib.util.spec_from_file_location("storage_test_artifact", ARTIFACT_PATH)
ARTIFACT = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ARTIFACT)
FIELDS = ["raw_id", "unprefixed_name", "c_name", "declaration", "physical_page", "section",
          "source_order", "family_id", "source_family_order"]


def artifact(method, *args):
    try:
        return method(*args)
    except ValueError as error:
        raise InventoryError(str(error)) from error


def fixture():
    engine = SimpleNamespace(INVENTORY=Path("bounded.json"), CATALOG=SimpleNamespace(FRAGMENTED=True),
                             canonical=ARTIFACT.canonical, exact=ARTIFACT.exact, ARTIFACT=ARTIFACT, artifact=artifact)
    body = {"schema": 1, "kind": "bounded", "source": {"sha256": "sealed-source"},
            "domain_families": [{"id": "draw", "source_order": 23}, {"id": "compute", "source_order": 30}],
            "raw_entry_fields": FIELDS, "raw_entry_defaults": {}, "raw_entry_count": 2,
            "raw_entries": [["a", "Draw", "glDraw", "void Draw();", 1, "1.1", 1, "draw", 23],
                            ["b", "Dispatch", "glDispatch", "void Dispatch();", 2, "2.1", 2, "compute", 30]],
            "raw_only": True, "promotion_allowed": False}
    body["raw_entries_sha256"] = hashlib.sha256(engine.canonical(body["raw_entries"])).hexdigest()
    return engine, storage.sealed(engine, body, "inventory_sha256")


class StorageTests(unittest.TestCase):
    def write(self, root, engine, value):
        for name, contents in storage.generated(engine, value):
            (root / name).write_text(contents)

    def test_exact_aggregate_and_family_fragments(self):
        engine, value = fixture()
        docs = storage.documents(engine, value)
        self.assertEqual(len(docs), 3)
        self.assertEqual(docs[0][1]["aggregate_inventory_sha256"], value["inventory_sha256"])
        self.assertEqual([item[1]["raw_entry_count"] for item in docs[1:]], [1, 1])
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.write(root, engine, value)
            storage.validate(engine, value, root / "bounded.json")

    def test_rehashed_stale_count_row_order_name_and_promotion_fail(self):
        engine, value = fixture()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for mutation in ("count", "order", "name", "promotion"):
                self.write(root, engine, value)
                path = root / "bounded_draw.json"
                changed = json.loads(path.read_text())
                if mutation == "count": changed["raw_entry_count"] = 0
                elif mutation == "order": changed["raw_entries"][0][6] = 2
                elif mutation == "name": changed["raw_entries"][0][2] = "glOther"
                else: changed["promotion_allowed"] = True
                changed.pop("fragment_sha256")
                path.write_text(json.dumps(storage.sealed(engine, changed, "fragment_sha256")))
                with self.subTest(mutation=mutation), self.assertRaises(InventoryError):
                    storage.validate(engine, value, root / "bounded.json")

    def test_missing_symlink_and_index_redirection_fail(self):
        engine, value = fixture()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.write(root, engine, value)
            fragment = root / "bounded_draw.json"
            saved = fragment.read_bytes()
            fragment.unlink()
            with self.assertRaises(InventoryError): storage.validate(engine, value, root / "bounded.json")
            (root / "other.json").write_bytes(saved)
            fragment.symlink_to(root / "other.json")
            with self.assertRaises(InventoryError): storage.validate(engine, value, root / "bounded.json")
            fragment.unlink(); fragment.write_bytes(saved)
            index = json.loads((root / "bounded.json").read_text())
            index["raw_entry_fragments"][0]["filename"] = "other.json"
            index.pop("inventory_sha256")
            (root / "bounded.json").write_text(json.dumps(storage.sealed(engine, index, "inventory_sha256")))
            with self.assertRaises(InventoryError): storage.validate(engine, value, root / "bounded.json")

    def test_family_omission_wrong_family_order_and_unsafe_name_fail(self):
        engine, value = fixture()
        for mutation in ("omitted", "family-order", "unsafe"):
            changed = copy.deepcopy(value)
            if mutation == "omitted": changed["domain_families"].pop()
            elif mutation == "family-order": changed["raw_entries"][0][8] = 30
            else: changed["domain_families"][0]["id"] = "../escape"
            with self.subTest(mutation=mutation), self.assertRaises(InventoryError): storage.documents(engine, changed)

    def test_default_vertex_documents_remain_byte_identical(self):
        engine, value = fixture()
        engine.CATALOG.FRAGMENTED = False
        self.assertEqual(storage.documents(engine, value), [(engine.INVENTORY.name, value, "inventory_sha256")])

    def test_state_selection_and_partial_completion_are_explicit(self):
        self.assertEqual(select_tasks([], "state"), list(STATE_TASKS))
        for selection in ([STATE_TASKS[0][0]] * 2, ["F03.3.2.2.3.6.1"]):
            with self.assertRaises(InventoryError): select_tasks(selection, "state")
        def loader(task):
            value = {"source": {"sha256": "same"}, "raw_entry_count": 0, "inventory_sha256": task[0],
                     "raw_entries": [], "source_coverage": {"complete": task != STATE_TASKS[0]}}
            return SimpleNamespace(INVENTORY=Path(task[2] + ".json"), rendered=lambda root: value,
                                   validate=lambda root: value, records=lambda value: [], exact=ARTIFACT.exact,
                                   ARTIFACT=ARTIFACT, artifact=artifact)
        report, _ = collect(Path("unused"), [], loader=loader, group="state")
        self.assertFalse(report["source_coverage"]["complete"])
        self.assertEqual(report["source_coverage"]["incomplete_tasks"], [STATE_TASKS[0][0]])
        selected, _ = collect(Path("unused"), [STATE_TASKS[1][0]], loader=loader, group="state")
        self.assertFalse(selected["source_coverage"]["complete"])
        self.assertEqual(len(selected["source_coverage"]["missing_tasks"]), 3)


if __name__ == "__main__":
    unittest.main()
