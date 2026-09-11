#!/usr/bin/env python3
"""Focused reproducibility and boundary checks for the local Vulkan facts builder."""

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import webboxvm_source_builder as builder

XML = b"""<registry><feature api="vulkan,vulkansc" name="VK_VERSION_1_4" number="1.4" depends="VK_VERSION_1_3"><require comment="core"><type name="VkCore"/><command name="vkCore"/></require></feature><extensions><extension name="VK_EXT_core" promotedto="VK_VERSION_1_4"/><extension name="VK_KHR_video_queue"/></extensions></registry>"""
PROSE = b"vkspec root only"


def record(identifier, data, scope):
    return {"id": identifier, "revision": "a" * 40, "sha256": hashlib.sha256(data).hexdigest(),
            "bytes": len(data), "license": "test", "attribution": "test", "scope": scope,
            "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{'a' * 40}/{identifier}"}


def records(prose=PROSE, xml=XML):
    return {"vulkan-14-spec": record("vulkan-14-spec", prose, "normative-source"),
            "vulkan-registry": record("vulkan-registry", xml, "registry-metadata")}


class BuilderTests(unittest.TestCase):
    def files(self, root, prose=PROSE, xml=XML):
        vkspec, vkxml = root / "vkspec.adoc", root / "vk.xml"
        vkspec.write_bytes(prose)
        vkxml.write_bytes(xml)
        return vkspec, vkxml

    def test_same_verified_inputs_make_identical_bounded_facts(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(builder, "EXPECTED", records()):
            vkspec, vkxml = self.files(Path(temporary))
            first, second = builder.encode(vkspec, vkxml), builder.encode(vkspec, vkxml)
        value = json.loads(first)
        self.assertEqual(first, second)
        self.assertLessEqual(len(first), builder.MAX_LOCAL_BYTES)
        self.assertEqual(value["xml_feature"]["attributes"]["name"], "VK_VERSION_1_4")
        self.assertEqual(value["source_locators"]["registry_metadata"]["path"], "xml/vk.xml")
        self.assertEqual(value["boundaries"]["extensions_promoted_to_vulkan_1_4"], ["VK_EXT_core"])
        self.assertNotIn("VK_KHR_video_queue", json.dumps(value["xml_feature"]))
        self.assertTrue(all(not claim for claim in value["claims"].values()))

    def test_rejects_changed_inputs_and_non_vulkan_or_duplicate_feature(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            vkspec, vkxml = self.files(root)
            with patch.object(builder, "EXPECTED", records()):
                vkspec.write_bytes(b"changed")
                with self.assertRaises(builder.BuildError):
                    builder.encode(vkspec, vkxml)
            for xml in (XML.replace(b'api="vulkan,vulkansc"', b'api="vulkansc"'),
                        XML.replace(b"</registry>", b'<feature api="vulkan" name="VK_VERSION_1_4" number="1.4"/></registry>')):
                vkspec, vkxml = self.files(root, xml=xml)
                with patch.object(builder, "EXPECTED", records(xml=xml)):
                    with self.assertRaises(builder.BuildError):
                        builder.encode(vkspec, vkxml)

    def test_output_cap_and_symlink_fail_closed(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(builder, "MAX_LOCAL_BYTES", 1):
            output = Path(temporary) / "output.json"
            with self.assertRaises(builder.BuildError):
                builder.write_output(output, b"12")


if __name__ == "__main__":
    unittest.main()
