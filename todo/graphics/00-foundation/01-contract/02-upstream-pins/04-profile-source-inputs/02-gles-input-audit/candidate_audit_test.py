#!/usr/bin/env python3
"""Hermetic positive and hostile checks for the GLES 3.2 input audit."""

from __future__ import annotations

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
PARENT = HERE.parent
CANDIDATES = HERE / "candidates.json"
CLOSURE = HERE / "cts_closure.json"
CONFIGURATIONS = HERE / "cts_configurations.json"


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


AUDIT = module("f024_gles_candidates", PARENT / "candidate_contract.py")
COMPOUND = module("f024_gles_compound", PARENT / "compound_selector_contract.py")


class GlesAuditTests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path, Path, Path]:
        temporary = tempfile.TemporaryDirectory()
        candidate, closure, configurations = (Path(temporary.name) / "candidates.json", Path(temporary.name) / "closure.json",
                                               Path(temporary.name) / "configurations.json")
        shutil.copyfile(CANDIDATES, candidate)
        shutil.copyfile(CLOSURE, closure)
        shutil.copyfile(CONFIGURATIONS, configurations)
        return temporary, candidate, closure, configurations

    def change(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_reviewed_material_and_compound_boundary_are_exact(self) -> None:
        self.assertEqual(AUDIT.validate(CANDIDATES, "gles-3.2"), ("accepted", "rejected"))
        self.assertEqual(COMPOUND.validate(CLOSURE, CONFIGURATIONS, CANDIDATES, "gles-3.2"), (4, 12477, 12, 30574, 1, 1))

    def test_mutable_or_substituted_normative_source_is_rejected(self) -> None:
        temporary, candidate, _, _ = self.copied()
        with temporary:
            self.change(candidate, lambda value: value["candidates"][0]["entry"].update(revision="a" * 40))
            with self.assertRaisesRegex(AUDIT.AuditError, "pinned raw source"):
                AUDIT.validate(candidate, "gles-3.2")
        temporary, candidate, _, _ = self.copied()
        with temporary:
            self.change(candidate, lambda value: value["candidates"][0]["entry"].update(source_family="alternate"))
            with self.assertRaisesRegex(AUDIT.AuditError, "reviewed exact"):
                AUDIT.validate(candidate, "gles-3.2")

    def test_compound_selector_cannot_be_admitted_as_a_single_source(self) -> None:
        temporary, candidate, closure, configurations = self.copied()
        with temporary:
            self.change(candidate, lambda value: value["candidates"][1].update(decision="accepted"))
            with self.assertRaisesRegex(AUDIT.AuditError, "reviewed exact selector"):
                AUDIT.validate(candidate, "gles-3.2")
            with self.assertRaisesRegex(COMPOUND.CompoundError, "candidate contract failed"):
                COMPOUND.validate(closure, configurations, candidate, "gles-3.2")

    def test_each_core_selector_and_digest_is_required(self) -> None:
        temporary, candidate, closure, configurations = self.copied()
        with temporary:
            self.change(closure, lambda value: value["included"].pop())
            with self.assertRaisesRegex(COMPOUND.CompoundError, "exact closure"):
                COMPOUND.validate(closure, configurations, candidate, "gles-3.2")
        temporary, candidate, closure, configurations = self.copied()
        with temporary:
            self.change(closure, lambda value: value["included"][0].update(sha256="0" * 64))
            with self.assertRaisesRegex(COMPOUND.CompoundError, "exact closure"):
                COMPOUND.validate(closure, configurations, candidate, "gles-3.2")
        temporary, candidate, closure, configurations = self.copied()
        with temporary:
            self.change(closure, lambda value: value["included"][0].update(case_count=473.0))
            with self.assertRaisesRegex(COMPOUND.CompoundError, "invalid case count"):
                COMPOUND.validate(closure, configurations, candidate, "gles-3.2")

    def test_optional_extension_boundary_and_case_count_are_required(self) -> None:
        temporary, candidate, closure, configurations = self.copied()
        with temporary:
            self.change(closure, lambda value: value.update(excluded=[]))
            with self.assertRaisesRegex(COMPOUND.CompoundError, "exact closure"):
                COMPOUND.validate(closure, configurations, candidate, "gles-3.2")
        temporary, candidate, closure, configurations = self.copied()
        with temporary:
            self.change(closure, lambda value: value.update(core_case_count=12478))
            with self.assertRaisesRegex(COMPOUND.CompoundError, "incorrect aggregate count"):
                COMPOUND.validate(closure, configurations, candidate, "gles-3.2")

    def test_configurations_and_count_types_cannot_be_collapsed(self) -> None:
        temporary, candidate, closure, configurations = self.copied()
        with temporary:
            self.change(configurations, lambda value: value["configurations"].pop())
            with self.assertRaisesRegex(COMPOUND.CompoundError, "exact configurations"):
                COMPOUND.validate(closure, configurations, candidate, "gles-3.2")
        temporary, candidate, closure, configurations = self.copied()
        with temporary:
            self.change(configurations, lambda value: value["configurations"][0].update(use_for_first_egl_config=1))
            with self.assertRaisesRegex(COMPOUND.CompoundError, "invalid configuration"):
                COMPOUND.validate(closure, configurations, candidate, "gles-3.2")
        temporary, candidate, closure, configurations = self.copied()
        with temporary:
            self.change(closure, lambda value: value.update(core_case_count=12477.0))
            with self.assertRaisesRegex(COMPOUND.CompoundError, "invalid core_case_count"):
                COMPOUND.validate(closure, configurations, candidate, "gles-3.2")

    def test_schema_and_descriptor_catalog_binding_are_strict(self) -> None:
        temporary, candidate, _, _ = self.copied()
        with temporary:
            self.change(candidate, lambda value: value.update(schema=True))
            with self.assertRaisesRegex(AUDIT.AuditError, "schema version"):
                AUDIT.validate(candidate, "gles-3.2")
        temporary, candidate, closure, configurations = self.copied()
        with temporary:
            self.change(closure, lambda value: value.update(schema=True))
            with self.assertRaisesRegex(COMPOUND.CompoundError, "schema version"):
                COMPOUND.validate(closure, configurations, candidate, "gles-3.2")
        expected = dict(COMPOUND.CATALOG["gles-3.2"])
        expected["candidate_sha256"] = "0" * 64
        with self.assertRaisesRegex(COMPOUND.CompoundError, "descriptor digest"):
            COMPOUND.rejected_candidate("gles-3.2", CANDIDATES, expected)

    def test_closure_binds_the_reviewed_rejected_descriptor(self) -> None:
        temporary, candidate, closure, configurations = self.copied()
        with temporary:
            self.change(closure, lambda value: value.update(candidate_sha256="0" * 64))
            with self.assertRaisesRegex(COMPOUND.CompoundError, "reviewed rejected candidate"):
                COMPOUND.validate(closure, configurations, candidate, "gles-3.2")

    def test_clis_report_candidate_and_compound_results(self) -> None:
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        audit = subprocess.run([sys.executable, str(PARENT / "candidate_contract.py"), str(CANDIDATES), "gles-3.2"],
                               capture_output=True, text=True, env=environment)
        compound = subprocess.run([sys.executable, str(PARENT / "compound_selector_contract.py"), str(CLOSURE),
                                   str(CONFIGURATIONS), str(CANDIDATES), "gles-3.2"], capture_output=True, text=True, env=environment)
        self.assertEqual(audit.returncode, 0, audit.stderr)
        self.assertEqual(audit.stdout.strip(), "AUDIT: gles-3.2 accepted, rejected")
        self.assertEqual(compound.returncode, 0, compound.stderr)
        self.assertEqual(compound.stdout.strip(), "COMPOUND: gles-3.2 4 core selectors, 12477 unique cases, 12 configurations, 30574 case-configuration runs, 1 excluded selector, 1 excluded configuration")


if __name__ == "__main__":
    unittest.main()
