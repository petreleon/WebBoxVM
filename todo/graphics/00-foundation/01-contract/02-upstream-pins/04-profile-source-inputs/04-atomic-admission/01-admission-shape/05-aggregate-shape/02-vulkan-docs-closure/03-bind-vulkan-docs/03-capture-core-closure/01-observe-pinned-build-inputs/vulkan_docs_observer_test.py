#!/usr/bin/env python3
"""Positive checks for observer-only Docs replay records."""

from __future__ import annotations

import tempfile
import unittest
import json
from pathlib import Path

from vulkan_docs_observer_contract import observation_value
from vulkan_docs_observer_fixture import observation, source_tree, traces
from vulkan_docs_observer_model import DERIVED, RAW
from vulkan_docs_observer_normalize import normalize, tree


class DocsObserverTest(unittest.TestCase):
    def capture(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        source, generated = source_tree(root)
        io, includes = traces(root)
        return source, generated, io, includes

    def test_normalizes_actual_content_reads_without_output_input_alias(self):
        source, generated, io, includes = self.capture()
        result = normalize(source, generated, io, includes)
        self.assertEqual((result["raw_count"], result["derived_count"], len(result["includes"])), (4, 1, 2))
        self.assertEqual({row["kind"] for row in result["records"]}, {RAW, DERIVED})
        self.assertEqual(result["ignored_runtime_reads"], 1)

    def test_normalization_keeps_derived_selector_and_phase_roles(self):
        source, generated, io, includes = self.capture()
        rows = normalize(source, generated, io, includes)["records"]
        katex = next(row for row in rows if row["selector"] == "katex/katex.min.js")
        self.assertEqual(katex["phase_roles"], ["asset-copy", "postprocess"])
        self.assertIn("generated/specattribs.adoc", {row["selector"] for row in rows})

    def test_tree_has_one_location_independent_identity(self):
        source, generated, _, _ = self.capture()
        copied = source.parent / "copied"
        copied.mkdir()
        for path in generated.rglob("*"):
            if path.is_file():
                target = copied / path.relative_to(generated)
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(path.read_bytes())
        self.assertEqual(tree(generated), tree(copied))

    def test_tree_accepts_zero_byte_rendered_members(self):
        _, generated, _, _ = self.capture()
        (generated / "empty-marker").write_bytes(b"")
        count, total, _ = tree(generated)
        self.assertEqual((count, total), (3, len(b"attributes") + len(b"output")))

    def test_normalization_accepts_an_unknown_exec_start_before_producer(self):
        source, generated, io, includes = self.capture()
        rows = io.read_text(encoding="utf-8").splitlines()
        rows.insert(0, json.dumps({"kind": "start", "pid": 11, "argv_hex": b"env".hex()}))
        io.write_text("\n".join(rows) + "\n", encoding="utf-8")
        self.assertEqual(normalize(source, generated, io, includes)["raw_count"], 4)

    def test_compact_header_stays_unadmitted(self):
        result = observation_value(observation())
        self.assertEqual(len(result.runs), 2)
        self.assertFalse(result.admitted)
        self.assertFalse(result.cutover_ready)


if __name__ == "__main__":
    unittest.main()
