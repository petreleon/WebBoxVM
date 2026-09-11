#!/usr/bin/env python3
"""Hermetic capture/replay tests for the frozen six-member GLES closure."""

from __future__ import annotations

import hashlib
import json
import os
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET
from pathlib import Path, PurePosixPath

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gles_capture as RUN
import gles_capture_marker as MARKER
import gles_capture_plan as PLAN
from source_cache import atomic_store
from source_model import ContractError, ExternalCache, SourceInput


class Response:
    def __init__(self, url: str, data: bytes): self.url, self.data, self.offset = url, data, 0
    def __enter__(self): return self
    def __exit__(self, *_): return None
    def geturl(self): return self.url
    def getcode(self): return 200
    def read(self, size=-1):
        end = len(self.data) if size < 0 else self.offset + size
        value, self.offset = self.data[self.offset:end], min(end, len(self.data))
        return value


class Opener:
    def __init__(self, values): self.values, self.calls = values, []
    def open(self, request, timeout):
        self.calls.append(request.full_url)
        return Response(request.full_url, self.values[request.full_url])


class CaptureTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)

    def cache(self, name):
        root = Path(self.temporary.name)
        return ExternalCache.from_path(root / name, root / "repository")

    def fixture(self, root_data=None):
        base = PLAN.plan()
        root = ET.Element("Mustpass", {"version": "main"})
        package = ET.SubElement(root, "TestPackage", {"name": "Khronos Mustpass ES"})
        for row in (*base.core_configurations, *base.excluded_configurations):
            ET.SubElement(package, "Configuration", PLAN.configuration(row))
        values = [root_data if root_data is not None else ET.tostring(root, encoding="utf-8", xml_declaration=True)]
        values.extend(f"fixture-{index}".encode() for index in range(1, 6))
        sources = []
        for index, (basis, value) in enumerate(zip(base.sources, values)):
            digest, revision = hashlib.sha256(value).hexdigest(), "a" * 40
            sources.append(SourceInput(basis.identifier, f"https://raw.githubusercontent.com/example/fixture/{revision}/{index}",
                                       revision, digest, len(value), PurePosixPath(f"webboxvm-graphics/f02/{basis.identifier}/{digest}.source")))
        current = PLAN.CapturePlan("b" * 64, "c" * 64, "d" * 64, tuple(sources), base.roles,
                                   base.core_configurations, base.excluded_configurations)
        return current, {source.url: value for source, value in zip(sources, values)}

    def test_real_plan_is_six_members_with_twelve_core_configurations(self):
        current = PLAN.plan()
        self.assertEqual((len(current.sources), len(current.core_configurations), len(current.excluded_configurations)), (6, 12, 1))
        self.assertEqual(current.roles, ("root", "core", "core", "core", "core", "excluded-extension"))

    def test_capture_publishes_last_and_replay_never_fetches(self):
        current, values = self.fixture()
        cache = self.cache("positive")
        opener = Opener(values)
        target = RUN.capture(cache, opener=opener, current=current)
        self.assertTrue(target.is_file())
        self.assertEqual(len(opener.calls), 6)
        original, RUN.fetch_to_cache = RUN.fetch_to_cache, lambda *_: self.fail("replay fetched")
        try: self.assertEqual(RUN.replay(cache, current), target)
        finally: RUN.fetch_to_cache = original

    def test_invalid_xml_leaves_partial_cache_without_a_marker(self):
        current, values = self.fixture(b"<!DOCTYPE Mustpass><Mustpass/>")
        cache = self.cache("bad-root")
        with self.assertRaisesRegex(ContractError, "unsafe XML"):
            RUN.capture(cache, opener=Opener(values), current=current)
        self.assertTrue(cache.target(current.root).is_file())
        self.assertFalse(MARKER.path(cache, current).exists())

    def test_root_parser_rejects_reordered_configurations(self):
        current, values = self.fixture()
        root = ET.fromstring(values[current.root.url])
        package, nodes = root[0], list(root[0])
        for node in nodes: package.remove(node)
        for node in reversed(nodes): package.append(node)
        with self.assertRaisesRegex(ContractError, "reordered"):
            PLAN.validate_root(ET.tostring(root, encoding="utf-8"), current)

    def test_root_only_and_substituted_replay_are_rejected(self):
        current, values = self.fixture()
        partial = self.cache("partial")
        atomic_store(partial, current.root, values[current.root.url])
        marker = MARKER.path(partial, current)
        marker.parent.mkdir(parents=True)
        marker.write_text(json.dumps(MARKER.value(current)), encoding="utf-8")
        with self.assertRaisesRegex(ContractError, "missing"):
            RUN.replay(partial, current)
        cache = self.cache("substituted")
        target = RUN.capture(cache, opener=Opener(values), current=current)
        source = current.sources[1]
        cache.target(source).write_bytes(b"x" * source.byte_count)
        with self.assertRaisesRegex(ContractError, "sha256 mismatch"):
            RUN.replay(cache, current)
        self.assertTrue(target.exists())

    def test_marker_mutation_symlink_and_republish_are_rejected(self):
        current, values = self.fixture()
        cache = self.cache("hostile")
        RUN.capture(cache, opener=Opener(values), current=current)
        with self.assertRaisesRegex(ContractError, "already exists"):
            MARKER.publish(cache, current)
        target = cache.target(current.sources[1])
        target.unlink()
        target.symlink_to(cache.target(current.root))
        with self.assertRaisesRegex(ContractError, "cannot safely open"):
            RUN.replay(cache, current)
        target.unlink()
        os.mkfifo(target)
        with self.assertRaisesRegex(ContractError, "exact regular file"):
            RUN.replay(cache, current)
        target.unlink()
        marker = MARKER.path(cache, current)
        marker.unlink()
        marker.symlink_to(cache.target(current.root))
        with self.assertRaisesRegex(ContractError, "cannot be safely opened"):
            RUN.replay(cache, current)
        marker.unlink()
        os.mkfifo(marker)
        with self.assertRaisesRegex(ContractError, "bounded regular file"):
            RUN.replay(cache, current)
        marker.unlink()
        marker.write_text("{}", encoding="utf-8")
        with self.assertRaisesRegex(ContractError, "self-hash or schema"):
            RUN.replay(cache, current)


if __name__ == "__main__":
    unittest.main(verbosity=2)
