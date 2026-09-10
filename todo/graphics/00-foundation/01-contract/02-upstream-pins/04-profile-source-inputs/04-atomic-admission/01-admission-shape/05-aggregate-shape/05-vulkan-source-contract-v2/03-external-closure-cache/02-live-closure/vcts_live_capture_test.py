#!/usr/bin/env python3
"""Hermetic end-to-end tests for the VCTS live capture coordinator."""

from __future__ import annotations

import errno
import hashlib
import json
import sys
import tempfile
import unittest
from io import BytesIO
from pathlib import Path
from types import SimpleNamespace

HERE = Path(__file__).resolve().parent
SCHEMA = HERE.parent.parent / "02-canonical-suite-schema"
sys.path.insert(0, str(HERE))
import vcts_live_capture as runner


def blob(data: bytes) -> str:
    return hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest()


class Response:
    def __init__(self, data: bytes, url: str):
        self.body, self.url = BytesIO(data), url
        self.headers = {"Content-Length": str(len(data)), "Content-Encoding": "identity"}

    def geturl(self):
        return self.url

    def getcode(self):
        return 200

    def read(self, size=-1):
        return self.body.read(size)

    def close(self):
        self.body.close()


class RawOpener:
    def __init__(self, rows):
        self.rows, self.requests = rows, []

    def open(self, request, timeout):
        self.requests.append(request.full_url)
        return Response(self.rows[request.full_url], request.full_url)


class LiveCaptureTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir="/private/tmp")
        self.directory = Path(self.temp.name)
        self.identity = SCHEMA / "vcts_root_identity.json"
        self.root = runner.cache.identity.validate(self.identity)
        self.selector = ("\n".join(path.removeprefix("external/vulkancts/mustpass/main/")
                                   for path in self.root.direct_members) + "\n").encode()
        self.payloads = {path: f"member-{index}".encode() for index, path in enumerate(self.root.direct_members)}
        self.payloads[self.root.direct_members[1]] = self.payloads[self.root.direct_members[0]]
        self.tree = runner.plan.build(self.api_input(), self.identity)
        self.raw = RawOpener(self.raw_rows())
        self.output = self.directory / "output"
        self.output.mkdir(mode=0o700)
        self.outputs = tuple(self.output / name for name in ("plan.json", "ledger.json", "receipt.json"))

    def tearDown(self):
        self.temp.cleanup()

    def api_input(self):
        hashes, parent, chain = iter(f"{value:040x}" for value in range(1, 300)), "1" * 40, []
        for name in runner.plan.TREE_NAMES:
            child = next(hashes)
            chain.append({"name": name, "mode": "040000", "parent_tree_sha1": parent, "tree_sha1": child})
            parent = child
        return {"schema": 1, "kind": runner.plan.INPUT_KIND, "tag_ref": runner.plan.identity.EXPECTED["tag_ref"],
                "tag_object_sha1": runner.plan.identity.EXPECTED["tag_object_sha1"],
                "tag_target_sha1": self.root.peeled_commit, "tag_target_type": "commit",
                "peeled_commit_sha1": self.root.peeled_commit, "commit_tree_sha1": "1" * 40,
                "tree_chain": chain, "root": {"path": self.root.root_path, "mode": "100644",
                "blob_sha1": blob(self.selector), "bytes": len(self.selector)},
                "members": [{"path": path, "mode": "100644", "blob_sha1": blob(data), "bytes": len(data)}
                            for path, data in self.payloads.items()]}

    def raw_rows(self):
        rows = {runner.plan.identity.ROOT["immutable_url"]: self.selector}
        rows.update({runner.cache.raw_url(self.root.peeled_commit, path): data for path, data in self.payloads.items()})
        return rows

    def capture_closure(self):
        return runner.capture(self.identity, self.directory / "external", *self.outputs, 5.0,
                              raw_opener=self.raw, tree_capture=SimpleNamespace(tree_plan=self.tree))

    def test_captures_99_raw_streams_then_offline_replays_and_discards_stage(self):
        result = self.capture_closure()
        plan_path, ledger_path, receipt_path = self.outputs
        self.assertEqual((result.member_count, len(self.raw.requests)), (98, 99))
        self.assertEqual(runner.plan.validate(plan_path, self.identity).digest, result.plan_sha256)
        self.assertEqual(runner.cache.ledger.validate(ledger_path, self.identity).digest, result.ledger_sha256)
        document = json.loads(receipt_path.read_text(encoding="utf-8"))
        self.assertEqual((document["status"], document["admitted"], document["satisfies_vulkan_14_core_manifest"]),
                         ("external-cache-offline-verified-unadmitted", False, False))
        runner.cache.verify(self.identity, ledger_path, self.directory / "external", runner.cache.repository_root(HERE))
        leaf = self.directory / "external" / "webboxvm-graphics" / "v2" / "vcts-live-stage" / self.root.digest / result.plan_sha256
        self.assertFalse(leaf.exists())

    def test_refuses_a_committed_output_bundle_before_any_raw_request(self):
        for path in self.outputs:
            path.write_text("reserved", encoding="utf-8")
        with self.assertRaises(runner.LiveCaptureError):
            self.capture_closure()
        self.assertEqual(self.raw.requests, [])

    def test_refuses_a_dangling_output_symlink_before_any_raw_request(self):
        self.outputs[0].symlink_to(self.directory / "not-created")
        with self.assertRaises(runner.LiveCaptureError):
            self.capture_closure()
        self.assertTrue(self.outputs[0].is_symlink())
        self.assertEqual(self.raw.requests, [])

    def test_rejects_bad_raw_payload_without_success_artifacts(self):
        url = runner.cache.raw_url(self.root.peeled_commit, self.root.direct_members[0])
        self.raw.rows[url] = b"x" * len(self.payloads[self.root.direct_members[0]])
        with self.assertRaises(runner.LiveCaptureError):
            self.capture_closure()
        self.assertFalse(any(path.exists() for path in self.outputs))

    def test_discards_the_stage_after_a_post_stage_cache_failure(self):
        original = runner.cache.populate
        runner.cache.populate = lambda *args: (_ for _ in ()).throw(runner.cache.io.CacheError("injected"))
        try:
            with self.assertRaises(runner.LiveCaptureError):
                self.capture_closure()
        finally:
            runner.cache.populate = original
        leaf = self.directory / "external" / "webboxvm-graphics" / "v2" / "vcts-live-stage" / self.root.digest
        self.assertFalse(any(leaf.iterdir()))
        result = self.capture_closure()
        self.assertEqual(result.member_count, 98)

    def test_normalizes_an_unexpected_stage_filesystem_error(self):
        original = runner.stage.capture
        runner.stage.capture = lambda *args: (_ for _ in ()).throw(OSError(errno.EIO, "injected"))
        try:
            with self.assertRaises(runner.LiveCaptureError) as caught:
                self.capture_closure()
        finally:
            runner.stage.capture = original
        self.assertIn("errno 5", str(caught.exception))
        self.assertFalse(any(path.exists() or path.is_symlink() for path in self.outputs))


if __name__ == "__main__":
    unittest.main(verbosity=2)
