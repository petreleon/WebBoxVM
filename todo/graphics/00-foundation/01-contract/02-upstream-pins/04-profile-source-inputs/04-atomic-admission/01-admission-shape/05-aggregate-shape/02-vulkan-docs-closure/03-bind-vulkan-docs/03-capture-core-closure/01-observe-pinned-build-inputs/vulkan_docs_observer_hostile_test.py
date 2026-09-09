#!/usr/bin/env python3
"""Hostile records must fail before an observer header is exposed."""

from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path

from vulkan_docs_observer_contract import observation_value
from vulkan_docs_observer_fixture import dump, observation, source_tree, traces
from vulkan_docs_observer_model import ObserverError
from vulkan_docs_observer_normalize import normalize
from vulkan_docs_observer_parse import canonical


def reseal(value):
    for item in value["runs"]:
        payload = dict(item)
        payload.pop("run_sha256")
        item["run_sha256"] = canonical(payload, "webboxvm-graphics-vulkan-docs-observation-run-v1")
    payload = dict(value)
    payload.pop("observation_sha256")
    value["observation_sha256"] = canonical(payload, "webboxvm-graphics-vulkan-docs-observation-v1")


class DocsObserverHostileTest(unittest.TestCase):
    def capture(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        source, generated = source_tree(root)
        io, includes = traces(root)
        return source, generated, io, includes

    def assert_header_rejected(self, change):
        value = observation()
        change(value)
        with self.assertRaises(ObserverError):
            observation_value(value)

    def test_header_rejects_active_states_and_stale_digests(self):
        self.assert_header_rejected(lambda value: value.__setitem__("status", "admitted"))
        self.assert_header_rejected(lambda value: value.__setitem__("observation_sha256", "f" * 64))

    def test_header_rejects_wrong_observer_or_build_witness(self):
        self.assert_header_rejected(lambda value: value.__setitem__("observer_source_sha256", "f" * 64))
        self.assert_header_rejected(lambda value: value.__setitem__("build_witness_sha256", "f" * 64))

    def test_header_rejects_duplicate_or_non_artifact_runs(self):
        def repeated(value):
            value["runs"][1] = copy.deepcopy(value["runs"][0])
        self.assert_header_rejected(repeated)
        self.assert_header_rejected(lambda value: value["runs"][0].__setitem__("artifact", "/tmp/run"))

    def test_header_rejects_missing_phase_and_bad_output_identity(self):
        self.assert_header_rejected(lambda value: value["runs"][0]["phase_counts"].pop("generator"))
        self.assert_header_rejected(lambda value: value["runs"][0].__setitem__("generated_tree_sha256", "f" * 64))

    def test_header_rejects_impossible_phase_counts_and_reused_artifacts(self):
        def impossible(value):
            value["runs"][0]["raw_count"], value["runs"][0]["derived_count"] = 1, 1
            value["runs"][0]["phase_counts"] = {name: 3 for name in value["runs"][0]["phase_counts"]}
            reseal(value)
        self.assert_header_rejected(impossible)
        def reused(value):
            value["runs"][1]["artifact"] = value["runs"][0]["artifact"]
            reseal(value)
        self.assert_header_rejected(reused)

    def test_normalizer_rejects_unknown_process_reads(self):
        source, generated, io, includes = self.capture()
        rows = [json.loads(row) for row in io.read_text().splitlines()]
        row = next(row for row in rows if row.get("kind") != "start")
        row["pid"] = 99
        io.write_text("\n".join(json.dumps(row) for row in rows) + "\n")
        with self.assertRaises(ObserverError):
            normalize(source, generated, io, includes)

    def test_normalizer_rejects_escaping_and_python_cache_paths(self):
        source, generated, io, includes = self.capture()
        rows = [json.loads(row) for row in io.read_text().splitlines()]
        row = next(row for row in rows if row.get("kind") != "start")
        row["path_hex"] = "/vulkan/../escape".encode().hex()
        io.write_text("\n".join(json.dumps(row) for row in rows) + "\n")
        with self.assertRaises(ObserverError):
            normalize(source, generated, io, includes)
        source, generated, io, includes = self.capture()
        rows = [json.loads(row) for row in io.read_text().splitlines()]
        row = next(row for row in rows if row.get("kind") != "start")
        row["path_hex"] = "/vulkan/scripts/__pycache__/x.pyc".encode().hex()
        io.write_text("\n".join(json.dumps(row) for row in rows) + "\n")
        with self.assertRaises(ObserverError):
            normalize(source, generated, io, includes)

    def test_normalizer_rejects_unread_or_malformed_include(self):
        source, generated, io, includes = self.capture()
        dump(includes, [{"file": "/vulkan/missing.adoc", "path": "/vulkan/missing.adoc", "line": 1}])
        with self.assertRaises(ObserverError):
            normalize(source, generated, io, includes)
        source, generated, io, includes = self.capture()
        includes.write_text('{"file":"/vulkan/vkspec.adoc","file":"duplicate","path":"/vulkan/vkspec.adoc","line":1}\n')
        with self.assertRaises(ObserverError):
            normalize(source, generated, io, includes)

    def test_normalizer_rejects_fake_producer_and_non_asciidoctor_include(self):
        source, generated, io, includes = self.capture()
        rows = io.read_text().splitlines()
        first = json.loads(rows[0])
        first["argv_hex"] = b"echo\0makeSpec".hex()
        rows[0] = json.dumps(first)
        io.write_text("\n".join(rows) + "\n")
        with self.assertRaises(ObserverError):
            normalize(source, generated, io, includes)
        source, generated, io, includes = self.capture()
        dump(includes, [{"file": "/vulkan/config/khronos.css", "path": "/vulkan/config/khronos.css", "line": 1}])
        with self.assertRaises(ObserverError):
            normalize(source, generated, io, includes)

    def test_normalizer_rejects_a_post_read_generated_change(self):
        source, generated, io, includes = self.capture()
        rows = [json.loads(row) for row in io.read_text().splitlines()]
        derived = next(row for row in rows if row.get("path_hex") == "/work/generated/specattribs.adoc".encode().hex())
        derived["observed_mtime_ns"] -= 1_000_000_000
        io.write_text("\n".join(json.dumps(row) for row in rows) + "\n")
        with self.assertRaises(ObserverError):
            normalize(source, generated, io, includes)


if __name__ == "__main__":
    unittest.main()
