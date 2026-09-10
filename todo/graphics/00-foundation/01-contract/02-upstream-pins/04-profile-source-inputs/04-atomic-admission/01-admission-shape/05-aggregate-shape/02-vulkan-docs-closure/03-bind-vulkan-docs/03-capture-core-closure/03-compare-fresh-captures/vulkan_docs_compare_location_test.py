#!/usr/bin/env python3
"""Hostile structural-location regressions for the Docs capture comparison."""

from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from vulkan_docs_compare_fixture import pair
from vulkan_docs_compare_locations import distinct, independent_locations
from vulkan_docs_compare_model import ComparisonError

FILES = ("run.json", "normalized-inputs.json", "io-events.jsonl", "resolved-includes.jsonl")


class ComparisonLocationTest(unittest.TestCase):
    @staticmethod
    def layout(root: Path, source_alias: bool = False, output_alias: bool = False,
               observer_alias: bool = False, file_alias: str | None = None, cross_role_alias: bool = False) -> None:
        for name in ("observer-a", "observer-b"):
            source = root / "sources" / name
            if name == "observer-b" and (source_alias or cross_role_alias):
                target = root / ("sources/observer-a" if source_alias else "runs/observer-a/generated")
                source.symlink_to(target, target_is_directory=True)
            else:
                source.mkdir(parents=True)
                (source / "vkspec.adoc").write_text("source", encoding="utf-8")
            base = root / "runs" / name
            observer, generated = base / "observer", base / "generated"
            base.mkdir(parents=True, exist_ok=True)
            if name == "observer-b" and observer_alias:
                observer.symlink_to(root / "runs/observer-a/observer", target_is_directory=True)
            else:
                observer.mkdir(parents=True)
                for filename in FILES:
                    path = observer / filename
                    path.write_text("{}", encoding="utf-8")
                    if name == "observer-b" and filename == file_alias:
                        path.unlink()
                        os.link(root / "runs/observer-a/observer" / filename, path)
            if name == "observer-b" and output_alias:
                generated.symlink_to(root / "runs/observer-a/generated", target_is_directory=True)
            else:
                primary = generated / "out" / "html"
                primary.mkdir(parents=True)
                (primary / "vkspec.html").write_text("output", encoding="utf-8")

    def checked(self, **kwargs) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            self.layout(root, **kwargs)
            independent_locations(root, pair())

    def test_nonaliased_structural_locations_are_accepted(self):
        self.checked()

    def test_source_output_and_root_aliases_are_rejected(self):
        for kwargs in ({"source_alias": True}, {"output_alias": True}, {"cross_role_alias": True}):
            with self.subTest(kwargs=kwargs):
                with self.assertRaises(ComparisonError):
                    self.checked(**kwargs)
        with tempfile.TemporaryDirectory() as directory:
            root = (Path(directory) / "artifact").resolve()
            self.layout(root)
            alias = Path(directory) / "artifact-alias"
            alias.symlink_to(root, target_is_directory=True)
            with self.assertRaises(ComparisonError):
                independent_locations(alias, pair())

    def test_required_primary_members_are_not_optional(self):
        for relative in ("sources/observer-a/vkspec.adoc", "runs/observer-a/generated/out/html/vkspec.html"):
            with self.subTest(relative=relative), tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                self.layout(root)
                (root / relative).unlink()
                with self.assertRaises(ComparisonError):
                    independent_locations(root, pair())

    def test_all_required_artifact_file_aliases_are_rejected(self):
        for filename in FILES:
            with self.subTest(filename=filename):
                with self.assertRaises(ComparisonError):
                    self.checked(file_alias=filename)

    def test_nested_observer_symlink_is_rejected(self):
        with self.assertRaises(ComparisonError):
            self.checked(observer_alias=True)

    def test_source_output_cross_role_matrix_is_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            self.layout(root)
            with patch("vulkan_docs_compare_locations.distinct", wraps=distinct) as checked:
                independent_locations(root, pair())
        labels = [call.args[2] for call in checked.call_args_list]
        self.assertEqual(labels.count("source/output"), 4)


if __name__ == "__main__":
    unittest.main()
