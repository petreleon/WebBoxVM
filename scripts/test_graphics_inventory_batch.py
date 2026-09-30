#!/usr/bin/env python3
"""Batch failures cannot regenerate partial source inventories or claim GPU support."""

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from scripts.graphics.inventories.source import InventoryError
from scripts.graphics.inventories.vertex_batch import TASKS, collect, export, select_tasks


class BatchTests(unittest.TestCase):
    def engine(self, task, source=None, fail=False, promoted=False):
        value = {"source": source or {"sha256": "sealed-source"}, "raw_entry_count": 1,
                 "inventory_sha256": task[0], "raw_entries": [["id", "Name", "glName", "void Name();", 1, "1.1", 1]],
                 "raw_only": True, "promotion_allowed": promoted}
        def rendered(_):
            if fail:
                raise InventoryError("source failure")
            return copy.deepcopy(value)
        def forbidden(value):
            if value.get("promotion_allowed"):
                raise InventoryError("promoted source inventory")
        return SimpleNamespace(INVENTORY=Path(task[2] + ".json"), rendered=rendered, validate=rendered,
                               records=lambda value: value["raw_entries"], exact=lambda a, b: a == b,
                               ARTIFACT=SimpleNamespace(forbidden=forbidden), artifact=lambda method, value: method(value))

    def test_complete_finite_batch(self):
        report, generated = collect(Path("unused"), [], loader=self.engine)
        self.assertEqual((report["task_count"], report["raw_entry_count"]), (5, 5))
        self.assertEqual([item["task"] for item in report["tasks"]], [item[0] for item in TASKS])
        self.assertEqual(len(generated), 5)
        self.assertTrue(report["raw_only"])
        self.assertFalse(report["promotion_allowed"])
        self.assertEqual(len(report["batch_sha256"]), 64)

    def test_selection_is_canonical_and_strict(self):
        selected = select_tasks([TASKS[4][0], TASKS[0][0]])
        self.assertEqual(selected, [TASKS[0], TASKS[4]])
        for selection in (["unknown"], [TASKS[0][0]] * 2):
            with self.assertRaises(InventoryError):
                select_tasks(selection)

    def test_failure_before_any_export(self):
        calls = []
        def loader(task):
            calls.append(task[0])
            return self.engine(task, fail=task == TASKS[2])
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary) / "output"
            with self.assertRaisesRegex(InventoryError, "source failure"):
                _, generated = collect(Path("unused"), [], True, loader)
                export(destination, generated)
            self.assertFalse(destination.exists())
            self.assertEqual(calls, [task[0] for task in TASKS[:3]])

    def test_source_disagreement_and_promotion_fail(self):
        for mutation in ("source", "promotion"):
            def loader(task):
                return self.engine(task, source={"sha256": task[0]} if mutation == "source" else None,
                                   promoted=mutation == "promotion")
            with self.subTest(mutation=mutation), self.assertRaises(InventoryError):
                collect(Path("unused"), [], loader=loader)

    def test_export_preserves_generated_values(self):
        _, generated = collect(Path("unused"), [], True, self.engine)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            export(root, generated)
            self.assertEqual(sorted(file.name for file in root.iterdir()), sorted(name for name, _ in generated))
            for name, value in generated:
                self.assertEqual(json.loads((root / name).read_text()), json.loads(value))

    def test_unsafe_output_is_rejected_before_replacement(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target = root / "target"
            target.write_text("preserve")
            (root / "linked.json").symlink_to(target)
            for generated in ([("../escape", "bad")], [("same", "a"), ("same", "b")], [("linked.json", "bad")]):
                with self.subTest(generated=generated), self.assertRaises(InventoryError):
                    export(root, generated)
            self.assertEqual(target.read_text(), "preserve")
            link = root / "link"
            link.symlink_to(root, target_is_directory=True)
            with self.assertRaises(InventoryError):
                export(link, [("item.json", "bad")])


if __name__ == "__main__":
    unittest.main()
