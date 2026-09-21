#!/usr/bin/env python3
"""Focused hostile checks for the F03.4.2.4 aggregate receipt."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "vulkan_no_claim_provenance_receipt.py"


def load():
    spec = importlib.util.spec_from_file_location("f03424_receipt_test", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def artifact(folder: str, name: str) -> dict[str, object]:
    return json.loads((HERE.parent / folder / name).read_text(encoding="utf-8"))


AUTH = load()
INPUTS = (artifact("01-source-channel-boundary", "vulkan_source_channels.json"),
          artifact("02-raw-docs-provenance", "vulkan_raw_docs_citations.json"),
          artifact("03-full-suite-diagnostics", "vulkan_full_suite_diagnostics.json"),
          json.loads((HERE / "vulkan_no_claim_provenance_receipt.json").read_text(encoding="utf-8"))["registry"])


class NoClaimReceiptTests(unittest.TestCase):
    def built(self) -> dict[str, object]:
        with patch.object(AUTH, "source_inputs", return_value=copy.deepcopy(INPUTS)):
            return AUTH.build(Path("/private/tmp/f03424-selector"), Path("/private/tmp/f03424-docs"))

    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory(); path = Path(temporary.name) / "receipt.json"
        path.write_text(json.dumps(self.built()), encoding="utf-8")
        return temporary, path

    def checked(self, path: Path) -> dict[str, object]:
        with patch.object(AUTH, "source_inputs", return_value=copy.deepcopy(INPUTS)):
            return AUTH.validate(path, Path("/private/tmp/f03424-selector"), Path("/private/tmp/f03424-docs"))

    def mutate(self, path: Path, edit, rehash: bool = False) -> None:
        value = json.loads(path.read_text(encoding="utf-8")); edit(value)
        if rehash:
            body = {key: item for key, item in value.items() if key != "receipt_sha256"}
            value["receipt_sha256"] = hashlib.sha256(AUTH.canonical(body)).hexdigest()
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_receipt_remains_raw_only_and_blocked(self) -> None:
        value = self.built()
        self.assertEqual((value["registry"]["raw_row_count"], value["registry"]["source_order"]), (1458, "1..1458"))
        self.assertEqual((value["docs"]["citation_candidate_count"], value["vcts"]["member_count"]), (2, 98))
        self.assertEqual((value["matrix_rows"], value["matrix_row_count"], value["cts_executions"]), ([], 0, 0))
        self.assertEqual(value["registry"]["row_policy"]["blocker"], AUTH.SOURCE.ROW_BLOCKER)
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual(value["states"]["profile_status"], "blocked")

    def test_stale_mixed_or_promoted_receipts_fail_closed(self) -> None:
        edits = (
            (lambda value: value.update(receipt_sha256="0" * 64), False),
            (lambda value: value.update(source_contract_sha256="0" * 64), True),
            (lambda value: value["registry"].update(admission="normative-root"), True),
            (lambda value: value["registry"].update(raw_row_count=1457), True),
            (lambda value: value["registry"]["row_policy"].update(blocker="reassigned"), True),
            (lambda value: value["docs"].update(map_role="matrix-import"), True),
            (lambda value: value["vcts"].update(matrix_row_count=1), True),
            (lambda value: value["claims"].update(guest_execution=True), True),
            (lambda value: value["matrix_rows"].append({"promoted": True}), True),
        )
        for edit, rehash in edits:
            temporary, path = self.copied()
            with temporary, self.subTest(edit=edit), self.assertRaises(AUTH.ReceiptError):
                self.mutate(path, edit, rehash); self.checked(path)

    def test_mixed_prerequisite_pins_fail_before_aggregation(self) -> None:
        boundary, docs, diagnostic, registry = copy.deepcopy(INPUTS); docs["source_contract_sha256"] = "0" * 64
        with patch.object(AUTH.CHANNEL, "validate", return_value=boundary), patch.object(AUTH.DOCS, "validate", return_value=docs), \
                patch.object(AUTH.VCTS, "validate", return_value=diagnostic), patch.object(AUTH.SOURCE, "registry_fact", return_value=registry), \
                self.assertRaises(AUTH.ReceiptError):
            AUTH.source_inputs(Path("/private/tmp/f03424-selector"), Path("/private/tmp/f03424-docs"))

    def test_docs_and_vcts_scope_promotions_fail_closed(self) -> None:
        edits = (lambda docs, vcts: docs["scope"].update(map_role="matrix-import"),
                 lambda docs, vcts: docs["scope"].update(include_closure="admitted"),
                 lambda docs, vcts: docs["matrix_rows"].append({"promoted": True}),
                 lambda docs, vcts: vcts["full_suite_root"].update(permitted_use="core-only-selector"),
                 lambda docs, vcts: vcts["diagnostic_scope"].update(matrix_row_count=1))
        for edit in edits:
            boundary, docs, vcts, registry = copy.deepcopy(INPUTS); edit(docs, vcts)
            with self.subTest(edit=edit), patch.object(AUTH, "source_inputs", return_value=(boundary, docs, vcts, registry)), \
                    self.assertRaises(AUTH.ReceiptError):
                AUTH.build(Path("/private/tmp/f03424-selector"), Path("/private/tmp/f03424-docs"))

    def test_missing_duplicate_and_reordered_raw_rows_fail_closed(self) -> None:
        duplicate = {"rows": [{"source_order": 1, "source_locator": "same"}, {"source_order": 2, "source_locator": "same"}]}
        reordered = {"rows": [{"source_order": 2, "source_locator": "one"}]}
        for value in (duplicate, reordered):
            with self.subTest(value=value), self.assertRaises(AUTH.ReceiptError):
                AUTH.SOURCE.registry_summary(value)

    def test_registry_effect_schema_is_exact(self) -> None:
        row = {"source_order": 1, "source_locator": "one", "status": "blocked",
               "implementation_owner": None, "independent_test_plan": None, "blocker": AUTH.SOURCE.ROW_BLOCKER}
        digest, effects = hashlib.sha256(AUTH.canonical([row])).hexdigest(), {name: False for name in AUTH.SOURCE.EFFECTS}
        value = {"schema": 2, "rows": [row], "rows_sha256": digest, "source_contract_sha256": AUTH.SOURCE.SOURCE_CONTRACT,
                 "inventory_lock_sha256": AUTH.SOURCE.INVENTORY_LOCK, "contract": "vulkan-registry-technical-inventory-v2",
                 "status": "blocked", "scope": AUTH.SOURCE.INVENTORY_SCOPE, "boundaries": AUTH.SOURCE.BOUNDARIES,
                 "registry": copy.deepcopy(AUTH.SOURCE.REGISTRY_ID), "effects": effects}
        with patch.object(AUTH.SOURCE, "ROWS", 1), patch.object(AUTH.SOURCE, "ROWS_SHA256", digest):
            self.assertEqual(AUTH.SOURCE.registry_summary(value)["raw_row_count"], 1)
            value["effects"].pop("near_native")
            with self.assertRaises(AUTH.ReceiptError):
                AUTH.SOURCE.registry_summary(value)
            value["effects"] = effects; value["matrix_rows"] = []
            with self.assertRaises(AUTH.ReceiptError):
                AUTH.SOURCE.registry_summary(value)
            value.pop("matrix_rows")
            invalid = (lambda item: item.update(schema=3),
                       lambda item: item["registry"].update(claim=True),
                       lambda item: item["effects"].update(supported=0),
                       lambda item: item["registry"].update(bytes=3309653.0))
            for edit in invalid:
                with self.subTest(edit=edit), self.assertRaises(AUTH.ReceiptError):
                    candidate = copy.deepcopy(value); edit(candidate); AUTH.SOURCE.registry_summary(candidate)

    def test_cache_and_subprocess_failures_do_not_fall_back(self) -> None:
        with self.assertRaises(AUTH.ReceiptError):
            AUTH.SOURCE.external_root(Path("relative"), "selector cache")
        failed = types.SimpleNamespace(returncode=2, stdout="", stderr="cache rejected")
        with patch.object(AUTH.SOURCE.subprocess, "run", return_value=failed), self.assertRaises(AUTH.ReceiptError):
            AUTH.SOURCE.registry_fact(Path("/private/tmp/f03424-selector"))

    def test_validation_requires_both_external_caches(self) -> None:
        with self.assertRaisesRegex(AUTH.ReceiptError, "selector and Docs caches"):
            AUTH.validate(HERE / "vulkan_no_claim_provenance_receipt.json")


if __name__ == "__main__":
    unittest.main()
