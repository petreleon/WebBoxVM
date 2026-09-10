#!/usr/bin/env python3
"""Focused hostile checks for the unadmitted Vulkan-Docs successor design."""

from __future__ import annotations

import copy
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import successor_inventory_proposal as proposal
import successor_inventory_transition as transition


def sealed_transition(value: dict[str, object]) -> dict[str, object]:
    result = copy.deepcopy(value)
    result["transition_sha256"] = hashlib.sha256(transition.canonical(result, "transition_sha256")).hexdigest()
    return result


def sealed_candidate(value: dict[str, object]) -> dict[str, object]:
    result = copy.deepcopy(value)
    result["candidate_sha256"] = hashlib.sha256(proposal.canonical(result)).hexdigest()
    return result


def reclose(value: dict[str, object]) -> None:
    raw = value["vulkan_docs"]["raw_members"]
    closure = value["vulkan_docs"]["closure"]
    closure["raw_closure_sha256"] = proposal.raw_closure_sha256(raw)
    closure["ordered_member_ids"] = [row["id"] for row in raw]
    for row in raw:
        row["closure_sha256"] = closure["raw_closure_sha256"]
        row["local_cache"] = f"webboxvm-graphics/f02-successor/vulkan-docs/{row['closure_sha256']}/{row['id']}/{row['sha256']}.source"


class SuccessorInventoryTransitionTest(unittest.TestCase):
    def setUp(self) -> None:
        self.design = transition.document(transition.RECORD)

    def candidate(self) -> dict[str, object]:
        raw = []
        for identifier, selector, source_hash in (("docs-root", "vkspec.adoc", transition.ROOT_SHA256),
                                                  ("docs-include", "appendices.adoc", "2" * 64)):
            raw.append({"id": identifier, "source_family": "vulkan-docs",
                        "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{proposal.REVISION}/{selector}",
                        "revision": proposal.REVISION, "sha256": source_hash, "bytes": 1,
                        "license": "shape-only-unverified", "local_cache": "pending",
                        "generated_code_role": "shape-only-no-output", "provenance": "shape-only-unverified",
                        "selector": selector, "source_role": "api-limit-format-spec", "closure_sha256": "pending"})
        closure_hash = proposal.raw_closure_sha256(raw)
        for row in raw:
            row["closure_sha256"] = closure_hash
            row["local_cache"] = f"webboxvm-graphics/f02-successor/vulkan-docs/{closure_hash}/{row['id']}/{row['sha256']}.source"
        closure = {"root_id": "docs-root", "ordered_member_ids": [row["id"] for row in raw],
                   "raw_member_count": len(raw), "raw_closure_sha256": closure_hash,
                   "authority_manifest_sha256": None, "write_lineage_sha256": None,
                   "fresh_authorized_capture": False}
        value = {"schema": 3, "contract": "vulkan-docs-successor-inventory-v3", "status": "shape-only-unadmitted",
                 "transition_sha256": transition.transition(), "predecessor_lock_sha256": transition.LOCK_SHA256,
                 "legacy_records": [dict(row) for row in transition.active_inventory().inputs],
                 "source_families": [*transition.ACTIVE_FAMILIES, "vulkan-docs"],
                 "vulkan_docs": {"source_family": "vulkan-docs", "raw_members": raw, "closure": closure,
                                 "generated_outputs_are_raw_sources": False},
                 "effects": {key: False for key in transition.EFFECTS}, "candidate_sha256": ""}
        return sealed_candidate(value)

    def reject_design(self, mutate, pattern: str) -> None:
        value = sealed_transition(self.design)
        mutate(value)
        with self.assertRaisesRegex(transition.TransitionError, pattern):
            transition.transition_value(sealed_transition(value))

    def reject_candidate(self, mutate, pattern: str) -> None:
        value = self.candidate()
        mutate(value)
        with self.assertRaisesRegex(proposal.ProposalError, pattern):
            proposal.proposal_value(sealed_candidate(value))

    def test_design_binds_the_live_predecessor_and_stays_unadmitted(self) -> None:
        self.assertEqual(transition.transition(), self.design["transition_sha256"])
        self.assertEqual(len(transition.active_inventory().inputs), 17)

    def test_design_rejects_schema_alias_cache_authority_and_effect_promotion(self) -> None:
        cases = (
            (lambda value: value["predecessor"].__setitem__("schema", True), "preserve"),
            (lambda value: value["predecessor"]["families"].pop(), "preserve"),
            (lambda value: value["successor"].__setitem__("legacy_vulkan_alias_allowed", True), "schema-v3"),
            (lambda value: value["successor"].__setitem__("active_cache_grammar_compatible", True), "schema-v3"),
            (lambda value: value["generated"].__setitem__("output_can_be_raw_source", True), "authority"),
            (lambda value: value["closure"].__setitem__("historical_counts_are_current", True), "fresh Docs"),
            (lambda value: value["effects"].__setitem__("admitted", True), "active, support, or release"),
        )
        for mutate, pattern in cases:
            with self.subTest(pattern=pattern):
                self.reject_design(mutate, pattern)

    def test_shape_only_candidate_preserves_all_legacy_records(self) -> None:
        value = self.candidate()
        self.assertEqual(proposal.proposal_value(value), value["candidate_sha256"])

    def test_candidate_rejects_drift_aliases_root_only_and_false_readiness(self) -> None:
        cases = (
            (lambda value: value["legacy_records"].pop(), "preserve every predecessor"),
            (lambda value: value["source_families"].__setitem__(-1, "vulkan"), "exactly one new family"),
            (lambda value: value["vulkan_docs"].__setitem__("generated_outputs_are_raw_sources", True), "misclassifies"),
            (lambda value: (value["vulkan_docs"]["raw_members"][0].__setitem__("id", "vulkan-registry"), value["vulkan_docs"]["closure"].__setitem__("root_id", "vulkan-registry"), reclose(value)), "ordered complete"),
            (lambda value: (value["vulkan_docs"]["raw_members"][0].__setitem__("sha256", "1" * 64), reclose(value)), "ordered complete"),
            (lambda value: value["vulkan_docs"]["raw_members"].__setitem__(0, {**value["vulkan_docs"]["raw_members"][0], "bytes": 8388609}), "8 MiB"),
            (lambda value: value["vulkan_docs"]["raw_members"].__setitem__(0, {**value["vulkan_docs"]["raw_members"][0], "immutable_url": "https://example.invalid/main"}), "immutable pinned"),
            (lambda value: value["vulkan_docs"]["raw_members"].__setitem__(1, {**value["vulkan_docs"]["raw_members"][1], "selector": "./appendices.adoc"}), "unsafe id or selector"),
            (lambda value: value["vulkan_docs"]["raw_members"][1].__setitem__("selector", "appendices.adoc?x=1"), "unsafe id or selector"),
            (lambda value: (value["vulkan_docs"]["raw_members"][1].__setitem__("selector", "vkspec.adoc"), value["vulkan_docs"]["raw_members"][1].__setitem__("immutable_url", f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{proposal.REVISION}/vkspec.adoc"), reclose(value)), "duplicate Docs"),
            (lambda value: value["vulkan_docs"]["raw_members"][0].__setitem__("license", "Apache-2.0"), "invent member authority"),
            (lambda value: value["vulkan_docs"]["raw_members"].__setitem__(0, {**value["vulkan_docs"]["raw_members"][0], "local_cache": "../escape"}), "cache path"),
            (lambda value: value["vulkan_docs"].__setitem__("raw_members", value["vulkan_docs"]["raw_members"][:1]), "root-and-nonroot"),
            (lambda value: value["vulkan_docs"]["closure"].__setitem__("fresh_authorized_capture", True), "unproven closure"),
            (lambda value: value["effects"].__setitem__("admitted", True), "self-admit"),
            (lambda value: value.__setitem__("status", "admitted"), "schema, contract, or status"),
        )
        for mutate, pattern in cases:
            with self.subTest(pattern=pattern):
                self.reject_candidate(mutate, pattern)

    def test_duplicate_oversize_fifo_symlink_and_cli_are_safe_and_read_only(self) -> None:
        watched = (transition.RECORD, transition.INVENTORY_DIR / "manifest.toml", transition.INVENTORY_DIR / "inventory.lock",
                   transition.INVENTORY_DIR / "inputs/part-0001.toml", transition.INVENTORY_DIR / "inputs/part-0002.toml",
                   transition.POLICY, transition.ANCHOR, transition.BOUNDARY)
        before = {path: path.read_bytes() for path in watched}
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            candidate = root / "candidate.json"
            candidate.write_text(json.dumps(self.candidate()))
            proposal_result = subprocess.run([sys.executable, str(HERE / "successor_inventory_proposal.py"), str(candidate)],
                                             text=True, capture_output=True, check=False,
                                             env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
            self.assertEqual(proposal_result.returncode, 0, proposal_result.stderr)
            self.assertIn("shape-only unadmitted", proposal_result.stdout)
            duplicate = root / "duplicate.json"
            duplicate.write_text(transition.RECORD.read_text().replace('"schema": 1', '"schema": 1, "schema": 1', 1))
            with self.assertRaisesRegex(transition.TransitionError, "duplicate JSON key"):
                transition.transition(duplicate)
            oversized = root / "oversized.json"
            oversized.write_bytes(b"x" * (transition.MAX_DOCUMENT_BYTES + 1))
            with self.assertRaisesRegex(transition.TransitionError, "bounded size"):
                transition.transition(oversized)
            fifo = root / "record.fifo"
            os.mkfifo(fifo)
            with self.assertRaisesRegex(transition.TransitionError, "regular file"):
                transition.transition(fifo)
            link = root / "record-link.json"
            link.symlink_to(transition.RECORD)
            with self.assertRaisesRegex(transition.TransitionError, "regular file"):
                transition.transition(link)
        result = subprocess.run([sys.executable, str(HERE / "successor_inventory_transition.py")], text=True,
                                capture_output=True, check=False, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("design-only unadmitted", result.stdout)
        self.assertEqual(before, {path: path.read_bytes() for path in watched})


if __name__ == "__main__":
    unittest.main()
