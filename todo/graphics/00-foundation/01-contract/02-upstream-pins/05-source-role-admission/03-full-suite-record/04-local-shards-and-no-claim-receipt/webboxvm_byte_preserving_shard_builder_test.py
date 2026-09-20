#!/usr/bin/env python3
"""Hermetic streaming tests for the local byte-preserving shard builder."""

import hashlib
import io
import tempfile
import unittest
from contextlib import redirect_stderr
from pathlib import Path

import webboxvm_byte_preserving_shard_builder as builder


def digest(data):
    return hashlib.sha256(data).hexdigest()


class BytePreservingShardBuilderTests(unittest.TestCase):
    def test_streams_one_exact_source_interval(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            data, source, output = b"0123456789abcdef", root / "source", root / "out/shard"
            source.write_bytes(data)
            self.assertEqual(builder.write_shard(source, output, 5, 7, len(data), digest(data), cap=7),
                             (7, digest(data[5:12])))
            self.assertEqual(output.read_bytes(), data[5:12])
            self.assertEqual(source.read_bytes(), data)

    def test_rejects_changed_source_oversize_existing_and_symlinked_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); source = root / "source"; source.write_bytes(b"abcdefghij")
            output = root / "output"
            cases = ((0, 9, 10, digest(b"abcdefghij"), 8), (0, 5, 10, "0" * 64, 8),
                     (7, 4, 10, digest(b"abcdefghij"), 8))
            for offset, length, bytes, sha256, cap in cases:
                with self.subTest(offset=offset, length=length), self.assertRaises(builder.BuildError):
                    builder.write_shard(source, output, offset, length, bytes, sha256, cap)
            output.write_bytes(b"exists")
            with self.assertRaises(builder.BuildError):
                builder.write_shard(source, output, 0, 5, 10, digest(b"abcdefghij"), 8)
            output.unlink(); link = root / "link"; link.symlink_to(source)
            with self.assertRaises(builder.BuildError):
                builder.write_shard(link, output, 0, 5, 10, digest(b"abcdefghij"), 8)

    def test_only_the_separate_shard_mode_is_accepted(self):
        with redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as error:
            builder.main(["--mode=core-definition", "--source=/absent", "--index=0", "--count=1",
                          "--offset=0", "--bytes=1", "--output=/absent-output"])
        self.assertEqual(error.exception.code, 2)


if __name__ == "__main__":
    unittest.main()
