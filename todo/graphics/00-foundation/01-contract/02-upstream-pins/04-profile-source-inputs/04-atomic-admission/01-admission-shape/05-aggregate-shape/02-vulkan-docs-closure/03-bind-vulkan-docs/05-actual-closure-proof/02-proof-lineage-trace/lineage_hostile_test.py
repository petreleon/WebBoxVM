#!/usr/bin/env python3
"""Hostile lineage traces must fail before an actual Docs closure can exist."""

from __future__ import annotations

import copy
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

from lineage_bind import bind
from lineage_contract import _receipt_with_scope, receipt, verify
from lineage_events import parse
from lineage_fixture import TRACE_SOURCE, fresh_events, scope, sealed
from lineage_model import LineageError

OBSERVATION, ARTIFACT = Path("/fixture/observation"), Path("/fixture/artifact")


class LineageHostileTest(unittest.TestCase):
    def assert_rejected(self, change):
        value = fresh_events(); change(value)
        with self.assertRaises(LineageError): bind(parse(value), scope())

    def test_rejects_missing_producer_and_unread_final(self):
        self.assert_rejected(lambda value: value[2].__setitem__("parent_instance_id", None))
        self.assert_rejected(lambda value: value.pop(13))

    def test_rejects_duplicate_writer_and_mutation_after_close(self):
        self.assert_rejected(lambda value: value[10].__setitem__("path", "generated/specattribs.adoc"))
        def mutate(value):
            for row in value:
                if row["sequence"] >= 8: row["sequence"] += 1
            value.insert(7, {"kind": "write", "sequence": 8, "instance_id": "shell", "object_id": "object-spec", "path": "generated/specattribs.adoc"})
        self.assert_rejected(mutate)

    def test_rejects_unsafe_or_unbound_temp_and_rename_paths(self):
        self.assert_rejected(lambda value: value[4].__setitem__("path", "outside/spec.tmp"))
        self.assert_rejected(lambda value: value[6].__setitem__("source_path", "temporary/missing.tmp"))
        self.assert_rejected(lambda value: value[6].__setitem__("target_path", "generated/out/spec.adoc"))

    def test_rejects_final_before_close_and_derived_read_before_finalization(self):
        self.assert_rejected(lambda value: value.pop(5))
        self.assert_rejected(lambda value: value[8].__setitem__("sha256", "f" * 64))
        self.assert_rejected(lambda value: value[8].__setitem__("input_id", "generated-next"))

    def test_rejects_raw_scope_expansion_identity_change_and_omission(self):
        def expand(value):
            value[1].update({"input_id": "raw-6574632f706173737764", "selector": "etc/passwd", "sha256": "9" * 64, "bytes": 1})
        self.assert_rejected(expand)
        self.assert_rejected(lambda value: value[1].__setitem__("sha256", "9" * 64))
        self.assert_rejected(lambda value: value.pop(3))

    def test_rejects_pid_reuse_and_events_after_exec(self):
        self.assert_rejected(lambda value: value[2].__setitem__("process_id", "pid-make"))
        self.assert_rejected(lambda value: value[10].__setitem__("instance_id", "shell"))

    def test_rejects_duplicate_or_changed_member_identity(self):
        self.assert_rejected(lambda value: value[7].__setitem__("input_id", "generated-next"))
        self.assert_rejected(lambda value: value[13].__setitem__("bytes", 99))

    def test_rejects_a_noncanonical_id_for_its_selector(self):
        self.assert_rejected(lambda value: value[7].__setitem__("input_id", "generated-specattribs"))

    def test_rejects_stale_or_active_receipt(self):
        result = sealed(); changed = copy.deepcopy(result); changed["status"] = "admitted"
        with patch("lineage_contract.capture_scope", return_value=scope()):
            with self.assertRaises(LineageError): verify(changed, fresh_events(), OBSERVATION, ARTIFACT, "observer-a")
        changed = copy.deepcopy(result); changed["lineage_sha256"] = "f" * 64
        with patch("lineage_contract.capture_scope", return_value=scope()):
            with self.assertRaises(LineageError): verify(changed, fresh_events(), OBSERVATION, ARTIFACT, "observer-a")

    def test_rejects_forged_scope_anchor_and_duplicate_scope_ids(self):
        forged = replace(scope(), observation_sha256="a" * 64, manifest_sha256="b" * 64, scope_identity_sha256="c" * 64)
        forged_receipt = _receipt_with_scope(fresh_events(), TRACE_SOURCE, forged)
        with patch("lineage_contract.capture_scope", return_value=scope()):
            with self.assertRaises(LineageError): verify(forged_receipt, fresh_events(), OBSERVATION, ARTIFACT, "observer-a")
        with self.assertRaises(LineageError): receipt(fresh_events(), TRACE_SOURCE, forged, ARTIFACT, "observer-a")
        duplicate = replace(scope(), raw_inputs=tuple(sorted((*scope().raw_inputs, ("vulkan-14-spec", "vkspec.adoc", "9" * 64, 11)))))
        with self.assertRaises(LineageError): bind(parse(fresh_events()), duplicate)


if __name__ == "__main__": unittest.main()
