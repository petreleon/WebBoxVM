#!/usr/bin/env python3
"""Hostile checks for the combined GL/GLES no-claim receipt."""

import copy
import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import gl_gles_receipt as receipt


def source(identifier: str) -> dict[str, object]:
    return {"id": identifier, "revision": "1" * 40, "sha256": "2" * 64, "bytes": 1,
            "authority": "Khronos", "producer": "Khronos"}


def local(root: str, members=()) -> dict[str, object]:
    body = {"authority": "WebBoxVM", "producer": "WebBoxVM", "claims": dict(receipt.NO_CLAIMS),
            "cts_executions": 0, "source_root": source(root), "ledger_sha256": "3" * 64,
            "case_count": 1, "case_sequence_sha256": "4" * 64, "members": list(members),
            "configurations": [{"case_list_file": "case.txt"}],
            "glesext_boundary": {"id": "gles-cts-gles32-khr-glesext", "semantics": "not-classified-by-this-ledger"}}
    return body


class GlGlesReceiptTests(unittest.TestCase):
    def values(self):
        ids = tuple(receipt.gles.records.member_id(item[0]) for item in receipt.gles.records.MEMBERS)
        return local(receipt.gl.ROOT_ID), local(receipt.gles.records.ROOT_ID, [{"id": item} for item in ids])

    def build(self):
        gl_value, gles_value = self.values()
        with patch.object(receipt.gl, "ledger", return_value=gl_value), patch.object(receipt.gles, "ledger", return_value=gles_value):
            value = receipt.receipt(b"gl", b"gles", {})
            receipt.validate_receipt(value, b"gl", b"gles", {})
        return value, gl_value, gles_value

    def test_binds_separate_khronos_roots_to_a_local_no_claim_receipt(self):
        value, _gl_value, _gles_value = self.build()
        self.assertEqual((value["authority"], value["producer"], value["cts_executions"]),
                         ("WebBoxVM", "WebBoxVM", 0))
        self.assertTrue(all(not claim for claim in value["claims"].values()))
        self.assertEqual((value["gl"]["source_root"]["id"], value["gles"]["source_root"]["id"]),
                         (receipt.gl.ROOT_ID, receipt.gles.records.ROOT_ID))

    def test_rejects_forged_incomplete_reordered_or_positive_claim_receipt(self):
        value, gl_value, gles_value = self.build()
        mutations = [
            lambda item: item.update(authority="Khronos"),
            lambda item: item.update(cts_executions=1),
            lambda item: item["gles"]["members"].pop(),
            lambda item: item["gles"]["members"].reverse(),
            lambda item: item["gl"].update(ledger_sha256="0" * 64),
            lambda item: item["gles"]["configurations"].pop(),
            lambda item: item["gles"]["glesext_boundary"].update(semantics="mandatory"),
            lambda item: item["gl"]["source_root"].update(revision="0" * 40),
            lambda item: item.update(extra="forged"),
        ]
        for claim in receipt.NO_CLAIMS:
            mutations.append(lambda item, key=claim: item["claims"].update({key: True}))
        for mutate in mutations:
            forged = copy.deepcopy(value); mutate(forged)
            with patch.object(receipt.gl, "ledger", return_value=gl_value), patch.object(receipt.gles, "ledger", return_value=gles_value):
                with self.subTest(mutate=mutate), self.assertRaises(receipt.ReceiptError):
                    receipt.validate_receipt(forged, b"gl", b"gles", {})

    def test_rejects_an_inner_positive_claim_wrong_authority_or_bad_member_order(self):
        gl_value, gles_value = self.values()
        cases = [
            (dict(gl_value, claims={**receipt.NO_CLAIMS, "conformance": True}), gles_value),
            (dict(gl_value, source_root={**gl_value["source_root"], "authority": "WebBoxVM"}), gles_value),
            (gl_value, dict(gles_value, members=list(reversed(gles_value["members"])))),
        ]
        for current_gl, current_gles in cases:
            with patch.object(receipt.gl, "ledger", return_value=current_gl), patch.object(receipt.gles, "ledger", return_value=current_gles):
                with self.subTest(gl=current_gl, gles=current_gles), self.assertRaises(receipt.ReceiptError):
                    receipt.receipt(b"gl", b"gles", {})

    def test_receipt_hash_is_content_addressed(self):
        value, _gl_value, _gles_value = self.build()
        body = {key: item for key, item in value.items() if key != "receipt_sha256"}
        self.assertEqual(value["receipt_sha256"], hashlib.sha256(receipt.canonical(body)).hexdigest())

    def test_external_cache_refuses_repository_or_symlinked_roots(self):
        repository = receipt.gl.roots.repository_root(receipt.HERE)
        with self.assertRaises(receipt.ReceiptError):
            receipt.external_cache(repository)
        with tempfile.TemporaryDirectory() as temporary:
            escaped = Path(temporary, "inside-repository")
            escaped.symlink_to(repository, target_is_directory=True)
            with self.assertRaises(receipt.ReceiptError):
                receipt.external_cache(escaped)

    def test_cached_bytes_refuses_a_symlinked_source_component(self):
        source = receipt.gl.roots.source_input(receipt.gl.root_record())
        with tempfile.TemporaryDirectory() as temporary:
            cache = receipt.external_cache(Path(temporary))
            target = cache.root / "payload"
            target.write_bytes(b"payload")
            path = cache.root / source.local_cache
            path.parent.mkdir(parents=True)
            path.symlink_to(target)
            with self.assertRaises(receipt.ReceiptError):
                receipt.cached_bytes(cache, source)


if __name__ == "__main__":
    unittest.main()
