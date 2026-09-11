#!/usr/bin/env python3
"""Hostile tests for the atomically blocked successor reconciliation."""
from __future__ import annotations
import copy, hashlib, importlib.util, json, os, tempfile, unittest
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("atomic_blocked_reconciliation", HERE / "atomic_blocked_reconciliation.py")
if SPEC is None or SPEC.loader is None: raise RuntimeError("cannot load atomic reconciliation")
MOD = importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(MOD)

class AtomicTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(); self.root = Path(self.temporary.name).resolve()
        self.addCleanup(self.temporary.cleanup)
    def record(self): return json.loads(MOD.RECORD.read_text(encoding="utf-8"))
    def write(self, value, name="aggregate.json"):
        path = self.root / name; path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8"); return path
    def sealed(self, value): value["reconciliation_sha256"] = MOD.digest(value); return value
    def rejected(self, change):
        value = self.record(); change(value)
        with self.assertRaisesRegex(MOD.AtomicError, "exact blocked aggregate"):
            MOD.validate(self.write(self.sealed(value)))

    def test_committed_record_is_read_only_offline_and_atomically_blocked(self):
        watched = tuple(MOD.PATHS.values()) + (MOD.RECORD,)
        before = tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in watched)
        with mock.patch.object(MOD.BINDING, "replay", side_effect=AssertionError("replay")) as replay, \
             mock.patch.object(MOD.BINDING.RUN, "fetch_to_cache", side_effect=AssertionError("fetch")) as fetch:
            value = MOD.validate()
        self.assertEqual((value["status"], value["global_blockers"]), ("atomically-blocked-unadmitted", [["vulkan-14-spec", "vulkan-docs-core-generated-closure-unadmitted"], ["vulkan-cts-mustpass", "vcts-vk-default-compound-oversize-core-scope-unadmitted"]]))
        self.assertEqual(value["historical_receipt"]["role"], "historical-only")
        self.assertTrue(all(item is False for item in value["effects"].values()))
        replay.assert_not_called(); fetch.assert_not_called()
        self.assertEqual(before, tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in watched))

    def test_resealed_record_rejects_order_alias_promotion_and_type_changes(self):
        changes = (
            lambda value: value.__setitem__("status", "admitted"),
            lambda value: value["documents"]["binding"].__setitem__("document_sha256", "0" * 64),
            lambda value: value["wrappers"]["families"].reverse(),
            lambda value: value["wrappers"].__setitem__("cross_wrapper_substitution_allowed", True),
            lambda value: value["wrappers"].__setitem__("active_alias_allowed", 0),
            lambda value: value["f03"]["missing_required_input_ids"].reverse(),
            lambda value: value["f03"]["missing_required_input_ids"].pop(),
            lambda value: value["global_blockers"].reverse(),
            lambda value: value["docs"].__setitem__("active_or_f03_mutation", True),
            lambda value: value["vcts"].__setitem__("local_filtering", "allowed"),
            lambda value: value["effects"].__setitem__("admitted", True),
            lambda value: value["effects"].__setitem__("admitted", 0),
        )
        for change in changes:
            with self.subTest(change=change): self.rejected(change)

    def test_duplicate_oversize_fifo_and_symlink_records_are_rejected(self):
        duplicate = self.root / "duplicate.json"; duplicate.write_text('{"schema":1,"schema":2}', encoding="utf-8")
        oversized = self.root / "oversized.json"; oversized.write_bytes(b"x" * (MOD.MAX_BYTES + 1))
        huge = self.root / "huge.json"; huge.write_text('{"schema":' + "9" * 5000 + "}", encoding="utf-8")
        nonfinite = self.root / "nonfinite.json"; nonfinite.write_text('{"schema":NaN}', encoding="utf-8")
        fifo = self.root / "aggregate.fifo"; os.mkfifo(fifo)
        link = self.root / "aggregate-link.json"; link.symlink_to(MOD.RECORD)
        for path in (duplicate, oversized, huge, nonfinite, fifo, link):
            with self.subTest(path=path.name), self.assertRaises(MOD.AtomicError): MOD.validate(path)
        valid = self.write(self.record(), "valid.json"); inside = self.root / "inside"; inside.mkdir()
        redirect = inside / "redirect"; redirect.symlink_to(self.root, target_is_directory=True)
        with self.assertRaises(MOD.AtomicError): MOD.validate(redirect / valid.name)

    def test_predecessor_edges_promotions_and_snapshot_mix_fail_closed(self):
        binding = copy.deepcopy(MOD.BINDING.validate()); binding["capture"]["closure_sha256"] = "0" * 64
        consumer = copy.deepcopy(MOD.CONSUMERS.validate()); consumer["f03"]["missing_required_input_ids"].reverse()
        integration = copy.deepcopy(MOD.INTEGRATION.validate()); integration["aggregate"]["wrapper_families"].reverse()
        handoff = copy.deepcopy(MOD.VCTS.validate()); handoff["local_filtering"] = "allowed"
        boundary = copy.deepcopy(MOD.BOUNDARY.validate()); boundary["vcts"]["f02_source_member_cap_bytes"] = 0
        receipt = copy.deepcopy(MOD.RECEIPT.validate()); receipt["receipt_sha256"] = "0" * 64
        cases = ((MOD.BINDING, "validate", binding), (MOD.CONSUMERS, "validate", consumer),
                 (MOD.INTEGRATION, "validate", integration), (MOD.DOCS, "transition", "0" * 64),
                 (MOD.VCTS, "validate", handoff), (MOD.BOUNDARY, "validate", boundary), (MOD.RECEIPT, "validate", receipt))
        for owner, name, result in cases:
            with self.subTest(edge=f"{owner.__name__}.{name}"), mock.patch.object(owner, name, return_value=result):
                with self.assertRaises(MOD.AtomicError): MOD.build()
        changed = dict(MOD.RAW); changed["docs"] = "0" * 64
        with mock.patch.object(MOD, "snapshot", side_effect=(dict(MOD.RAW), changed)):
            with self.assertRaisesRegex(MOD.AtomicError, "changed or have stale raw identities"): MOD.inputs()

if __name__ == "__main__": unittest.main(verbosity=2)
