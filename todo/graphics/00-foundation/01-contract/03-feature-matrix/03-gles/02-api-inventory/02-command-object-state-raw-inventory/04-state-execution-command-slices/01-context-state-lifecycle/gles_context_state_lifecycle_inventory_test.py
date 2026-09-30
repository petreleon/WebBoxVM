"""Visually transcribed reference and adversarial lifecycle source boundaries."""

import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
CACHE = Path(os.environ.get("WEBBOXVM_GRAPHICS_CACHE_ROOT", "/private/tmp/webboxvm-f0341.cqT6ZX"))
SPEC = importlib.util.spec_from_file_location("lifecycle_test", HERE / "gles_context_state_lifecycle_inventory.py")
MAP = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MAP)
REFERENCE = (
    ["GenBuffers"], ["BindBuffer", "GenBuffers", "BindSampler"], ["CreateProgram", "FenceSync"], ["DeleteBuffers"],
    ["DeleteBuffers", "DeleteTextures", "DeleteTransformFeedbacks", "DeleteRenderbuffers"],
    ["ClientWaitSync", "WaitSync"], ["BufferSubData"], ["EndTransformFeedback"], ["BufferData"],
    ["UnmapBuffer", "FlushMappedBufferRange"], ["Finish", "FenceSync", "WaitSync"], ["EndTransformFeedback"],
)


class LifecycleSourceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.value = MAP.validate(CACHE)
        cls.raw = MAP.CACHE.pdf_bytes(CACHE, MAP.CACHE.SOURCE)
        cls.texts = {page: MAP.CATALOG.page_text(cls.raw, page) for page, _ in MAP.CATALOG.PAGE_HASHES}

    def facts(self, texts=None, normalizer=None):
        source = self.texts if texts is None else texts
        with patch.object(MAP.CATALOG, "page_text", side_effect=lambda _, page: source[page]):
            return MAP.CATALOG.facts(self.raw, 601, self.value["domain_families"], normalizer or MAP.GRAMMAR.normalize)

    def bad_text(self, page, original, replacement):
        texts = dict(self.texts)
        self.assertIn(original, texts[page])
        texts[page] = texts[page].replace(original, replacement)
        with self.assertRaises(MAP.CATALOG.CatalogError):
            self.facts(texts)

    def sealed_catalog_mutation(self, rules):
        with patch.object(MAP.CATALOG, "RULES", rules), \
                patch.object(MAP.CATALOG, "RULES_SHA256", MAP.CATALOG.sha256(rules)):
            with self.assertRaises(MAP.CATALOG.CatalogError):
                self.facts()

    def mutated_artifact(self, name, change, hash_field):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary)
            for filename, content in MAP.ENGINE.generated(self.value):
                (destination / filename).write_text(content)
            path = destination / name
            value = json.loads(path.read_text())
            change(value)
            body = {key: item for key, item in value.items() if key != hash_field}
            value[hash_field] = hashlib.sha256(MAP.ENGINE.canonical(body)).hexdigest()
            path.write_text(json.dumps(value))
            with patch.object(MAP.ENGINE, "rendered", return_value=self.value), self.assertRaises(MAP.InventoryError):
                MAP.validate(CACHE, destination / MAP.INVENTORY.name)

    def test_exact_visual_reference_and_page_section_order(self):
        rows = MAP.records(self.value)
        self.assertEqual([row["triggering_commands"] for row in rows], list(REFERENCE))
        self.assertEqual([row["c_names"] for row in rows], [["gl" + name for name in names] for names in REFERENCE])
        self.assertEqual([row["physical_page"] for row in rows], [43, 44, 44, 44, 63, 63, 65, 65, 65, 65, 65, 66])
        self.assertEqual([row["source_order"] for row in rows], list(range(1, 13)))
        self.assertEqual([row["family_id"] for row in rows], ["object-taxonomy"] * 4 + ["context-and-lifecycle"] * 8)

    def test_qualifying_clauses_stay_in_raw_rules(self):
        rows = MAP.records(self.value)
        self.assertIn("do not initially correspond to an instance of an object", rows[1]["rule_text"])
        self.assertIn("Attachments to unbound container objects", rows[4]["rule_text"])
        self.assertIn("not affected and continue to act as references", rows[4]["rule_text"])
        self.assertIn("restored to default values", rows[4]["rule_text"])
        self.assertIn("blocked on the sync object", rows[5]["rule_text"])
        self.assertIn("same context", rows[11]["rule_text"])

    def test_wildcards_generic_rules_and_undefined_clauses_are_not_inferred(self):
        rows = MAP.records(self.value)
        names = {name for row in rows for name in row["triggering_commands"]}
        self.assertFalse(names & {"TexParameter", "TexSubImage2D", "TexImage2D", "Gen", "DeleteSync"})
        self.assertIn("TexSubImage*", rows[6]["rule_text"])
        self.assertNotIn("undefined", rows[10]["rule_text"] + rows[11]["rule_text"])
        self.assertFalse(self.value["promotion_allowed"])

    def test_all_eight_routes_and_five_pending_families_remain_open(self):
        coverage = self.value["source_coverage"]
        self.assertEqual(len(self.value["domain_families"]), 8)
        self.assertFalse(coverage["complete"])
        self.assertEqual(coverage["pending_family_count"], 5)
        self.assertEqual([row[0] for row in coverage["routes"] if row[1] == "pending"],
                         ["vertex-remaining-state", "programmable-vertex-stage", "post-vertex-state",
                          "programmable-fragment-stage", "state-tables"])

    def test_missing_duplicate_or_reordered_source_quotes_fail(self):
        quote = " ".join(MAP.CATALOG.RULES[2][4])
        self.bad_text(44, quote, "")
        self.bad_text(44, quote, quote + " " + quote)
        self.sealed_catalog_mutation(tuple(reversed(MAP.CATALOG.RULES)))

    def test_section_heading_footer_and_relocated_quote_fail(self):
        self.bad_text(43, "2.6.1.1 Name Spaces, Name Generation, and Object Creation", "2.6.2 Buffer Objects")
        self.bad_text(63, MAP.CATALOG.FOOTER, "OpenGL 4.6 Core Profile")
        quote = " ".join(MAP.CATALOG.RULES[3][4])
        moved = self.texts[44].replace(quote, "").replace("2.6.1.3 Shared Object State",
                                                      "2.6.1.3 Shared Object State " + quote)
        self.bad_text(44, self.texts[44], moved)
        with self.assertRaises(MAP.CATALOG.CatalogError):
            MAP.CATALOG.source_window(44, "2.6.2", self.texts[44] + " " + quote)

    def test_wrong_trigger_or_extension_command_in_source_fails(self):
        self.bad_text(65, "BufferSubData", "BufferSubDataEXT")
        self.bad_text(66, "EndTransformFeedback", "PauseTransformFeedback")
        self.bad_text(63, "DeleteRenderbuffers", "BindRenderbuffer")

    def test_rehashed_catalog_trigger_family_and_section_substitutions_fail(self):
        for position, replacement in ((0, "context-and-lifecycle"), (2, "2.6.1.2"), (3, ("GenTextures",))):
            rules = list(MAP.CATALOG.RULES)
            changed = list(rules[0])
            changed[position] = replacement
            rules[0] = tuple(changed)
            self.sealed_catalog_mutation(tuple(rules))

    def test_wrong_profile_locator_and_guessed_grammar_prefix_fail(self):
        for key, value in (("PROFILE", "gl-4.6-core"), ("PRIMARY_PAGE", 44), ("PRIMARY_SECTION", "2.6.2")):
            with patch.object(MAP.CATALOG, key, value), self.assertRaises(MAP.CATALOG.CatalogError):
                self.facts()
        with self.assertRaises(MAP.CATALOG.CatalogError):
            self.facts(normalizer=lambda forms: [name for _, name in forms])

    def test_domain_route_anchor_and_pending_family_omission_fail(self):
        for key, replacement in (("route", "shader-unavailable"), ("source_order", 35), ("reason", "supported"),
                                 ("anchor", {"kind": "heading", "physical_page": 504, "section": "21.40"})):
            def document(path):
                value = copy.deepcopy(MAP.ARTIFACT.document(path))
                for row in value.get("families", []):
                    if row["id"] == "state-tables":
                        row[key] = replacement
                return value
            with self.assertRaises(MAP.CATALOG.CatalogError):
                MAP.CATALOG.bound_family(MAP.DOMAIN_API.CHUNKS, document)

    def test_rehashed_complete_support_and_matrix_promotions_fail(self):
        for change in (lambda value: value["source_coverage"].update(complete=True),
                       lambda value: value.update(profile_support=True), lambda value: value.update(matrix_owner="runtime")):
            self.mutated_artifact(MAP.INVENTORY.name, change, "inventory_sha256")

    def test_rehashed_fragment_omission_and_prefixed_trigger_fail(self):
        name = MAP.INVENTORY.stem + "_context-and-lifecycle.json"
        self.mutated_artifact(name, lambda value: value["raw_entries"].pop(), "fragment_sha256")
        self.mutated_artifact(name, lambda value: value["raw_entries"][0][3].append("glBindBuffer"), "fragment_sha256")


if __name__ == "__main__":
    unittest.main()
