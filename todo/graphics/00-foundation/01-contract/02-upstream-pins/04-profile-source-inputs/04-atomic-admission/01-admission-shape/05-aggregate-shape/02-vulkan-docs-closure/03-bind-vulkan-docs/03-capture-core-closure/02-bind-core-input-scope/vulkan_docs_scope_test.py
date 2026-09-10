#!/usr/bin/env python3
"""Positive and capture-binding regressions for the core input/scope model."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from vulkan_docs_scope_bind import bind_capture, bind_value
from vulkan_docs_scope_contract import compact, receipt_value
from vulkan_docs_scope_fixture import expectation, valid
from vulkan_docs_scope_model import ScopeError
from vulkan_docs_observer_run import assemble, seal_run
from vulkan_docs_identity_build import OUTPUT, TREE
from vulkan_docs_observer_events import producer_digest
from vulkan_docs_observer_plan import SOURCE_TREE


def write(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")


def capture_run(value: dict[str, object], identifier: str, artifact: str, io: str) -> dict[str, object]:
    return seal_run({
        "id": identifier, "artifact": artifact, "source_tree_sha256": SOURCE_TREE[2], "generated_tree_sha256": TREE[2],
        "primary_html_sha256": OUTPUT["sha256"], "io_trace_sha256": io, "include_trace_sha256": "a" * 64,
        "input_manifest_sha256": value["input_manifest_sha256"], "include_identity_sha256": value["include_identity_sha256"],
        "producer_argv_sha256": producer_digest(), "raw_count": value["raw_count"], "derived_count": value["derived_count"],
        "include_count": len(value["includes"]), "phase_counts": value["phase_counts"],
    })


def write_capture(root: Path, value: dict[str, object]) -> Path:
    paths = []
    for identifier, io in (("fixture-a", "4" * 64), ("fixture-b", "5" * 64)):
        artifact = f"runs/{identifier}"
        directory = root / artifact / "observer"
        write(directory / "normalized-inputs.json", value)
        run = capture_run(value, identifier, artifact, io)
        path = directory / "run.json"
        write(path, run)
        paths.append(path)
    observation = root / "observation.json"
    assemble(paths, observation)
    return observation


class ScopeBindingTest(unittest.TestCase):
    def test_valid_scope_and_receipt_remain_unadmitted(self):
        value, capture = valid()
        result = bind_value(value, capture)
        self.assertEqual(result.state, "input-scope-only-unadmitted")
        self.assertFalse(result.admitted)
        self.assertFalse(result.cutover_ready)
        self.assertIs(receipt_value(compact(result), result), result)

    def test_selected_artifact_run_must_match_the_tracked_header(self):
        value, _ = valid()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            observation = write_capture(root, value)
            result = bind_capture(observation, root, "fixture-a")
            self.assertEqual(result.capture.identifier, "fixture-a")
            stale = root / "runs/fixture-b/observer/run.json"
            (root / "runs/fixture-a/observer/run.json").write_bytes(stale.read_bytes())
            with self.assertRaises(ScopeError):
                bind_capture(observation, root, "fixture-a")

    def test_fixture_expectation_matches_the_normalized_identities(self):
        value, capture = valid()
        self.assertEqual(capture, expectation(value))


if __name__ == "__main__":
    unittest.main()
