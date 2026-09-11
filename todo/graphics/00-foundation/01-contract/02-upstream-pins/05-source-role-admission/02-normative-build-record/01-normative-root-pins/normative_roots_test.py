#!/usr/bin/env python3
"""Focused hostile checks for the F02.5.2.1 normative-root catalog."""

import unittest

from normative_roots import EXPECTED, ROOTS, RootError, catalog, provenance, source_input, validate_normative_catalog


class NormativeRootsTests(unittest.TestCase):
    def test_exact_four_roots_have_distinct_roles(self):
        self.assertEqual(validate_normative_catalog(catalog()), tuple(item["id"] for item in ROOTS))
        scopes = {item["id"]: item["scope"] for item in ROOTS}
        self.assertEqual(scopes["vulkan-registry"], "registry-metadata")
        self.assertEqual({scope for identifier, scope in scopes.items() if identifier != "vulkan-registry"},
                         {"normative-source"})

    def test_source_inputs_preserve_external_cache_keys_and_provenance(self):
        for record in ROOTS:
            with self.subTest(identifier=record["id"]):
                source = source_input(record)
                self.assertEqual(str(source.local_cache), record["artifact"])
                self.assertEqual(source.url, record["immutable_url"])
                self.assertEqual(provenance(record), f"https://github.com/KhronosGroup/{record['immutable_url'].split('/')[4]}/tree/{record['revision']}")

    def test_rejects_incomplete_or_locally_relabelled_roots(self):
        cases = []
        partial = catalog(); partial["records"].pop(); cases.append(partial)
        local = catalog(); local["records"][0]["producer"] = "WebBoxVM"; cases.append(local)
        metadata = catalog(); metadata["records"][3]["scope"] = "normative-source"; cases.append(metadata)
        for value in cases:
            with self.subTest(value=value), self.assertRaises(RootError):
                validate_normative_catalog(value)

    def test_rejects_mutable_or_foreign_identity_and_terms_changes(self):
        cases = []
        mutable = catalog(); mutable["records"][0]["immutable_url"] = mutable["records"][0]["immutable_url"].replace(EXPECTED["opengl-46-spec"]["revision"], "main"); cases.append(mutable)
        foreign = catalog(); foreign["records"][1]["immutable_url"] = foreign["records"][1]["immutable_url"].replace("KhronosGroup", "Elsewhere"); cases.append(foreign)
        for field, value in (("license", ""), ("attribution", "WebBoxVM"), ("sha256", "0" * 64), ("bytes", 1)):
            changed = catalog(); changed["records"][2][field] = value; cases.append(changed)
        for value in cases:
            with self.subTest(value=value), self.assertRaises(RootError):
                validate_normative_catalog(value)

    def test_records_are_copyable_without_mutating_reviewed_constants(self):
        changed = catalog()
        changed["records"][0]["claims"]["api_support"] = True
        with self.assertRaises(RootError):
            validate_normative_catalog(changed)
        self.assertFalse(ROOTS[0]["claims"]["api_support"])


if __name__ == "__main__":
    unittest.main()
