#!/usr/bin/env python3
"""Hostile schema tests for the F05 source-sealed registration catalog."""

from __future__ import annotations

import copy
import hashlib
import json
import unittest

import profile_registration as registration
import profile_registration_sources as sources


def rehash(value: dict[str, object]) -> dict[str, object]:
    body = {key: item for key, item in value.items() if key != "catalog_sha256"}
    value["catalog_sha256"] = hashlib.sha256(registration.canonical(body)).hexdigest()
    return value


class RegistrationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.base = registration.document(registration.CATALOG)

    def value(self) -> dict[str, object]:
        return rehash(copy.deepcopy(self.base))

    def rejected(self, change) -> None:
        value = self.value()
        change(value)
        with self.assertRaises(registration.RegistrationError):
            registration.validate_catalog(rehash(value))

    def implementation(self) -> dict[str, object]:
        value, contract = self.value(), sources.admitted_contract()
        bindings = {(item["profile"], item["role"]): item for item in contract["bindings"]}
        check = copy.deepcopy(value["checks"][1])
        check.update(id="vulkan-implementation-fixture", kind="implementation", profile="vulkan-1.4-core",
                     profile_effect="blocked-observation", lane="implementation", requires_selector_cache=False,
                     source={"roles": [bindings[("vulkan-1.4-core", "normative-root")],
                                       bindings[("vulkan-1.4-core", "full-suite-root")]]})
        value["checks"], value["states"]["profile_implementation_count"] = [check], 1
        return rehash(value)

    def test_current_catalog_registers_only_no_claim_lanes(self) -> None:
        value = registration.load_catalog()
        self.assertEqual(value["states"]["profile_implementation_count"], 0)
        self.assertTrue(all(item is False for item in value["claims"].values()))
        self.assertEqual(value["cts_executions"], 0)
        self.assertEqual([item["id"] for item in value["checks"]], [
            "vulkan-registry-inventory-v2", "make-test", "web-pkg-serial", "web-pkg-threaded", "virgl-guest-transport",
        ])
        auxiliary = value["checks"][0]
        self.assertEqual((auxiliary["kind"], auxiliary["profile_effect"], auxiliary["source"]["scope"]),
                         ("auxiliary-inventory", "none", "registry-metadata"))

    def test_rejects_stale_headers_or_hash(self) -> None:
        self.rejected(lambda value: value.update(source_contract_sha256="0" * 64))
        self.rejected(lambda value: value.update(inventory_lock_sha256="0" * 64))
        value = self.value()
        value["catalog_sha256"] = "0" * 64
        with self.assertRaises(registration.RegistrationError):
            registration.validate_catalog(value)

    def test_rejects_auxiliary_promotion_and_qualifying_source_evidence(self) -> None:
        self.rejected(lambda value: value["checks"][0].update(profile_effect="blocked-observation"))
        self.rejected(lambda value: value["checks"][0]["source"].update(scope="normative-source"))
        contract = sources.admitted_contract()
        contract["auxiliary"][0]["cts_executions"] = 1
        with self.assertRaises(registration.RegistrationError):
            registration.validate_catalog(self.value(), contract)
        contract = sources.admitted_contract()
        contract["auxiliary"][0]["claims"]["profile_support"] = True
        with self.assertRaises(registration.RegistrationError):
            registration.validate_catalog(self.value(), contract)

    def test_rejects_empty_command_count_artifact_or_evidence(self) -> None:
        changes = (
            lambda check: check.update(command=[]), lambda check: check.update(expected_count=0),
            lambda check: check.update(artifacts=[]), lambda check: check.update(evidence="missing-evidence.md"),
        )
        for change in changes:
            with self.subTest(change=change):
                self.rejected(lambda value: change(value["checks"][1]))

    def test_implementation_needs_exact_same_profile_role_pair(self) -> None:
        value = self.implementation()
        registration.validate_catalog(value)
        contract = sources.admitted_contract()
        wrong = next(item for item in contract["bindings"] if item["profile"] == "opengl-4.6-core" and item["role"] == "full-suite-root")
        value["checks"][0]["source"]["roles"][1] = wrong
        with self.assertRaises(registration.RegistrationError):
            registration.validate_catalog(rehash(value))

    def test_profile_null_baseline_cannot_be_promoted(self) -> None:
        self.rejected(lambda value: value["checks"][1].update(profile="vulkan-1.4-core"))
        self.rejected(lambda value: value["checks"][1].update(profile_effect="blocked-observation"))


if __name__ == "__main__":
    unittest.main()
