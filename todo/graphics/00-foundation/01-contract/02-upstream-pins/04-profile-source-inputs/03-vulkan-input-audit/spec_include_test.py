#!/usr/bin/env python3
"""Hermetic positive and hostile checks for Vulkan specification includes."""

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
INCLUDES = HERE / "spec_includes.json"


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


AUDIT = module("f024_vulkan_spec_candidates", PARENT / "candidate_contract.py")
INCLUDE = module("f024_vulkan_spec_includes", HERE / "spec_include_contract.py")


class VulkanSpecIncludeTests(unittest.TestCase):
    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path, Path]:
        temporary = tempfile.TemporaryDirectory()
        candidate, includes = Path(temporary.name) / "candidates.json", Path(temporary.name) / "includes.json"
        shutil.copyfile(CANDIDATES, candidate)
        shutil.copyfile(INCLUDES, includes)
        return temporary, candidate, includes

    def change(self, path: Path, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8"))
        edit(value)
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_reviewed_rejected_spec_root_has_its_exact_direct_closure(self) -> None:
        self.assertEqual(AUDIT.validate(CANDIDATES, "vulkan-1.4-core"), ("rejected", "rejected"))
        self.assertEqual(INCLUDE.validate(INCLUDES, CANDIDATES, "vulkan-1.4-core"), 73)

    def test_every_include_and_its_order_are_required(self) -> None:
        temporary, candidate, includes = self.copied()
        with temporary:
            self.change(includes, lambda value: value["direct_includes"].reverse())
            with self.assertRaisesRegex(INCLUDE.IncludeError, "reviewed directive transcript"):
                INCLUDE.validate(includes, candidate, "vulkan-1.4-core")
        temporary, candidate, includes = self.copied()
        with temporary:
            self.change(includes, lambda value: value["direct_includes"].pop())
            with self.assertRaisesRegex(INCLUDE.IncludeError, "incorrect count"):
                INCLUDE.validate(includes, candidate, "vulkan-1.4-core")

    def test_include_syntax_digest_and_count_types_fail_closed(self) -> None:
        temporary, candidate, includes = self.copied()
        with temporary:
            self.change(includes, lambda value: value["direct_includes"].__setitem__(0, "include::../escape.adoc[]"))
            with self.assertRaisesRegex(INCLUDE.IncludeError, "path is unsafe"):
                INCLUDE.validate(includes, candidate, "vulkan-1.4-core")
        temporary, candidate, includes = self.copied()
        with temporary:
            self.change(includes, lambda value: value.update(direct_include_count=73.0))
            with self.assertRaisesRegex(INCLUDE.IncludeError, "stale count or digest"):
                INCLUDE.validate(includes, candidate, "vulkan-1.4-core")
        temporary, candidate, includes = self.copied()
        with temporary:
            self.change(includes, lambda value: value.update(include_list_sha256="0" * 64))
            with self.assertRaisesRegex(INCLUDE.IncludeError, "stale count or digest"):
                INCLUDE.validate(includes, candidate, "vulkan-1.4-core")
        temporary, candidate, includes = self.copied()
        with temporary:
            self.change(includes, lambda value: value["direct_includes"].__setitem__(1, value["direct_includes"][0]))
            with self.assertRaisesRegex(INCLUDE.IncludeError, "repeat a path"):
                INCLUDE.validate(includes, candidate, "vulkan-1.4-core")

    def test_root_binding_schema_and_closure_boundary_are_strict(self) -> None:
        changes = ((lambda value: value.update(candidate_sha256="0" * 64), "reviewed root"),
                   (lambda value: value.update(schema=True), "schema version"),
                   (lambda value: value.update(closure_boundary="single-file"), "closure boundary"),
                   (lambda value: value["boundary_examples"].pop(), "closure boundary"))
        for edit, message in changes:
            with self.subTest(message=message):
                temporary, candidate, includes = self.copied()
                with temporary:
                    self.change(includes, edit)
                    with self.assertRaisesRegex(INCLUDE.IncludeError, message):
                        INCLUDE.validate(includes, candidate, "vulkan-1.4-core")

    def test_invalid_candidate_is_wrapped_by_the_include_contract(self) -> None:
        temporary, candidate, includes = self.copied()
        with temporary:
            self.change(candidate, lambda value: value.update(schema=True))
            with self.assertRaisesRegex(INCLUDE.IncludeError, "candidate contract failed"):
                INCLUDE.validate(includes, candidate, "vulkan-1.4-core")

    def test_cli_reports_the_rejected_root_and_include_count(self) -> None:
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        result = subprocess.run([sys.executable, str(HERE / "spec_include_contract.py"), str(INCLUDES),
                                 str(CANDIDATES), "vulkan-1.4-core"], capture_output=True, text=True, env=environment)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "INCLUDES: vulkan-1.4-core 73 direct specification includes")


if __name__ == "__main__":
    unittest.main()
