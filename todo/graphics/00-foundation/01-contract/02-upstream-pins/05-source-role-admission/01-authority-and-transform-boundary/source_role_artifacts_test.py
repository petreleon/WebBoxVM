#!/usr/bin/env python3
"""Content-addressed evidence coverage for F02.5 role artifacts."""

import hashlib
import tempfile
import unittest
from pathlib import Path

from source_role_artifacts import verify_catalog
from source_role_contract import MAX_LOCAL_BYTES, RoleError
from source_role_contract_test import catalog, suite, suite_member, transform, upstream


def sha256(value):
    return hashlib.sha256(value).hexdigest()


def write(root, record, value):
    path = root / record["artifact"]
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(value)
    record["bytes"], record["sha256"] = len(value), sha256(value)
    return path


def write_builder(root, *records):
    value = b"pinned-builder-v1"
    for record in records:
        builder = record["builder"]
        path = root / builder["artifact"]
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(value)
        builder["sha256"] = sha256(value)
    return path


def simple(root):
    source, local = upstream(), transform()
    write(root, source, b"pinned-source")
    local["inputs"][0]["sha256"] = source["sha256"]
    write(root, local, b"local-output")
    return catalog(source, local), local, write_builder(root, local)


class SourceRoleArtifactTests(unittest.TestCase):
    def test_verifies_local_output_and_pinned_builder_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            value, local, builder_path = simple(root)
            self.assertEqual(verify_catalog(value, root), ("vulkan-spec", "vk14-map"))
            (root / local["artifact"]).write_bytes(b"tampered-out")
            with self.assertRaises(RoleError):
                verify_catalog(value, root)
            write(root, local, b"local-output")
            builder_path.write_bytes(b"tampered-builder")
            with self.assertRaises(RoleError):
                verify_catalog(value, root)

    def test_rejects_an_actual_over_eight_mebibyte_transform_declared_small(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            value, local, _builder_path = simple(root)
            path = root / local["artifact"]
            path.write_bytes(b"x" * (MAX_LOCAL_BYTES + 1))
            with self.assertRaises(RoleError):
                verify_catalog(value, root)

    def test_reassembles_actual_shards_against_the_upstream_member(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            full, member = suite(), suite_member()
            source = b"abcdefghij"
            first = transform("part-0", scope="webboxvm-byte-preserving-shard", bytes=5,
                              inputs=[{"id": "member", "kind": "upstream-source", "sha256": "b" * 64, "revision": "a" * 40}],
                              shard={"source_id": "member", "source_sha256": "b" * 64, "source_bytes": 10,
                                     "index": 0, "count": 2, "offset": 0, "reassembled_sha256": "b" * 64})
            second = transform("part-1", scope="webboxvm-byte-preserving-shard", bytes=5,
                               inputs=[{"id": "member", "kind": "upstream-source", "sha256": "b" * 64, "revision": "a" * 40}],
                               shard={"source_id": "member", "source_sha256": "b" * 64, "source_bytes": 10,
                                      "index": 1, "count": 2, "offset": 5, "reassembled_sha256": "b" * 64})
            write(root, full, b"root")
            write(root, member, source)
            for piece, data in ((first, source[:5]), (second, source[5:])):
                piece["inputs"][0]["sha256"] = member["sha256"]
                piece["shard"]["source_sha256"] = member["sha256"]
                piece["shard"]["reassembled_sha256"] = member["sha256"]
                write(root, piece, data)
            value = catalog(full, member, first, second)
            write_builder(root, first, second)
            self.assertEqual(verify_catalog(value, root), ("vulkan-cts-mustpass", "member", "part-0", "part-1"))
            changed = b"fghik"
            write(root, second, changed)
            with self.assertRaises(RoleError):
                verify_catalog(value, root)


if __name__ == "__main__":
    unittest.main()
