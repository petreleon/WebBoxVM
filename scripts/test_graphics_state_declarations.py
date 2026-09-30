#!/usr/bin/env python3
"""Independent command sets and hostile section checks for three complete GLES routes."""

import copy
import importlib.util
import os
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
STATE = ROOT / ("todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/"
                "02-command-object-state-raw-inventory/04-state-execution-command-slices")
CACHE = Path(os.environ.get("WEBBOXVM_GRAPHICS_CACHE_ROOT", "/private/tmp/webboxvm-f0341.cqT6ZX"))


def load(folder, stem):
    spec = importlib.util.spec_from_file_location(stem + "_test", STATE / folder / (stem + "_inventory.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SourceChecks:
    @classmethod
    def setUpClass(cls):
        cls.api = load(cls.FOLDER, cls.STEM)
        cls.value = cls.api.validate(CACHE)
        cls.rows = cls.api.records(cls.value)
        cls.raw = cls.api.CACHE.pdf_bytes(CACHE, cls.api.CACHE.SOURCE)

    def test_independent_complete_command_set(self):
        self.assertEqual([row["unprefixed_name"] for row in self.rows], self.NAMES.split())
        self.assertEqual(self.value["raw_entry_count"], len(self.NAMES.split()))
        self.assertTrue(self.value["source_coverage"]["complete"])
        self.assertTrue(self.value["raw_only"])
        self.assertFalse(self.value["promotion_allowed"])

    def test_exact_source_order_locators_and_domain_families(self):
        self.assertEqual([row["source_order"] for row in self.rows], list(range(1, len(self.rows) + 1)))
        self.assertEqual([row["physical_page"] for row in self.rows], sorted(row["physical_page"] for row in self.rows))
        binding = {row["id"]: row["source_order"] for row in self.value["domain_families"]}
        for row in self.rows:
            self.assertEqual(row["c_name"], "gl" + row["unprefixed_name"])
            self.assertEqual(row["source_family_order"], binding[row["family_id"]])
            self.assertEqual(row["source_locator"], f"gles32-pdf-v1:page={row['physical_page']};section={row['section']}")
        self.api.ARTIFACT.forbidden(self.value)

    def test_missing_duplicate_and_extension_source_forms_fail(self):
        helper = self.api.CATALOG.HELPER
        original = helper.PDF.sealed_pdf_pages
        row = self.rows[self.MUTATION_ROW]
        for mutation in ("missing", "duplicate", "extension", "return"):
            def read(raw, first, last):
                texts = original(raw, first, last)
                page, name = row["physical_page"], row["unprefixed_name"]
                if first <= page <= last and first != 571:
                    if mutation == "duplicate":
                        texts[page] += "\n" + row["declaration"] + "\n"
                    elif mutation == "return":
                        texts[page] = texts[page].replace("void " + name + "(", "enum " + name + "(")
                    else:
                        texts[page] = texts[page].replace(name + "(", ("Missing" if mutation == "missing" else name + "EXT") + "(")
                return texts
            with self.subTest(mutation=mutation), patch.object(helper.PDF, "sealed_pdf_pages", side_effect=read):
                with self.assertRaises(self.api.CATALOG.CatalogError):
                    self.api.CATALOG.CATALOG.extracted(self.raw)

    def test_source_boundary_and_full_route_mutations_fail(self):
        catalog = self.api.CATALOG.CATALOG
        with patch.object(catalog.contract, "WINDOWS", catalog.contract.WINDOWS[:-1]):
            with self.assertRaises(self.api.CATALOG.CatalogError):
                catalog.extracted(self.raw)
        document = self.api.ARTIFACT.document(self.api.DOMAIN_API.CHUNKS["execution"])
        routed = next(row for row in document["families"] if row["route"] == self.api.CATALOG.ROUTE)
        for mutation in ("route", "source_order", "source_scope"):
            changed = copy.deepcopy(document)
            item = next(row for row in changed["families"] if row["id"] == routed["id"])
            item[mutation] = "wrong" if mutation != "source_scope" else []
            with self.subTest(mutation=mutation), self.assertRaises(self.api.CATALOG.CatalogError):
                catalog.bound_family(self.api.DOMAIN_API.CHUNKS, lambda path: changed)

    def test_index_omission_and_unadmitted_registry_are_rejected(self):
        helper = self.api.CATALOG.HELPER
        with patch.object(helper.PDF, "sealed_pdf_pages", return_value={page: "missing" for page in range(571, 602)}):
            with self.assertRaises(ValueError):
                helper.CHECK.verify_index(self.raw, self.api.CATALOG.INDEX_WITNESSES, helper.PDF)
        with self.assertRaises(self.api.InventoryError):
            self.api.ENGINE.reject_unadmitted("extension", "https://registry.khronos.org/OpenGL/api/GLES3/gl32.h")


class DrawTests(SourceChecks, unittest.TestCase):
    FOLDER, STEM, MUTATION_ROW = "02-draw-raster-commands", "gles_draw_raster_command", 0
    # Independently checked against formal declarations and the command-name PDF index.
    NAMES = ("DrawArrays DrawArraysInstanced DrawArraysIndirect DrawElements DrawElementsInstanced "
             "DrawRangeElements DrawElementsBaseVertex DrawRangeElementsBaseVertex DrawElementsInstancedBaseVertex "
             "DrawElementsIndirect PrimitiveBoundingBox GetMultisamplefv MinSampleShading LineWidth FrontFace "
             "CullFace PolygonOffset Scissor SampleCoverage SampleMaski StencilFunc StencilFuncSeparate "
             "StencilOp StencilOpSeparate DepthFunc Enablei Disablei BlendEquation BlendEquationSeparate "
             "BlendEquationi BlendEquationSeparatei BlendFunc BlendFuncSeparate BlendFunci BlendFuncSeparatei "
             "BlendBarrier BlendColor DrawBuffers ColorMask ColorMaski DepthMask StencilMask StencilMaskSeparate "
             "Clear ClearColor ClearDepthf ClearStencil ClearBufferiv ClearBufferfv ClearBufferuiv ClearBufferfi "
             "InvalidateSubFramebuffer InvalidateFramebuffer DispatchCompute DispatchComputeIndirect")

    def test_pseudo_commands_and_indirect_body_do_not_create_commands(self):
        names = {row["unprefixed_name"] for row in self.rows}
        self.assertFalse(names & {"DrawArraysOneInstance", "DrawElementsOneInstance"})
        self.assertEqual(len(self.value["source_coverage"]["excluded_non_commands"]), 2)
        self.assertEqual(self.value["source_coverage"]["source_windows"][0]["formal_declaration_count"], 0)

    def test_clear_template_and_compute_signatures_keep_source_spelling(self):
        clear = [row for row in self.rows if row["unprefixed_name"] in ("ClearBufferiv", "ClearBufferfv", "ClearBufferuiv")]
        self.assertEqual(len(clear), 3)
        self.assertEqual({(row["physical_page"], row["section"]) for row in clear}, {(420, "15.2.3.1")})
        self.assertEqual({row["declaration"] for row in clear}, {"void ClearBuffer{if ui}v( enum buffer, int drawbuffer, const T *value );"})
        self.assertEqual(self.rows[-2]["declaration"], "void DispatchCompute( uint num groups x, uint num groups y, uint num groups z );")


class PixelTests(SourceChecks, unittest.TestCase):
    FOLDER, STEM, MUTATION_ROW = "03-pixel-transfer-commands", "gles_pixel_transfer_command", 2
    NAMES = "PixelStorei ReadBuffer ReadPixels ReadnPixels BlitFramebuffer CopyImageSubData"

    def test_pack_transfer_and_image_copy_are_formal_declarations_only(self):
        self.assertEqual(self.rows[0]["declaration"], "void PixelStorei( enum pname, int param );")
        self.assertIn("sizei bufSize", self.rows[3]["declaration"])
        self.assertEqual((self.rows[-1]["physical_page"], self.rows[-1]["section"]), (433, "16.2.2"))
        self.assertIn("formats", self.value["source_coverage"]["semantic_routes"])
        self.assertNotIn("TexImage3D", self.NAMES.split())


class DebugTests(SourceChecks, unittest.TestCase):
    FOLDER, STEM, MUTATION_ROW = "04-debug-special-queries", "gles_debug_special_query_command", 0
    NAMES = ("DebugMessageCallback DebugMessageControl DebugMessageInsert PushDebugGroup PopDebugGroup "
             "ObjectLabel ObjectPtrLabel GetDebugMessageLog GetObjectLabel GetObjectPtrLabel Hint GetBooleanv "
             "GetIntegerv GetInteger64v GetFloatv GetBooleani_v GetIntegeri_v GetInteger64i_v IsEnabled IsEnabledi "
             "GetPointerv GetString GetStringi GetInternalformativ")

    def test_visually_confirmed_underscore_and_pointer_return_declarations(self):
        rows = {row["unprefixed_name"]: row for row in self.rows}
        self.assertEqual(rows["GetString"]["declaration"], "ubyte *GetString( enum name );")
        self.assertEqual(rows["GetStringi"]["declaration"], "ubyte *GetStringi( enum name, uint index );")
        self.assertIn("GetBooleani v(", rows["GetBooleani_v"]["declaration"])
        self.assertIn("size *length", rows["GetObjectPtrLabel"]["declaration"])
        self.assertEqual(len(self.value["source_coverage"]["typographic_spellings"]), 3)

    def test_reset_declaration_uses_existing_exact_generic_receipt(self):
        route = self.value["source_coverage"]["routed_declarations"]
        self.assertEqual(len(route), 1)
        self.assertEqual((route[0][2], route[0][4], route[0][6]), ("F03.3.2.2.3.1", "glGetGraphicsResetStatus", 34))
        self.assertNotIn("GetGraphicsResetStatus", self.NAMES.split())


if __name__ == "__main__":
    unittest.main()
