#!/usr/bin/env python3
"""Hermetic hostile-fixture tests for the F02 composite inventory layout."""

from __future__ import annotations

import hashlib
import json
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from inventory_layout import FAMILIES, InventoryLayoutError, load_inventory, render_v2_lock


def quote(value: str) -> str:
    return json.dumps(value)


def entry(family: str, number: int) -> str:
    identifier, digest = f"fixture-{number}", f"{number + 1:064x}"
    values = {
        "id": identifier, "source_family": family,
        "immutable_url": f"https://example.test/{'a' * 40}/input-{number}",
        "revision": "a" * 40, "sha256": digest, "bytes": 1,
        "license": "Fixture", "local_cache": f"webboxvm-graphics/f02/{identifier}/{digest}.source",
        "generated_code_role": "fixture only", "provenance": "https://example.test/tree",
    }
    return "[[inputs]]\n" + "\n".join(
        f"{name} = {value if name == 'bytes' else quote(value)}" for name, value in values.items()) + "\n"


def root(schema: int, files: list[str] | None = None) -> str:
    fields = [
        f"schema = {schema}", 'cache_root = "$XDG_CACHE_HOME"', 'cache_note = "fixture"',
        "required_families = [" + ", ".join(map(quote, sorted(FAMILIES))) + "]",
    ]
    if schema == 1:
        return "\n".join(fields) + "\n"
    return "\n".join(fields + ["input_files = [" + ", ".join(map(quote, files or [])) + "]"]) + "\n"


class InventoryLayoutTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory(dir=HERE)
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)

    def write_v2(self, files: list[str] | None = None, fragment: str | None = None) -> Path:
        files = files if files is not None else ["inputs/part-0001.toml"]
        manifest = self.base / "manifest.toml"
        manifest.write_text(root(2, files), encoding="utf-8")
        for index, name in enumerate(files):
            if name.startswith("../"):
                continue
            target = self.base / name
            target.parent.mkdir(exist_ok=True)
            families = sorted(FAMILIES)[index::len(files)]
            payload = fragment if fragment is not None and index == 0 else "\n".join(
                entry(family, index + number * len(files)) for number, family in enumerate(families))
            target.write_text(payload, encoding="utf-8")
        return manifest

    def lock(self, manifest: Path) -> None:
        manifest.with_name("inventory.lock").write_bytes(render_v2_lock(manifest))

    def test_v2_lock_is_location_independent_and_authoritative(self) -> None:
        manifest = self.write_v2()
        rendered = render_v2_lock(manifest)
        with self.assertRaisesRegex(InventoryLayoutError, "cannot be read"):
            load_inventory(manifest)
        self.lock(manifest)
        inventory = load_inventory(manifest)
        copy = self.base / "copy"
        copy.mkdir()
        self.base = copy
        copied_manifest = self.write_v2()
        self.lock(copied_manifest)
        self.assertEqual(inventory.schema, 2)
        self.assertEqual(len(inventory.inputs), len(FAMILIES))
        self.assertEqual(inventory.revision, hashlib.sha256(rendered).hexdigest())
        self.assertEqual(inventory.revision_path, manifest.with_name("inventory.lock"))
        self.assertEqual(rendered, render_v2_lock(copied_manifest))
        self.assertEqual(inventory.revision, load_inventory(copied_manifest).revision)
        self.assertEqual(rendered.splitlines()[1].split()[0], b"manifest.toml")

    def test_v1_requires_explicit_compatibility(self) -> None:
        manifest = self.base / "manifest.toml"
        manifest.write_text(root(1) + "\n".join(entry(family, number) for number, family in enumerate(sorted(FAMILIES))), encoding="utf-8")
        with self.assertRaisesRegex(InventoryLayoutError, "explicit compatibility"):
            load_inventory(manifest)
        self.assertEqual(load_inventory(manifest, allow_v1=True).schema, 1)
        manifest.write_text(manifest.read_text().replace("schema = 1", "schema = true"), encoding="utf-8")
        with self.assertRaisesRegex(InventoryLayoutError, "schema must be an integer"):
            load_inventory(manifest, allow_v1=True)

    def test_root_component_or_lock_tampering_breaks_the_lock(self) -> None:
        for number, name in enumerate(("manifest.toml", "inputs/part-0001.toml", "inventory.lock")):
            with self.subTest(name=name):
                self.base = Path(self.temporary.name) / f"tamper-{number}"
                self.base.mkdir()
                manifest = self.write_v2()
                self.lock(manifest)
                target = self.base / name
                target.write_bytes(target.read_bytes() + b"# changed\n")
                with self.assertRaisesRegex(InventoryLayoutError, "inventory.lock"):
                    load_inventory(manifest)

    def test_two_components_are_covered_by_the_one_lock(self) -> None:
        files = ["inputs/part-0001.toml", "inputs/part-0002.toml"]
        manifest = self.write_v2(files)
        self.lock(manifest)
        self.assertEqual(len(load_inventory(manifest).inputs), len(FAMILIES))
        second = self.base / files[1]
        second.write_bytes(second.read_bytes() + b"# changed\n")
        with self.assertRaisesRegex(InventoryLayoutError, "inventory.lock"):
            load_inventory(manifest)

    def test_unsafe_duplicate_unsorted_or_renamed_paths_are_rejected(self) -> None:
        cases = (
            ["inputs/part-0002.toml"], ["inputs/part-0001.toml", "inputs/part-0001.toml"],
            ["inputs/part-0002.toml", "inputs/part-0001.toml"], ["../part-0001.toml"],
        )
        for files in cases:
            with self.subTest(files=files):
                manifest = self.write_v2(files)
                with self.assertRaises(InventoryLayoutError):
                    render_v2_lock(manifest)

    def test_missing_symlink_or_malformed_fragment_is_rejected(self) -> None:
        manifest = self.write_v2()
        (self.base / "inputs/part-0001.toml").unlink()
        with self.assertRaisesRegex(InventoryLayoutError, "cannot be read"):
            render_v2_lock(manifest)
        self.base = Path(self.temporary.name) / "malformed"
        self.base.mkdir()
        manifest = self.write_v2(fragment="invalid = true\n")
        with self.assertRaisesRegex(InventoryLayoutError, "unexpected fields"):
            render_v2_lock(manifest)
        self.base = Path(self.temporary.name) / "invalid-toml"
        self.base.mkdir()
        manifest = self.write_v2(fragment="[[inputs]\n")
        with self.assertRaisesRegex(InventoryLayoutError, "not UTF-8 TOML"):
            render_v2_lock(manifest)
        self.base = Path(self.temporary.name) / "linked"
        self.base.mkdir()
        manifest = self.write_v2()
        actual = self.base / "actual"
        (self.base / "inputs").rename(actual)
        (self.base / "inputs").symlink_to(actual, target_is_directory=True)
        with self.assertRaisesRegex(InventoryLayoutError, "symlink"):
            render_v2_lock(manifest)
        self.base = Path(self.temporary.name) / "lock-linked"
        self.base.mkdir()
        manifest = self.write_v2()
        self.lock(manifest)
        lock = self.base / "inventory.lock"
        lock.rename(self.base / "actual.lock")
        lock.symlink_to("actual.lock")
        with self.assertRaisesRegex(InventoryLayoutError, "symlink"):
            load_inventory(manifest)


if __name__ == "__main__":
    unittest.main()
