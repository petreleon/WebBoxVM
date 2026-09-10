#!/usr/bin/env python3
"""Positive checks for conservative, capture-anchored derived input lineage."""

from __future__ import annotations

import unittest
from pathlib import Path
from unittest.mock import patch

from lineage_bind import bind
from lineage_contract import receipt, verify
from lineage_events import parse
from lineage_fixture import TRACE_SOURCE, alternate_scope, fresh_events, scope, sealed

OBSERVATION, ARTIFACT = Path("/fixture/observation"), Path("/fixture/artifact")


class LineageTest(unittest.TestCase):
    def test_binds_exec_snapshot_and_closed_writer_inputs_in_observed_order(self):
        records = bind(parse(fresh_events()), scope())
        self.assertEqual([item.identifier for item in records], ["derived-67656e6572617465642f73706563617474726962732e61646f63", "derived-67656e6572617465642f6e6578742e61646f63"])
        self.assertEqual(records[0].producer_ids, ("vulkan-14-spec",))
        self.assertEqual(records[1].producer_ids, ("vulkan-14-spec", "derived-67656e6572617465642f73706563617474726962732e61646f63"))
        self.assertEqual((records[0].writer_instance, records[1].writer_instance), ("shell", "generator"))

    def test_receipt_is_self_hashed_unadmitted_and_capture_anchored(self):
        result = sealed()
        with patch("lineage_contract.capture_scope", return_value=scope()):
            self.assertEqual(verify(result, fresh_events(), OBSERVATION, ARTIFACT, "observer-a"), result)
        self.assertEqual(result["status"], "proof-lineage-only-unadmitted")
        self.assertEqual(result["event_count"], 16)
        with patch("lineage_contract.capture_scope", return_value=alternate_scope()):
            with self.assertRaisesRegex(ValueError, "sealed capture"):
                verify(result, fresh_events(), OBSERVATION, ARTIFACT, "observer-a")

    def test_repeated_fresh_normalized_trace_has_one_identity(self):
        with patch("lineage_contract.capture_scope", return_value=scope()):
            self.assertEqual(receipt(fresh_events(), TRACE_SOURCE, OBSERVATION, ARTIFACT, "observer-a"),
                             receipt(fresh_events(), TRACE_SOURCE, OBSERVATION, ARTIFACT, "observer-a"))


if __name__ == "__main__": unittest.main()
