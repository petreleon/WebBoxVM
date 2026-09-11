#!/usr/bin/env python3
"""Hostile artifact and no-claim checks for F02.5.2.3 admission."""

import copy
import hashlib
import tempfile
import unittest
from contextlib import contextmanager
from pathlib import Path
from unittest.mock import patch

import admission_receipt as admission
import vulkan_definition_contract as definition
import webboxvm_source_builder as builder

REVISION = "a" * 40
PAYLOADS = {
    "opengl-46-core-spec": b"gl",
    "gles-32-spec": b"gles",
    "vulkan-14-spec": b"vkspec root only",
    "vulkan-registry": b'<registry><feature api="vulkan" name="VK_VERSION_1_4" number="1.4"><require><type name="VkCore"/></require></feature><extensions/></registry>',
}


def root(identifier, repository, path, scope):
    data = PAYLOADS[identifier]
    digest = hashlib.sha256(data).hexdigest()
    return {"kind": "upstream-source", "id": identifier, "sha256": digest, "bytes": len(data),
            "license": "test terms", "attribution": "Khronos test", "scope": scope,
            "authority": "Khronos", "producer": "Khronos", "claims": dict(admission.NO_CLAIMS),
            "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/{repository}/{REVISION}/{path}",
            "revision": REVISION, "artifact": f"webboxvm-graphics/test/{identifier}/{digest}.source"}


def roots():
    return (
        root("opengl-46-core-spec", "OpenGL-Registry", "specs/gl/glspec46.core.pdf", "normative-source"),
        root("gles-32-spec", "OpenGL-Registry", "specs/es/3.2/es_spec_3.2.pdf", "normative-source"),
        root("vulkan-14-spec", "Vulkan-Docs", "vkspec.adoc", "normative-source"),
        root("vulkan-registry", "Vulkan-Docs", "xml/vk.xml", "registry-metadata"),
    )


class AdmissionReceiptTests(unittest.TestCase):
    @contextmanager
    def staged(self):
        with tempfile.TemporaryDirectory() as temporary:
            root_path = Path(temporary)
            all_roots = roots()
            inputs = {item["id"]: item for item in all_roots if item["id"].startswith("vulkan-")}
            for item in all_roots:
                target = root_path / item["artifact"]
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(PAYLOADS[item["id"]])
            with patch.object(admission, "ROOTS", all_roots), patch.object(admission, "ROOT_IDS", tuple(item["id"] for item in all_roots)), \
                    patch.object(definition, "EXPECTED", inputs), patch.object(builder, "EXPECTED", inputs):
                data = builder.encode(root_path / inputs["vulkan-14-spec"]["artifact"],
                                      root_path / inputs["vulkan-registry"]["artifact"])
                with patch.object(definition, "OUTPUT_BYTES", len(data)), \
                        patch.object(definition, "OUTPUT_SHA256", hashlib.sha256(data).hexdigest()):
                    value = admission.admission_catalog()
                    transform = value["records"][-1]
                    staged_builder = root_path / transform["builder"]["artifact"]
                    staged_builder.parent.mkdir(parents=True, exist_ok=True)
                    staged_builder.write_bytes(definition.BUILDER_PATH.read_bytes())
                    output = root_path / transform["artifact"]
                    output.parent.mkdir(parents=True, exist_ok=True)
                    output.write_bytes(data)
                    yield value, root_path

    def test_staged_catalog_has_exact_receipt_and_false_claims(self):
        with self.staged() as (value, root_path):
            receipt = admission.verify_admission(value, root_path)
        self.assertEqual(receipt["source_ids"], list(admission.ROOT_IDS))
        self.assertEqual(receipt["cts_executions"], 0)
        self.assertTrue(all(not claim for claim in receipt["claims"].values()))

    def test_rejects_tampered_source_builder_or_output(self):
        for target_kind in ("source", "builder", "output"):
            with self.subTest(target_kind=target_kind), self.staged() as (value, root_path):
                transform = value["records"][-1]
                target = {"source": root_path / value["records"][0]["artifact"],
                          "builder": root_path / transform["builder"]["artifact"],
                          "output": root_path / transform["artifact"]}[target_kind]
                target.write_bytes(b"tampered")
                with self.assertRaises(admission.AdmissionError):
                    admission.verify_admission(value, root_path)

    def test_rejects_substituted_or_partial_catalog_positive_claims_and_bad_receipt(self):
        with self.staged() as (value, _root_path):
            cases = []
            substituted = copy.deepcopy(value)
            substituted["records"][0]["sha256"] = "f" * 64
            cases.append(substituted)
            partial = copy.deepcopy(value)
            partial["records"].pop(0)
            cases.append(partial)
            for name in admission.NO_CLAIMS:
                claimed = copy.deepcopy(value)
                claimed["records"][-1]["claims"][name] = True
                cases.append(claimed)
            for bad_catalog in cases:
                with self.subTest(catalog=bad_catalog), self.assertRaises(admission.AdmissionError):
                    admission.validate_admission(bad_catalog)
            receipt = admission.admission_receipt(value)
            receipt["cts_executions"] = 1
            with self.assertRaises(admission.AdmissionError):
                admission.validate_receipt(value, receipt)

    def test_nonfresh_live_root_fails_before_network_fetch(self):
        with tempfile.TemporaryDirectory() as temporary:
            Path(temporary, "old").write_bytes(b"old")
            with self.assertRaises(admission.AdmissionError):
                admission.admit(Path(temporary), 1)


if __name__ == "__main__":
    unittest.main()
