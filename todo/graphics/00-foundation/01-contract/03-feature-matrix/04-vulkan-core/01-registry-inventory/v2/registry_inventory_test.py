#!/usr/bin/env python3
"""Hermetic hostile tests for the v2 Vulkan registry fact inventory."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import registry_inventory_contract as contract
import registry_inventory_source as source
import registry_inventory_validation as validation

HERE = Path(__file__).resolve().parent


def fixture() -> bytes:
    members = {("BASE_", 0): '<require><command name="vkBase"/><type name="VkBase"/></require><deprecate><type name="VkOld"/></deprecate>',
               ("COMPUTE_", 0): '<require><enum name="VK_COMPUTE"/></require>',
               ("GRAPHICS_", 0): '<require><comment>ignored metadata</comment><feature name="graphicsFeature"/></require>',
               ("", 4): ('<require comment="features"><feature name="pipelineProtectedAccess" struct="One"/></require>'
                         '<require depends="protected"><feature name="pipelineProtectedAccess" struct="Two"/></require>')}
    def feature(prefix: str, minor: int) -> str:
        internal = ' apitype="internal"' if prefix else ""
        return f'<feature api="vulkan"{internal} name="VK_{prefix}VERSION_1_{minor}" number="1.{minor}">{members.get((prefix, minor), "")}</feature>'
    features = ''.join(feature(prefix, minor) for minor in range(5) for prefix in ("BASE_", "COMPUTE_", "GRAPHICS_", ""))
    return f'<registry>{features}<extensions><extension><require><command name="vkExtension"/></require></extension></extensions></registry>'.encode()


def value(rows: list[dict[str, object]], identity: dict[str, object]) -> dict[str, object]:
    return {"schema": 2, "contract": "vulkan-registry-technical-inventory-v2", "status": "blocked",
            "source_contract_sha256": identity["source_contract_sha256"], "inventory_lock_sha256": identity["inventory_lock_sha256"],
            "registry": identity, "scope": "raw-direct-cumulative-vulkan-1.0-through-1.4-structural-blocks",
            "boundaries": list(contract.BOUNDARIES), "rows": rows, "rows_sha256": contract.rows_sha256(rows),
            "effects": {effect: False for effect in contract.EFFECTS}}


def changed(document: dict[str, object], trail: list[str], replacement: object) -> dict[str, object]:
    result, cursor = copy.deepcopy(document), None
    cursor = result
    for key in trail[:-1]:
        cursor = cursor[key]
    cursor[trail[-1]] = replacement
    result["scaffold_sha256"] = hashlib.sha256(validation.canonical(result)).hexdigest()
    return result


class RegistryInventoryTests(unittest.TestCase):
    def expected(self, rows: list[dict[str, object]]) -> tuple[int, str]:
        previous = validation.EXPECTED_COUNT, validation.EXPECTED_ROWS_SHA256
        validation.EXPECTED_COUNT, validation.EXPECTED_ROWS_SHA256 = len(rows), contract.rows_sha256(rows)
        return previous

    def test_extracts_structural_blocks_and_preserves_lifecycle_and_duplicates(self) -> None:
        rows = contract.rows_from_raw(fixture())
        markers = [row["name"] for row in rows if row["requirement_kind"] == "version-marker"]
        self.assertEqual(markers, [name for name, _ in contract.FEATURES])
        self.assertEqual([row["source_order"] for row in rows], list(range(1, len(rows) + 1)))
        self.assertTrue({"command", "enum", "feature", "type"} <= {row["requirement_kind"] for row in rows})
        self.assertIn("deprecate", [row["container_kind"] for row in rows])
        self.assertEqual(len([row for row in rows if row["name"] == "pipelineProtectedAccess"]), 2)
        self.assertNotIn("vkExtension", [row["name"] for row in rows])

    def test_rejects_malformed_and_reordered_xml(self) -> None:
        bad = (b'<!DOCTYPE registry><registry/>', '<!DOCTYPE registry [<!ENTITY x "x">]><registry/>'.encode("utf-16"), b'<wrong/>',
               fixture().replace(b'VK_VERSION_1_3', b'VK_VERSION_1_X'), fixture().replace(b'VK_BASE_VERSION_1_0', b'VK_COMPUTE_VERSION_1_0', 1),
               fixture().replace(b'<deprecate>', b'<remove>', 1), fixture().replace(b'<command name="vkBase"/>', b'<bogus name="x"/>', 1),
               b" " * (contract.MAX_SERIALIZED + 1))
        for raw in bad:
            with self.assertRaises(source.RegistryError):
                contract.rows_from_raw(raw)

    def test_rejects_partial_reordered_duplicate_promoted_or_assigned_rows(self) -> None:
        identity, rows = source.registry_identity(), contract.rows_from_raw(fixture())
        previous = self.expected(rows)
        try:
            validation.validate_inventory(value(copy.deepcopy(rows), identity), identity)
            with self.assertRaisesRegex(source.RegistryError, "partial"):
                validation.validate_inventory(value(copy.deepcopy(rows[:-1]), identity), identity)
            for altered, change in ((list(reversed(copy.deepcopy(rows))), lambda items: None),
                                    (copy.deepcopy(rows), lambda items: items[0].update(status="supported")),
                                    (copy.deepcopy(rows), lambda items: items[0].update(implementation_owner="F99.1"))):
                change(altered)
                validation.EXPECTED_COUNT, validation.EXPECTED_ROWS_SHA256 = len(altered), contract.rows_sha256(altered)
                with self.assertRaises(source.RegistryError):
                    validation.validate_inventory(value(altered, identity), identity)
            duplicate = copy.deepcopy(rows) + [copy.deepcopy(rows[0])]
            duplicate[-1]["source_order"] = len(duplicate)
            validation.EXPECTED_COUNT, validation.EXPECTED_ROWS_SHA256 = len(duplicate), contract.rows_sha256(duplicate)
            with self.assertRaisesRegex(source.RegistryError, "duplicate source locators"):
                validation.validate_inventory(value(duplicate, identity), identity)
        finally:
            validation.EXPECTED_COUNT, validation.EXPECTED_ROWS_SHA256 = previous

    def test_scaffold_rejects_headers_identity_promotions_and_type_aliases(self) -> None:
        original = json.loads(validation.SCAFFOLD.read_text(encoding="utf-8"))
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "scaffold.json"
            path.write_text('{"schema":2,"schema":2}', encoding="utf-8")
            with self.assertRaisesRegex(source.RegistryError, "duplicate"):
                validation.validate_scaffold(path)
            cases = ((["source_contract_sha256"], "stale"), (["registry", "scope"], "normative-source"),
                     (["registry", "discharges_required_role"], True), (["effects", "supported"], 0),
                     (["schema"], True), (["registry", "bytes"], float(original["registry"]["bytes"])))
            for trail, replacement in cases:
                path.write_text(json.dumps(changed(original, trail, replacement)), encoding="utf-8")
                with self.assertRaises(source.RegistryError):
                    validation.validate_scaffold(path)

    def test_reader_and_cli_require_the_sealed_cache_path(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            target, link = Path(temporary) / "payload", Path(temporary) / "link"
            target.write_bytes(b"abc")
            for size in (0, -1, True, source.MAX_MEMBER_BYTES + 1):
                with self.assertRaises(source.RegistryError):
                    source.bounded_bytes(target, size, "fixture")
            link.symlink_to(target)
            with self.assertRaises(source.RegistryError):
                source.bounded_bytes(link, 3, "fixture")
        no_cache = subprocess.run([sys.executable, str(HERE / "registry_inventory.py")], text=True, capture_output=True, check=False)
        self.assertEqual(no_cache.returncode, 3)
        self.assertIn("exact external selector cache", no_cache.stdout)
        arbitrary = subprocess.run([sys.executable, str(HERE / "registry_inventory.py"), "--payload", "x"], text=True, capture_output=True, check=False)
        self.assertEqual(arbitrary.returncode, 2)


if __name__ == "__main__":
    unittest.main()
