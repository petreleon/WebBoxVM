#!/usr/bin/env python3
"""Independent reference and semantic fences for F03.3.2.2.3.6.5."""

import importlib.util
import hashlib
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
CACHE = Path(os.environ.get("WEBBOXVM_GRAPHICS_CACHE_ROOT", "/private/tmp/webboxvm-f0341.cqT6ZX"))
PROBE = HERE / "gles_transform_feedback_capture_control_inventory.py"
spec = importlib.util.spec_from_file_location("f03322365_capture_test", PROBE)
MAP = importlib.util.module_from_spec(spec)
spec.loader.exec_module(MAP)

# Independently transcribed from visually inspected physical p.357 (printed p.339).
REFERENCE = (
    "void BeginTransformFeedback( enum primitiveMode );",
    "void EndTransformFeedback( void );",
    "void PauseTransformFeedback( void );",
    "void ResumeTransformFeedback( void );",
)
NAMES = ("BeginTransformFeedback", "EndTransformFeedback", "PauseTransformFeedback", "ResumeTransformFeedback")


class CaptureControlTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.value = MAP.validate(CACHE)
        cls.raw = MAP.CACHE.pdf_bytes(CACHE, MAP.CACHE.SOURCE)
        cls.text = MAP.CATALOG.page_text(cls.raw, 357)

    def facts(self, text=None, normalizer=None):
        with patch.object(MAP.CATALOG, "page_text", return_value=self.text if text is None else text):
            return MAP.CATALOG.facts(self.raw, 601, 26, normalizer or MAP.GRAMMAR.normalize)

    def reject_texts(self, candidates):
        for text in candidates:
            with self.subTest(text=text), self.assertRaises(MAP.CATALOG.CatalogError):
                self.facts(text)

    def test_exact_reference_inventory_and_formal_void_signatures(self):
        rows = MAP.records(self.value)
        self.assertEqual([row["unprefixed_name"] for row in rows], list(NAMES))
        self.assertEqual([row["c_name"] for row in rows], ["gl" + name for name in NAMES])
        self.assertEqual([row["declaration"] for row in rows], list(REFERENCE))
        self.assertEqual([(row["physical_page"], row["section"], row["source_order"]) for row in rows],
                         [(357, "12.2.2", index) for index in range(1, 5)])
        self.assertEqual(self.value["raw_entry_count"], 4)
        self.assertTrue(self.value["raw_only"])
        self.assertFalse(self.value["promotion_allowed"])
        self.assertEqual(self.value["domain_families"], [{"id": "transform-feedback", "source_order": 26}])

    def test_missing_duplicate_and_reordered_source_declarations_fail(self):
        swapped = self.text.replace(REFERENCE[0], "TEMP_DECLARATION").replace(REFERENCE[1], REFERENCE[0])
        swapped = swapped.replace("TEMP_DECLARATION", REFERENCE[1])
        self.reject_texts((self.text.replace(REFERENCE[2], ""),
                           self.text.replace(REFERENCE[2], REFERENCE[2] + " " + REFERENCE[2]), swapped))

    def test_changed_return_parameter_type_and_void_parameter_fail(self):
        self.reject_texts((self.text.replace(REFERENCE[0], "void BeginTransformFeedback( uint primitiveMode );"),
                           self.text.replace(REFERENCE[1], "boolean EndTransformFeedback( void );"),
                           self.text.replace(REFERENCE[2], "void PauseTransformFeedback();")))

    def test_heading_page_end_and_moved_declarations_fail(self):
        heading, footer = MAP.CATALOG.SECTION_START[1], MAP.CATALOG.SECTION_BOUNDARY[1]
        before = self.text.replace(REFERENCE[0], "").replace(heading, REFERENCE[0] + " " + heading)
        after = self.text.replace(REFERENCE[3], "").replace(footer, footer + " " + REFERENCE[3])
        self.reject_texts((self.text.replace(heading, "12.2.1 Transform Feedback Objects"),
                           self.text.replace(footer, "missing page boundary"), before, after))
        with self.assertRaises(MAP.CATALOG.CatalogError):
            MAP.CATALOG.source_window(358, self.text)

    def test_object_vertex_array_program_varying_draw_and_extension_forms_fail(self):
        additions = ("void BindTransformFeedback( enum target, uint id );", "void BindVertexArray( uint array );",
                     "void TransformFeedbackVaryings( uint program, sizei count, const char *varyings, enum bufferMode );",
                     "void DrawArrays( enum mode, int first, sizei count );", "void PauseTransformFeedbackEXT( void );")
        footer = MAP.CATALOG.SECTION_BOUNDARY[1]
        self.reject_texts(self.text.replace(footer, declaration + " " + footer) for declaration in additions)

    def test_catalog_locator_page_scope_and_order_mutations_fail(self):
        variants = (MAP.CATALOG.DECLARATIONS[:-1], tuple(reversed(MAP.CATALOG.DECLARATIONS)))
        for declarations in variants:
            with self.subTest(declarations=declarations), patch.object(MAP.CATALOG, "DECLARATIONS", declarations):
                with self.assertRaises(MAP.CATALOG.CatalogError):
                    self.facts()
        for key, value in (("PRIMARY_PAGE", 358), ("PRIMARY_SECTION", "12.2.1"), ("SOURCE_PAGES", (357, 358))):
            with self.subTest(key=key), patch.object(MAP.CATALOG, key, value):
                with self.assertRaises(MAP.CATALOG.CatalogError):
                    self.facts()

    def test_rehashed_cross_family_catalog_and_wrong_c_prefix_fail(self):
        changed = ((357, "12.2.2", "BindTransformFeedback", "void BindTransformFeedback( enum target, uint id );"),)
        changed += MAP.CATALOG.DECLARATIONS[1:]
        with patch.object(MAP.CATALOG, "DECLARATIONS", changed), \
                patch.object(MAP.CATALOG, "SEALED_DECLARATIONS_SHA256", MAP.CATALOG.sha256(changed)):
            with self.assertRaises(MAP.CATALOG.CatalogError):
                self.facts()
        with self.assertRaises(MAP.CATALOG.CatalogError):
            self.facts(normalizer=lambda forms: [forms[0][1]])

    def test_exact_family_anchor_and_route_fail_closed(self):
        self.assertEqual(MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document), 26)
        for key, value in (("ROUTE", "buffer-commands"), ("FAMILY", (*MAP.CATALOG.FAMILY[:3], 25, *MAP.CATALOG.FAMILY[4:]))):
            with self.subTest(key=key), patch.object(MAP.CATALOG, key, value):
                with self.assertRaises(MAP.CATALOG.CatalogError):
                    MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, MAP.ARTIFACT.document)

    def test_rehashed_capture_state_and_support_promotions_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            artifact = Path(temporary) / "promoted.json"
            for key, claim in (("capture_state", "active"), ("profile_support", True)):
                value = json.loads(json.dumps(self.value))
                value[key] = claim
                body = {field: item for field, item in value.items() if field != "inventory_sha256"}
                value["inventory_sha256"] = hashlib.sha256(MAP.ENGINE.canonical(body)).hexdigest()
                artifact.write_text(json.dumps(value))
                with self.subTest(key=key), patch.object(MAP.ENGINE, "rendered", return_value=self.value):
                    with self.assertRaises(MAP.InventoryError):
                        MAP.validate(CACHE, artifact)


if __name__ == "__main__":
    unittest.main()
