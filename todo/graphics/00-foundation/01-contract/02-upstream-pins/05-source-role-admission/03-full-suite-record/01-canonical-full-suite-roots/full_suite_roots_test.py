#!/usr/bin/env python3
"""Focused identity and hostile-boundary checks for canonical full-suite roots."""

import copy
import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import full_suite_roots as roots


class FullSuiteRootsTests(unittest.TestCase):
    def test_exact_ordered_roots_have_selector_only_claims_and_release_proofs(self):
        value = roots.catalog()
        self.assertEqual(roots.validate_full_suite_catalog(value), roots.ROOT_IDS)
        self.assertEqual(value["records"][0]["suite_id"], value["records"][0]["id"])
        self.assertEqual(roots.proof_for(value["records"][0])["tag"], "opengl-cts-4.6.8.1")
        self.assertEqual(roots.proof_for(value["records"][2])["tag"], "vulkan-cts-1.4.6.2")
        for record in value["records"]:
            self.assertTrue(record["claims"]["khronos_selector"])
            self.assertTrue(all(not claim for name, claim in record["claims"].items()
                                if name != "khronos_selector"))

    def test_rejects_missing_reordered_or_mutated_root_roles(self):
        cases = []
        missing = roots.catalog(); missing["records"].pop(); cases.append(missing)
        reordered = roots.catalog(); reordered["records"].reverse(); cases.append(reordered)
        suite = roots.catalog(); suite["records"][0]["suite_id"] = "gles-cts-main"; cases.append(suite)
        mutable = roots.catalog(); mutable["records"][0]["immutable_url"] = mutable["records"][0]["immutable_url"].replace(roots.GL_REVISION, "main"); cases.append(mutable)
        foreign = roots.catalog(); foreign["records"][0]["immutable_url"] = foreign["records"][0]["immutable_url"].replace("KhronosGroup", "Elsewhere"); cases.append(foreign)
        filtered = roots.catalog(); filtered["records"][2]["unfiltered"] = False; cases.append(filtered)
        fraction = roots.catalog(); fraction["records"][2]["selector_path"] = "external/vulkancts/mustpass/main/vk-fraction.txt"; cases.append(fraction)
        claim = roots.catalog(); claim["records"][2]["claims"]["conformance"] = True; cases.append(claim)
        for value in cases:
            with self.subTest(value=value), self.assertRaises(roots.FullSuiteError):
                roots.validate_full_suite_catalog(value)

    def test_source_inputs_and_license_proofs_preserve_exact_cache_identity(self):
        for record in roots.catalog()["records"]:
            with self.subTest(identifier=record["id"]):
                source = roots.source_input(record)
                self.assertEqual((source.identifier, str(source.local_cache)), (record["id"], record["artifact"]))
                self.assertEqual((source.url, source.revision), (record["immutable_url"], record["revision"]))
        for proof in roots.RELEASE_PROOFS:
            with self.subTest(tag=proof["tag"]):
                source = roots.license_input(proof)
                self.assertEqual(source.identifier, proof["license_id"])
                self.assertEqual((source.byte_count, source.sha256), (roots.LICENSE_BYTES, roots.LICENSE_SHA256))
                self.assertEqual(source.revision, proof["peeled_commit"])
                self.assertIn(proof["peeled_commit"], source.url)

    def test_license_payload_and_fresh_cache_fail_closed(self):
        payload = b"Apache License\nVersion 2.0\n"
        proof = copy.deepcopy(roots.RELEASE_PROOFS[0])
        with patch.object(roots, "LICENSE_BYTES", len(payload)), \
                patch.object(roots, "LICENSE_SHA256", hashlib.sha256(payload).hexdigest()):
            roots.verify_license_payload(proof, payload)
            with self.assertRaises(roots.FullSuiteError):
                roots.verify_license_payload(proof, b"tampered")
        with tempfile.TemporaryDirectory() as temporary:
            Path(temporary, "old").write_bytes(b"old")
            with self.assertRaises(roots.FullSuiteError):
                roots.fetch_and_verify(roots.catalog(), Path(temporary), 1)

    def test_local_receipt_does_not_claim_execution_or_core_only_vulkan(self):
        receipt = roots.receipt(roots.catalog())
        self.assertEqual(receipt["cts_executions"], 0)
        self.assertTrue(all(not claim for claim in receipt["claims"].values()))
        self.assertIn("broader", receipt["vulkan_selector_scope"])
        self.assertIn("core-only", receipt["vulkan_selector_scope"])


if __name__ == "__main__":
    unittest.main()
