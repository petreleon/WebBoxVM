#!/usr/bin/env python3
"""Hermetic tests for deterministic graphics chunk generation."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

import graphics_chunker as chunker
import graphics_chunk_schema as chunk_schema

V1_REVISION = "a" * 64
V2_REVISION = "b" * 64


def spec(schema: int, revision: str, records: list[dict[str, object]]) -> bytes:
    identity = ("manifest_revision", "inventory_sha256")[schema - 1]
    return (json.dumps({"schema": schema, identity: revision, "records": records}) + "\n").encode()


class ChunkerTests(unittest.TestCase):
    def write_spec(self, directory: Path, records: list[dict[str, object]], schema: int = 1, revision: str = V1_REVISION) -> Path:
        path = directory / "input.json"
        path.write_bytes(spec(schema, revision, records))
        return path

    def records(self) -> list[dict[str, object]]:
        return [
            {"id": "alpha", "lines": ["kind: fixture", "role: deterministic"]},
            {"id": "beta", "lines": ["kind: fixture", "role: provenance"]},
        ]

    def test_v1_write_and_check_are_byte_identical(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            input_path, output = self.write_spec(root, self.records()), root / "output"
            bundle = chunker.expected_bundle(input_path)
            chunker.write_bundle(output, bundle)
            first = {path.name: path.read_bytes() for path in output.iterdir()}
            chunker.check_bundle(output, bundle)
            chunker.write_bundle(output, bundle)
            self.assertEqual(first, {path.name: path.read_bytes() for path in output.iterdir()})
            metadata = json.loads((output / "metadata.json").read_text())
            self.assertEqual(metadata["manifest_revision"], V1_REVISION)

    def test_v2_uses_a_raw_lock_identity_without_parsing_composition(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            lock = root / "inventory.lock"
            lock.write_bytes(b"\x00raw lock bytes; not an inventory layout\n")
            revision = chunker.sha256(lock.read_bytes())
            input_path = self.write_spec(root, self.records(), schema=2, revision=revision)
            output, bundle = root / "output", chunker.expected_bundle(input_path)
            chunker.verify_source(lock, revision, chunk_schema.V2)
            chunker.write_bundle(output, bundle)
            first = {path.name: path.read_bytes() for path in output.iterdir()}
            chunker.check_bundle(output, bundle)
            chunker.write_bundle(output, bundle)
            self.assertEqual(first, {path.name: path.read_bytes() for path in output.iterdir()})
            metadata = json.loads((output / "metadata.json").read_text())
            self.assertEqual(metadata["inventory_sha256"], revision)
            self.assertNotIn("manifest_revision", metadata)
            self.assertIn("# generator: webboxvm.graphics-chunker.v2", (output / "chunk-0001.md").read_text())

    def test_v2_rejects_a_manifest_only_identity_field(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "input.json"
            path.write_text(json.dumps({"schema": 2, "manifest_revision": V1_REVISION, "records": self.records()}))
            with self.assertRaisesRegex(chunker.ChunkError, "fields do not match"):
                chunker.expected_bundle(path)

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
                chunker.verify_manifest(manifest, V1_REVISION)

    def test_v2_rejects_a_mismatched_raw_lock(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            lock = Path(temporary) / "inventory.lock"
            lock.write_bytes(b"a raw lock\n")
            with self.assertRaisesRegex(chunker.ChunkError, "inventory lock identity"):
                chunker.verify_source(lock, V2_REVISION, chunk_schema.V2)

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
