#!/usr/bin/env python3
"""Hermetic hostile tests for the blocked Vulkan registry fact inventory."""

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

import registry_inventory_contract as contract
import registry_inventory_validation as validation
from registry_inventory_source import RegistryError, MAX_MEMBER_BYTES, payload_bytes

HERE = Path(__file__).resolve().parent


def fixture() -> bytes:
    members = {
        ("BASE_", 0): '<require><command name="vkBase"/><type name="VkBase"/></require><deprecate><type name="VkOld"/></deprecate>',
        ("COMPUTE_", 0): '<require><enum name="VK_COMPUTE"/></require>',
        ("GRAPHICS_", 0): '<require><comment>ignored metadata</comment><feature name="graphicsFeature"/></require>',
        ("", 4): ('<require comment="features"><feature name="pipelineProtectedAccess" struct="One"/></require>'
                  '<require depends="protected"><feature name="pipelineProtectedAccess" struct="Two"/></require>'),
    }
    def feature(prefix: str, minor: int) -> str:
        internal = ' apitype="internal"' if prefix else ""
        return f'<feature api="vulkan"{internal} name="VK_{prefix}VERSION_1_{minor}" number="1.{minor}">{members.get((prefix, minor), "")}</feature>'
    features = ''.join(feature(prefix, minor) for minor in range(5)
                       for prefix in ("BASE_", "COMPUTE_", "GRAPHICS_", ""))
    return f'<registry>{features}<extensions><extension><require><command name="vkExtension"/></require></extension></extensions></registry>'.encode()


def value(rows: list[dict[str, object]], identity: dict[str, object]) -> dict[str, object]:
    return {"schema": 1, "contract": "vulkan-registry-technical-inventory-v1", "status": "blocked",
            "registry": identity, "scope": "raw-direct-cumulative-vulkan-1.0-through-1.4-structural-blocks",
            "boundaries": list(contract.BOUNDARIES), "rows": rows,
            "rows_sha256": contract.rows_sha256(rows),
            "effects": {effect: False for effect in contract.EFFECTS}}


def changed(document: dict[str, object], trail: list[str], replacement: object) -> dict[str, object]:
    value, cursor = copy.deepcopy(document), None
    cursor = value
    for key in trail[:-1]:
        cursor = cursor[key]
    cursor[trail[-1]] = replacement
    value["scaffold_sha256"] = hashlib.sha256(validation.canonical(value)).hexdigest()
    return value


class RegistryInventoryTests(unittest.TestCase):
    def expected(self, rows: list[dict[str, object]]) -> tuple[int, str]:
        previous = validation.EXPECTED_COUNT, validation.EXPECTED_ROWS_SHA256
        validation.EXPECTED_COUNT, validation.EXPECTED_ROWS_SHA256 = len(rows), contract.rows_sha256(rows)
        return previous

    def test_extracts_all_structural_blocks_and_preserves_lifecycle_and_duplicates(self) -> None:
        rows = contract.rows_from_raw(fixture())
        markers = [row["name"] for row in rows if row["requirement_kind"] == "version-marker"]
        self.assertEqual(markers, [name for name, _ in contract.FEATURES])
        self.assertEqual([row["source_order"] for row in rows], list(range(1, len(rows) + 1)))
        self.assertTrue({"command", "enum", "feature", "type"} <= {row["requirement_kind"] for row in rows})
        self.assertIn("deprecate", [row["container_kind"] for row in rows])
        protected = [row for row in rows if row["name"] == "pipelineProtectedAccess"]
        self.assertEqual(len(protected), 2)
        self.assertNotEqual(protected[0]["condition"], protected[1]["condition"])
        self.assertNotIn("vkExtension", [row["name"] for row in rows])

    def test_rejects_non_utf8_dtd_missing_or_reordered_blocks_and_unknown_members(self) -> None:
        bad = (b'<!DOCTYPE registry><registry/>', '<!DOCTYPE registry [<!ENTITY x "x">]><registry/>'.encode("utf-16"),
               b'<wrong/>', fixture().replace(b'VK_VERSION_1_3', b'VK_VERSION_1_X'),
               fixture().replace(b'VK_BASE_VERSION_1_0', b'VK_COMPUTE_VERSION_1_0', 1),
               fixture().replace(b'<deprecate>', b'<remove>', 1),
               fixture().replace(b'<command name="vkBase"/>', b'<bogus name="x"/>', 1),
               b" " * (contract.MAX_SERIALIZED + 1))
        for raw in bad:
            with self.assertRaises(RegistryError):
                contract.rows_from_raw(raw)

    def test_rejects_partial_reordered_duplicate_promoted_or_assigned_rows(self) -> None:
        identity, rows = {"source_id": "fixture"}, contract.rows_from_raw(fixture())
        previous = self.expected(rows)
        try:
            baseline = value(copy.deepcopy(rows), identity)
            validation.validate_inventory(baseline, identity)
            partial = value(copy.deepcopy(rows[:-1]), identity)
            with self.assertRaisesRegex(RegistryError, "partial"):
                validation.validate_inventory(partial, identity)
            for altered, change in ((list(reversed(copy.deepcopy(rows))), lambda records: None),
                                    (copy.deepcopy(rows), lambda records: records[0].update(status="supported")),
                                    (copy.deepcopy(rows), lambda records: records[0].update(implementation_owner="F99.1"))):
                change(altered)
                validation.EXPECTED_COUNT, validation.EXPECTED_ROWS_SHA256 = len(altered), contract.rows_sha256(altered)
                with self.assertRaises(RegistryError):
                    validation.validate_inventory(value(altered, identity), identity)
            duplicate = copy.deepcopy(rows) + [copy.deepcopy(rows[0])]
            duplicate[-1]["source_order"] = len(duplicate)
            validation.EXPECTED_COUNT, validation.EXPECTED_ROWS_SHA256 = len(duplicate), contract.rows_sha256(duplicate)
            with self.assertRaisesRegex(RegistryError, "duplicate source locators"):
                validation.validate_inventory(value(duplicate, identity), identity)
        finally:
            validation.EXPECTED_COUNT, validation.EXPECTED_ROWS_SHA256 = previous

    def test_scaffold_rejects_identity_promotion_duplicate_keys_and_type_aliases(self) -> None:
        original = json.loads(validation.SCAFFOLD.read_text(encoding="utf-8"))
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "scaffold.json"
            path.write_text('{"schema":1,"schema":1}', encoding="utf-8")
            with self.assertRaisesRegex(RegistryError, "duplicate"):
                validation.validate_scaffold(path)
            cases = ((["registry", "revision"], "stale"), (["effects", "supported"], 0), (["schema"], True), (["schema"], 1.0),
                     (["registry", "bytes"], float(original["registry"]["bytes"])))
            for trail, replacement in cases:
                path.write_text(json.dumps(changed(original, trail, replacement)), encoding="utf-8")
                with self.assertRaises(RegistryError):
                    validation.validate_scaffold(path)

    def test_reader_and_cli_are_bounded_and_hermetic(self) -> None:
        digest = hashlib.sha256(b"abc").hexdigest()
        with tempfile.TemporaryDirectory() as temporary:
            target, link = Path(temporary) / "payload", Path(temporary) / "link"
            target.write_bytes(b"abd")
            for size in (3, -1, True, MAX_MEMBER_BYTES + 1):
                with self.assertRaises(RegistryError):
                    payload_bytes(target, {"bytes": size, "sha256": digest})
            os.symlink(target, link)
            with self.assertRaises(RegistryError):
                payload_bytes(link, {"bytes": 3, "sha256": digest})
        result = subprocess.run([sys.executable, str(HERE / "registry_inventory.py")], text=True,
                                capture_output=True, check=False)
        self.assertEqual(result.returncode, 3)
        self.assertIn("exact external vk.xml payload is required", result.stdout)


if __name__ == "__main__":
    unittest.main()
