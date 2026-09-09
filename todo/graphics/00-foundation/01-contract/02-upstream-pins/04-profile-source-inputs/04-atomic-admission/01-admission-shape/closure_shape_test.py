#!/usr/bin/env python3
"""Hermetic checks for the unadmitted GLES logical-closure shape."""

from __future__ import annotations

import copy
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
AUDIT = HERE.parents[1] / "02-gles-input-audit"
CLOSURE = AUDIT / "cts_closure.json"
CONFIGURATIONS = AUDIT / "cts_configurations.json"
CANDIDATES = AUDIT / "candidates.json"


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


SHAPE = module("f024_closure_shape", HERE / "closure_shape_contract.py")


class ClosureShapeTests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path, Path, Path]:
        temporary = tempfile.TemporaryDirectory()
        candidates, closure, configurations = (Path(temporary.name) / "candidates.json", Path(temporary.name) / "closure.json",
                                               Path(temporary.name) / "configurations.json")
        for source, target in ((CANDIDATES, candidates), (CLOSURE, closure), (CONFIGURATIONS, configurations)):
            shutil.copyfile(source, target)
        return temporary, candidates, closure, configurations

    def change(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        path.write_text(json.dumps(value), encoding="utf-8")

    def validate(self, candidates: Path = CANDIDATES, closure: Path = CLOSURE,
                 configurations: Path = CONFIGURATIONS):
        return SHAPE.validate(closure, configurations, candidates, "gles-3.2")

    def test_exact_closure_is_unadmitted_and_keeps_the_root_rejected(self) -> None:
        shape = self.validate()
        self.assertEqual(shape.state, "unadmitted")
        self.assertEqual(shape.required_input_id, "gles-cts-manifest")
        self.assertEqual(len(shape.core_member_ids), 4)
        self.assertEqual(len(shape.excluded_member_ids), 1)
        self.assertEqual(shape.configuration_count, 12)
        self.assertEqual(SHAPE.COMPOUND.CANDIDATES.validate(CANDIDATES, "gles-3.2"), ("accepted", "rejected"))
        included = json.loads(CLOSURE.read_text(encoding="utf-8"))["included"]
        self.assertEqual(len({item["source_family"] for item in included}), 1)
        self.assertEqual(SHAPE.members(included, json.loads(CLOSURE.read_text(encoding="utf-8"))["excluded"])[0],
                         shape.core_member_ids)
        with self.assertRaises(TypeError):
            SHAPE.UnadmittedClosure("gles-3.2", "root", "0" * 64, (), (), 0, "admitted")

    def test_root_only_missing_or_scope_expanded_closure_is_rejected(self) -> None:
        temporary, candidates, closure, configurations = self.copied()
        with temporary:
            self.change(closure, lambda value: value.update(included=[]))
            with self.assertRaisesRegex(SHAPE.ShapeError, "compound audit failed"):
                self.validate(candidates, closure, configurations)
        temporary, candidates, closure, configurations = self.copied()
        with temporary:
            self.change(closure, lambda value: value["included"].pop())
            with self.assertRaisesRegex(SHAPE.ShapeError, "compound audit failed"):
                self.validate(candidates, closure, configurations)
        temporary, candidates, closure, configurations = self.copied()
        with temporary:
            self.change(closure, lambda value: value["included"].append(value["excluded"][0]))
            with self.assertRaisesRegex(SHAPE.ShapeError, "compound audit failed"):
                self.validate(candidates, closure, configurations)

    def test_stale_member_root_and_configuration_drift_are_rejected(self) -> None:
        temporary, candidates, closure, configurations = self.copied()
        with temporary:
            self.change(closure, lambda value: value["included"][0].update(sha256="0" * 64))
            with self.assertRaisesRegex(SHAPE.ShapeError, "compound audit failed"):
                self.validate(candidates, closure, configurations)
        temporary, candidates, closure, configurations = self.copied()
        with temporary:
            self.change(candidates, lambda value: value["candidates"][1].update(decision="accepted"))
            with self.assertRaisesRegex(SHAPE.ShapeError, "compound audit failed"):
                self.validate(candidates, closure, configurations)
        temporary, candidates, closure, configurations = self.copied()
        with temporary:
            self.change(configurations, lambda value: value["configurations"].pop())
            with self.assertRaisesRegex(SHAPE.ShapeError, "compound audit failed"):
                self.validate(candidates, closure, configurations)

    def test_f02_member_policy_rejects_mutable_and_oversize_inputs(self) -> None:
        closure = json.loads(CLOSURE.read_text(encoding="utf-8"))
        mutable = copy.deepcopy(closure["included"][0])
        mutable["immutable_url"] = mutable["immutable_url"].replace(mutable["revision"], "main")
        with self.assertRaisesRegex(SHAPE.ShapeError, "F02.2 policy"):
            SHAPE.physical_member(mutable, "core")
        oversize = copy.deepcopy(closure["included"][0])
        oversize["bytes"] = 8 * 1024 * 1024 + 1
        with self.assertRaisesRegex(SHAPE.ShapeError, "F02.2 policy"):
            SHAPE.physical_member(oversize, "core")

    def test_duplicate_physical_id_cache_or_selector_is_rejected(self) -> None:
        closure = json.loads(CLOSURE.read_text(encoding="utf-8"))
        excluded = closure["excluded"]
        for field in ("id", "local_cache", "selector"):
            included = copy.deepcopy(closure["included"])
            included[1][field] = included[0][field]
            with self.assertRaisesRegex(SHAPE.ShapeError, "repeats a physical member"):
                SHAPE.members(included, excluded)

    def test_cli_reports_only_an_unadmitted_shape(self) -> None:
        result = subprocess.run([sys.executable, str(HERE / "closure_shape_contract.py"), str(CLOSURE),
                                 str(CONFIGURATIONS), str(CANDIDATES), "gles-3.2"], capture_output=True, text=True,
                                env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "SHAPE: gles-3.2 unadmitted, 4 core members, 1 excluded members, 12 configurations")


if __name__ == "__main__":
    unittest.main()
