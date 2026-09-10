#!/usr/bin/env python3
"""Hermetic synthetic tests for the V2 external closure cache contract."""

from __future__ import annotations

import hashlib
import json
import os
import sys
import tempfile
import unittest
from io import BytesIO
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import vcts_cache_contract as contract


class Response:
    def __init__(self, payload: bytes, url: str, headers: dict[str, str]):
        self.body, self.url, self.headers, self.read_sizes = BytesIO(payload), url, headers, []

    def close(self):
        self.body.close()

    def geturl(self):
        return self.url

    def getcode(self):
        return 200

    def read(self, size=-1):
        self.read_sizes.append(size)
        return self.body.read(size)


class Opener:
    def __init__(self, rows):
        self.rows, self.requests, self.responses = rows, [], []

    def open(self, request, timeout):
        self.requests.append((request.full_url, timeout))
        payload, url, headers = self.rows[request.full_url]
        response = Response(payload, url, headers)
        self.responses.append(response)
        return response


def blob(payload: bytes) -> str:
    return hashlib.sha1(f"blob {len(payload)}\0".encode() + payload).hexdigest()


class CacheContractTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(dir="/private/tmp")
        self.root = contract.identity.validate(contract.SCHEMA / "vcts_root_identity.json")
        self.repository, self.cache = contract.repository_root(HERE), Path(self.temp.name) / "cache"
        self.ledger = Path(self.temp.name) / "synthetic-ledger.json"
        paths = [path.removeprefix("external/vulkancts/mustpass/main/") for path in self.root.direct_members]
        self.root_payload = ("\n".join(paths) + "\n").encode()
        self.payloads = {path: f"synthetic-member-{number}".encode()
                         for number, path in enumerate(self.root.direct_members)}
        self.document = self.make_ledger()
        self.assertEqual((len(self.root_payload), hashlib.sha256(self.root_payload).hexdigest()),
                         (contract.identity.ROOT["bytes"], contract.identity.ROOT["sha256"]))

    def tearDown(self) -> None:
        self.temp.cleanup()

    def make_ledger(self) -> dict[str, object]:
        rows = [{"path": path, "parent_path": None, "revision": self.root.peeled_commit,
                 "blob_sha1": blob(payload), "sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload)}
                for path, payload in self.payloads.items()]
        value = {"schema": 1, "kind": "canonical-upstream-suite", "root_identity_sha256": self.root.digest,
                 "limits": contract.ledger.LIMITS, "member_count": len(rows),
                 "member_total_bytes": sum(row["bytes"] for row in rows), "ledger_sha256": "0" * 64, "members": rows}
        value["ledger_sha256"] = contract.ledger.digest(value)
        self.ledger.write_text(json.dumps(value), encoding="utf-8")
        return value

    def transport(self, root_url=None, bad_length=False) -> Opener:
        rows = {contract.identity.ROOT["immutable_url"]: (
            self.root_payload, root_url or contract.identity.ROOT["immutable_url"],
            {"Content-Length": str(len(self.root_payload))})}
        for path, payload in self.payloads.items():
            rows[contract.raw_url(self.root.peeled_commit, path)] = (
                payload, contract.raw_url(self.root.peeled_commit, path),
                {"Content-Length": str(len(payload) + int(bad_length))})
        return Opener(rows)

    def populate(self, opener=None):
        return contract.populate(contract.SCHEMA / "vcts_root_identity.json", self.ledger, self.cache,
                                 self.repository, 5.0, opener or self.transport())

    def closure(self) -> Path:
        return (self.cache / "webboxvm-graphics" / "v2" / "vulkan-cts-mustpass" /
                self.root.digest / self.document["ledger_sha256"])

    def test_populates_streams_and_verifies_without_mutation(self) -> None:
        opener = self.transport()
        receipt = self.populate(opener)
        before = {path.relative_to(self.cache): path.read_bytes()
                  for path in self.cache.rglob("*") if path.is_file()}
        checked = contract.verify(contract.SCHEMA / "vcts_root_identity.json", self.ledger,
                                  self.cache, self.repository)
        after = {path.relative_to(self.cache): path.read_bytes()
                 for path in self.cache.rglob("*") if path.is_file()}
        marker = contract.receipt.parse((self.closure() / contract.receipt.name(self.document)).read_bytes())
        self.assertEqual((receipt.member_count, receipt.reused, checked.digest), (98, False, receipt.digest))
        self.assertEqual(before, after)
        self.assertEqual((marker["status"], marker["admitted"], marker["cutover_ready"]),
                         ("external-cache-verified-unadmitted", False, False))
        self.assertEqual((self.closure() / "members").stat().st_mode & 0o777, 0o700)
        self.assertEqual(len(opener.requests), 99)
        self.assertTrue(all(size <= contract.io.CHUNK for row in opener.responses for size in row.read_sizes))

    def test_rejects_redirect_length_and_unsafe_raw_url(self) -> None:
        with self.assertRaises(contract.io.CacheError):
            self.populate(self.transport(root_url="https://example.invalid/substitute"))
        with self.assertRaises(contract.io.CacheError):
            self.populate(self.transport(bad_length=True))
        with self.assertRaises(contract.io.CacheError):
            contract.raw_url("z" * 40, self.root.direct_members[0])
        with self.assertRaises(contract.io.CacheError):
            contract.raw_url(self.root.peeled_commit, "../escape.txt")

    def test_rejects_missing_tampered_and_symlinked_cache_without_repair(self) -> None:
        with self.assertRaises(contract.io.CacheError):
            contract.verify(contract.SCHEMA / "vcts_root_identity.json", self.ledger, self.cache, self.repository)
        self.assertFalse(self.cache.exists())
        self.populate()
        marker = self.closure() / contract.receipt.name(self.document)
        marker.write_bytes(b"{}")
        with self.assertRaises(contract.io.CacheError):
            contract.verify(contract.SCHEMA / "vcts_root_identity.json", self.ledger, self.cache, self.repository)
        with self.assertRaises(contract.io.CacheError):
            self.populate()
        self.assertEqual(marker.read_bytes(), b"{}")
        target, link = Path(self.temp.name) / "target", Path(self.temp.name) / "link"
        target.mkdir()
        link.symlink_to(target, target_is_directory=True)
        with self.assertRaises(contract.io.CacheError):
            contract.verify(contract.SCHEMA / "vcts_root_identity.json", self.ledger, link, self.repository)
        unsafe = Path(self.temp.name) / "unsafe"
        unsafe.mkdir()
        unsafe.chmod(0o777)
        with self.assertRaises(contract.io.CacheError):
            contract.populate(contract.SCHEMA / "vcts_root_identity.json", self.ledger, unsafe, self.repository)
        alias = Path(self.temp.name) / "repo-alias"
        alias.symlink_to(self.repository, target_is_directory=True)
        with self.assertRaises(contract.io.CacheError):
            contract.populate(contract.SCHEMA / "vcts_root_identity.json", self.ledger, alias / "cache", self.repository)

    def test_selector_requires_terminal_lf_and_no_crlf(self) -> None:
        for payload in (self.root_payload.replace(b"\n", b"\r\n"), self.root_payload[:-1]):
            path = Path(self.temp.name) / "selector"
            path.write_bytes(payload)
            fd = os.open(self.temp.name, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
            try:
                with self.assertRaises(contract.io.CacheError):
                    contract.io.selector(fd, "selector", self.root.direct_members, contract.ROOT_LIMIT)
            finally:
                os.close(fd)

    def test_uses_immutable_ledger_snapshot_after_source_swap(self) -> None:
        original = contract.ledger.validate
        def swap(frozen, identity_path):
            self.ledger.write_bytes(b"{}")
            return original(frozen, identity_path)
        with patch.object(contract.ledger, "validate", side_effect=swap):
            _, checked, document = contract.inputs(contract.SCHEMA / "vcts_root_identity.json", self.ledger)
        self.assertEqual((checked.digest, document["ledger_sha256"]),
                         (self.document["ledger_sha256"], self.document["ledger_sha256"]))


if __name__ == "__main__":
    unittest.main(verbosity=2)
