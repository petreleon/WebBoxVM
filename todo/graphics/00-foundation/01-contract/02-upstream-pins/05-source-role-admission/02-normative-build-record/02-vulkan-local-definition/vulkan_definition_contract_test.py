#!/usr/bin/env python3
"""Hostile reproducibility checks for the bounded local Vulkan build record."""

import copy
import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import vulkan_definition_contract as contract
import webboxvm_source_builder as builder

XML = b'<registry><feature api="vulkan" name="VK_VERSION_1_4" number="1.4"><require><type name="VkCore"/></require></feature><extensions/></registry>'
PROSE = b"identity-only Vulkan prose root"
REVISION = "a" * 40


def source(identifier, data, path, scope):
    return {"kind": "upstream-source", "id": identifier, "sha256": hashlib.sha256(data).hexdigest(),
            "bytes": len(data), "license": "test terms", "attribution": "Khronos test", "scope": scope,
            "authority": "Khronos", "producer": "Khronos", "claims": dict(contract.NO_CLAIMS),
            "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{REVISION}/{path}",
            "revision": REVISION, "artifact": f"webboxvm-graphics/test/{identifier}.source"}


def roots():
    return {"vulkan-14-spec": source("vulkan-14-spec", PROSE, "vkspec.adoc", "normative-source"),
            "vulkan-registry": source("vulkan-registry", XML, "xml/vk.xml", "registry-metadata")}


class VulkanDefinitionContractTests(unittest.TestCase):
    def fixture(self, root):
        vkspec, vkxml = root / "vkspec.adoc", root / "vk.xml"
        vkspec.write_bytes(PROSE)
        vkxml.write_bytes(XML)
        return vkspec, vkxml

    def record(self, vkspec, vkxml):
        pinned = roots()
        with patch.object(contract, "EXPECTED", pinned), patch.object(builder, "EXPECTED", pinned):
            data = builder.encode(vkspec, vkxml)
            with patch.object(contract, "OUTPUT_BYTES", len(data)):
                with patch.object(contract, "OUTPUT_SHA256", hashlib.sha256(data).hexdigest()):
                    yield contract.build_record()

    def test_rebuild_is_byte_exact_and_rejects_changed_input_or_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            vkspec, vkxml = self.fixture(root)
            for record in self.record(vkspec, vkxml):
                with patch.object(contract, "EXPECTED", roots()), patch.object(builder, "EXPECTED", roots()):
                    output = contract.rebuild(record, vkspec, vkxml, root / "definition.json")
                    contract.verify_output(record, output)
                    output.write_bytes(b"changed")
                    with self.assertRaises(contract.DefinitionError):
                        contract.verify_output(record, output)
                    vkspec.write_bytes(b"changed")
                    with self.assertRaises(builder.BuildError):
                        contract.rebuild(record, vkspec, vkxml, root / "second.json")

    def test_rejects_builder_runtime_role_command_and_output_changes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            vkspec, vkxml = self.fixture(root)
            for record in self.record(vkspec, vkxml):
                changed_builder = root / "builder.py"
                changed_builder.write_bytes(b"changed")
                with self.assertRaises(contract.DefinitionError):
                    contract.validate_build(record, builder_path=changed_builder)
                cases = []
                role = copy.deepcopy(record); role["catalog"]["records"][-1]["authority"] = "Khronos"; cases.append(role)
                claim = copy.deepcopy(record); claim["catalog"]["records"][-1]["claims"]["khronos_selector"] = True; cases.append(claim)
                command = copy.deepcopy(record); command["catalog"]["records"][-1]["command"].pop(); cases.append(command)
                output = copy.deepcopy(record); output["catalog"]["records"][-1]["sha256"] = "0" * 64; cases.append(output)
                runtime = copy.deepcopy(record); runtime["toolchain"]["runtime"]["version"] = "0"; cases.append(runtime)
                for value in cases:
                    with self.subTest(value=value), self.assertRaises(contract.DefinitionError):
                        contract.validate_build(value)

    def test_record_uses_only_the_two_pinned_inputs_and_a_local_no_claim_transform(self):
        record = contract.build_record()
        transform = record["catalog"]["records"][-1]
        self.assertEqual([item["id"] for item in transform["inputs"]], list(contract.INPUT_IDS))
        self.assertEqual(transform["authority"], "WebBoxVM")
        self.assertTrue(all(not item for item in transform["claims"].values()))
        self.assertEqual(record["toolchain"]["execution"]["network"], "forbidden")
        self.assertLessEqual(transform["bytes"], 8 * 1024 * 1024)


if __name__ == "__main__":
    unittest.main()
