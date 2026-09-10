#!/usr/bin/env python3
"""Hermetic raw-stream and closed-world replay tests for the VCTS live stage."""

from __future__ import annotations

import errno
import hashlib
import json
import sys
import tempfile
import unittest
from io import BytesIO
from pathlib import Path
from urllib.request import Request

HERE = Path(__file__).resolve().parent
CACHE = HERE.parent / "01-cache-contract"
SCHEMA = HERE.parent.parent / "02-canonical-suite-schema"
for location in (HERE, CACHE):
    sys.path.insert(0, str(location))
import vcts_cache_contract as cache
import vcts_live_stage as live

def blob(data: bytes) -> str:
    return hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest()


class Response:
    def __init__(self, data, url, headers):
        self.body, self.url, self.headers = BytesIO(data), url, headers

    def geturl(self):
        return self.url

    def getcode(self):
        return 200

    def read(self, size=-1):
        return self.body.read(size)

    def close(self):
        self.body.close()

class Opener:
    def __init__(self, rows):
        self.rows, self.requests = rows, []

    def open(self, request, timeout):
        self.requests.append((request.full_url, timeout))
        data, url, headers = self.rows[request.full_url]
        return Response(data, url, headers)

class LiveStageTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir="/private/tmp")
        self.identity = SCHEMA / "vcts_root_identity.json"
        self.root = live.plan.checked_identity(self.identity)
        self.repository = cache.repository_root(HERE)
        self.payloads = {path: f"member-{number}".encode() for number, path in enumerate(self.root.direct_members)}
        self.payloads[self.root.direct_members[1]] = self.payloads[self.root.direct_members[0]]
        self.selector = ("\n".join(path.removeprefix("external/vulkancts/mustpass/main/")
                                     for path in self.root.direct_members) + "\n").encode()
        self.plan_path = Path(self.temp.name) / "plan.json"
        self.plan_path.write_text(json.dumps(live.plan.build(self.plan_input(), self.identity)), encoding="utf-8")
        self.source = Opener(self.raw_rows())

    def tearDown(self):
        self.temp.cleanup()

    def plan_input(self):
        chain, parent = [], "1" * 40
        for number, name in enumerate(live.plan.TREE_NAMES, 2):
            child = f"{number:040x}"
            chain.append({"name": name, "mode": "040000", "parent_tree_sha1": parent, "tree_sha1": child})
            parent = child
        return {"schema": 1, "kind": live.plan.INPUT_KIND, "tag_ref": live.plan.identity.EXPECTED["tag_ref"],
                "tag_object_sha1": live.plan.identity.EXPECTED["tag_object_sha1"], "tag_target_sha1": self.root.peeled_commit,
                "tag_target_type": "commit", "peeled_commit_sha1": self.root.peeled_commit, "commit_tree_sha1": "1" * 40,
                "tree_chain": chain, "root": {"path": self.root.root_path, "mode": "100644", "blob_sha1": blob(self.selector),
                "bytes": len(self.selector)}, "members": [{"path": path, "mode": "100644", "blob_sha1": blob(data),
                "bytes": len(data)} for path, data in self.payloads.items()]}

    def raw_rows(self):
        rows = {live.plan.identity.ROOT["immutable_url"]: (self.selector, live.plan.identity.ROOT["immutable_url"],
                {"Content-Length": str(len(self.selector))})}
        for path, data in self.payloads.items():
            url = cache.raw_url(self.root.peeled_commit, path)
            rows[url] = (data, url, {"Content-Length": str(len(data))})
        return rows

    def capture(self):
        return live.capture(self.identity, self.plan_path, Path(self.temp.name) / "stage", self.repository, 5, self.source)

    def ledger(self, stage):
        rows = [member.ledger_row() for member in stage.members]
        value = {"schema": 1, "kind": "canonical-upstream-suite", "root_identity_sha256": self.root.digest,
                 "limits": cache.ledger.LIMITS, "member_count": len(rows), "member_total_bytes": sum(row["bytes"] for row in rows),
                 "ledger_sha256": "0" * 64, "members": rows}
        value["ledger_sha256"] = cache.ledger.digest(value)
        path = Path(self.temp.name) / "ledger.json"
        path.write_text(json.dumps(value), encoding="utf-8")
        return path

    def test_streams_99_once_and_replays_them_to_the_cache(self):
        stage = self.capture()
        self.addCleanup(stage.close)
        self.assertEqual((len(self.source.requests), len(stage.members), stage.root.sha256), (99, 98, live.plan.identity.ROOT["sha256"]))
        expected = [live.plan.identity.ROOT["immutable_url"],
                    *(cache.raw_url(self.root.peeled_commit, path) for path in self.root.direct_members)]
        self.assertEqual([url for url, _ in self.source.requests], expected)
        with stage.replay() as local:
            receipt = cache.populate(self.identity, self.ledger(stage), Path(self.temp.name) / "cache", self.repository, 5, local)
            local.assert_complete()
            self.assertEqual(len(local.used), 98)
        self.assertEqual((receipt.member_count, receipt.reused), (98, False))
        cache.verify(self.identity, self.ledger(stage), Path(self.temp.name) / "cache", self.repository)
        stage.discard()
        self.assertFalse(stage.directory.exists())
    def test_rejects_a_same_length_payload_that_misses_its_git_blob_identity(self):
        path = self.root.direct_members[0]
        url = cache.raw_url(self.root.peeled_commit, path)
        self.source.rows[url] = (b"x" * len(self.payloads[path]), url, {"Content-Length": str(len(self.payloads[path]))})
        with self.assertRaises(live.StageError):
            self.capture()
        leaf = Path(self.temp.name) / "stage" / "webboxvm-graphics" / "v2" / "vcts-live-stage" / self.root.digest
        self.assertFalse(any(leaf.iterdir()))
        self.source = Opener(self.raw_rows())
        stage = self.capture()
        self.assertTrue(stage.directory.is_dir())
        stage.discard()
    def test_normalizes_a_stage_write_error_and_removes_its_partial_directory(self):
        original, calls = live.os.write, 0
        def fail_second_write(*args):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError(errno.ENOSPC, "injected")
            return original(*args)

        live.os.write = fail_second_write
        try:
            with self.assertRaises(live.StageError) as caught:
                self.capture()
        finally:
            live.os.write = original
        leaf = Path(self.temp.name) / "stage" / "webboxvm-graphics" / "v2" / "vcts-live-stage" / self.root.digest
        self.assertIn("errno 28", str(caught.exception))
        self.assertFalse(any(leaf.iterdir()))
    def test_closing_a_raw_response_cannot_strand_a_part_file(self):
        original = self.source.open
        def hostile(*args, **kwargs):
            response, close = original(*args, **kwargs), None
            close = response.close
            response.close = lambda: (close(), (_ for _ in ()).throw(OSError(errno.EIO, "injected")))
            return response

        self.source.open = hostile
        with self.assertRaises(live.StageError) as caught:
            self.capture()
        leaf = Path(self.temp.name) / "stage" / "webboxvm-graphics" / "v2" / "vcts-live-stage" / self.root.digest
        self.assertIn("errno 5", str(caught.exception))
        self.assertFalse(any(leaf.iterdir()))
    def test_replay_rejects_tampering_repetition_and_incomplete_use(self):
        stage = self.capture()
        self.addCleanup(stage.close)
        target = stage.directory / "member-000.source"
        target.write_bytes(b"x" * target.stat().st_size)
        with stage.replay() as local:
            with self.assertRaises(live.StageError):
                local.open(Request(stage.members[0].url), 5)
        with stage.replay() as local:
            response = local.open(Request(stage.root.url), 5)
            response.close()
            with self.assertRaises(live.StageError):
                local.open(Request(stage.root.url), 5)
            with self.assertRaises(live.StageError):
                local.assert_complete()
if __name__ == "__main__":
    unittest.main(verbosity=2)
