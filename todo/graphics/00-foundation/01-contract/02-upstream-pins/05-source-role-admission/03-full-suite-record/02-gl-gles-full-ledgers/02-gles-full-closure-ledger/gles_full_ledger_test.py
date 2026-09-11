#!/usr/bin/env python3
"""Focused exact-shape and hostile-boundary checks for the GLES full closure."""

import copy
import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import gles_full_ledger as ledger
import gles_ledger_records as records


def xml(configurations):
    nodes = []
    for name, command, label, os_name, first in configurations:
        nodes.append(f'<Configuration caseListFile="{name}" commandLine="{command}" name="{label}" os="{os_name}" useForFirstEGLConfig="{first}"/>')
    return ("<Mustpass version=\"main\"><TestPackage name=\"Khronos Mustpass ES\">" + "".join(nodes)
            + "</TestPackage></Mustpass>").encode()


class GlesFullLedgerTests(unittest.TestCase):
    def test_exact_catalog_has_root_and_five_unclassified_members(self):
        value = records.catalog()
        records.validate_catalog(value)
        items = value["records"]
        self.assertEqual((items[0]["id"], len(items)), ("gles-cts-main", 6))
        self.assertEqual([item["member_path"].rsplit("/", 1)[-1] for item in items[1:]],
                         [item[0] for item in records.MEMBERS])
        self.assertTrue(all(item["scope"] == "suite-member" for item in items[1:]))
        self.assertTrue(all(not claim for claim in records.NO_CLAIMS.values()))

    def test_member_inputs_keep_exact_root_anchor_and_cache_identity(self):
        root = records.catalog()["records"][0]
        for item in records.member_records(root):
            with self.subTest(identifier=item["id"]):
                source = records.member_input(item)
                self.assertEqual((source.identifier, source.revision), (item["id"], root["revision"]))
                self.assertEqual((str(source.local_cache), source.sha256), (item["artifact"], item["sha256"]))
        with self.assertRaises(records.RecordError):
            records.member_input({**records.member_records(root)[0], "revision": "0" * 40})

    def test_xml_has_exact_thirteen_ordered_configurations(self):
        parsed = ledger.configurations(xml(ledger.CONFIGURATIONS))
        self.assertEqual(parsed, ledger.CONFIGURATIONS)
        self.assertEqual(sum(item[0] == "gles32-khr-glesext.txt" for item in parsed), 1)

    def test_rejects_malformed_dtd_missing_reordered_or_substituted_xml(self):
        cases = [b"<!DOCTYPE x><Mustpass/>", b"<Mustpass version=\"main\"/>",
                 xml(tuple(reversed(ledger.CONFIGURATIONS))),
                 xml(ledger.CONFIGURATIONS).replace(b"gles2-khr-main.txt", b"other.txt", 1)]
        for data in cases:
            with self.subTest(data=data[:40]), self.assertRaises(ledger.LedgerError):
                ledger.configurations(data)

    def test_revalidates_a_serialized_glesext_boundary_against_sources(self):
        configs, member_data = (ledger.config("case.txt"),), b"A\n"
        root_data = xml(configs)
        root = {"id": records.ROOT_ID, "revision": "1" * 40, "sha256": hashlib.sha256(root_data).hexdigest(),
                "bytes": len(root_data), "license": "Apache-2.0", "attribution": "Khronos", "authority": "Khronos",
                "producer": "Khronos", "scope": "full-conformance-suite"}
        member = {"id": "case", "revision": root["revision"], "sha256": hashlib.sha256(member_data).hexdigest(),
                  "bytes": len(member_data), "license": "Apache-2.0", "attribution": "Khronos", "authority": "Khronos",
                  "producer": "Khronos", "scope": "suite-member"}
        with patch.object(records, "catalog", return_value={"schema": 1, "records": [root, member]}), \
                patch.object(records, "validate_catalog"), patch.object(records, "MEMBERS", (("case.txt", member["sha256"], 2, 1),)), \
                patch.object(ledger, "CONFIGURATIONS", configs):
            value = ledger.ledger(root_data, {"case": member_data})
            ledger.validate_ledger(value, root_data, {"case": member_data})
            value["glesext_boundary"]["semantics"] = "mandatory"
            with self.assertRaises(ledger.LedgerError):
                ledger.validate_ledger(value, root_data, {"case": member_data})

    def test_rejects_bad_flat_lists_catalog_mutations_and_nonempty_cache(self):
        for data in (b"A\n\n", b" A\n", b"A\x00\n", b"A\nA\n", b"A"):
            with self.subTest(data=data), self.assertRaises(ledger.LedgerError):
                ledger.flat_cases(data)
        for action in (lambda value: value["records"].pop(), lambda value: value["records"].reverse(),
                       lambda value: value["records"][1].update(revision="0" * 40),
                       lambda value: value["records"].append(copy.deepcopy(value["records"][1]))):
            value = records.catalog(); action(value)
            with self.assertRaises(records.RecordError):
                records.validate_catalog(value)
        with tempfile.TemporaryDirectory() as temporary:
            Path(temporary, "old").write_bytes(b"old")
            with self.assertRaises(ledger.LedgerError):
                ledger.refresh(Path(temporary), 1)


if __name__ == "__main__":
    unittest.main()
