#!/usr/bin/env python3
"""Hermetic tests for deterministic graphics chunk generation."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

import graphics_chunker as chunker

REVISION = "a" * 64


def spec(records: list[dict[str, object]]) -> bytes:
    return (json.dumps({"schema": 1, "manifest_revision": REVISION, "records": records}) + "\n").encode()


class ChunkerTests(unittest.TestCase):
    def write_spec(self, directory: Path, records: list[dict[str, object]]) -> Path:
        path = directory / "input.json"
        path.write_bytes(spec(records))
        return path

    def records(self) -> list[dict[str, object]]:
        return [
            {"id": "alpha", "lines": ["kind: fixture", "role: deterministic"]},
            {"id": "beta", "lines": ["kind: fixture", "role: provenance"]},
        ]

    def test_write_and_check_are_byte_identical(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            input_path, output = self.write_spec(root, self.records()), root / "output"
            bundle = chunker.expected_bundle(input_path)
            chunker.write_bundle(output, bundle)
            first = {path.name: path.read_bytes() for path in output.iterdir()}
            chunker.check_bundle(output, bundle)
            chunker.write_bundle(output, bundle)
            self.assertEqual(first, {path.name: path.read_bytes() for path in output.iterdir()})

    def test_rejects_unordered_records(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = self.write_spec(Path(temporary), list(reversed(self.records())))
            with self.assertRaisesRegex(chunker.ChunkError, "uniquely ordered"):
                chunker.expected_bundle(path)

    def test_rejects_a_mismatched_source_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / "manifest.toml"
            manifest.write_text("different source manifest\n")
            with self.assertRaisesRegex(chunker.ChunkError, "manifest revision"):
                chunker.verify_manifest(manifest, REVISION)

    def test_rejects_a_record_that_exceeds_the_chunk_limit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = self.write_spec(Path(temporary), [{"id": "alpha", "lines": ["x"] * 180}])
            with self.assertRaisesRegex(chunker.ChunkError, "exceeds 180"):
                chunker.expected_bundle(path)

    def test_check_rejects_stale_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            input_path, output = self.write_spec(root, self.records()), root / "output"
            bundle = chunker.expected_bundle(input_path)
            chunker.write_bundle(output, bundle)
            (output / "metadata.json").write_text("stale\n")
            with self.assertRaisesRegex(chunker.ChunkError, "stale: metadata"):
                chunker.check_bundle(output, bundle)

    def test_check_rejects_a_stale_chunk_header(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            input_path, output = self.write_spec(root, self.records()), root / "output"
            bundle = chunker.expected_bundle(input_path)
            chunker.write_bundle(output, bundle)
            chunk = output / "chunk-0001.md"
            chunk.write_text(chunk.read_text().replace("# generator:", "# stale-generator:"))
            with self.assertRaisesRegex(chunker.ChunkError, "stale: chunk-0001"):
                chunker.check_bundle(output, bundle)


if __name__ == "__main__":
    unittest.main()
