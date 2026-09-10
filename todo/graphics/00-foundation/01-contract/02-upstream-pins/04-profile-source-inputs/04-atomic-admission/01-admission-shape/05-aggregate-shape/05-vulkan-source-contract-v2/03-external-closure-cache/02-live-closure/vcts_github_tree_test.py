#!/usr/bin/env python3
"""Hermetic hostile tests for immutable GitHub VCTS tree metadata capture."""
from __future__ import annotations

import copy
import json
import sys
import unittest
from io import BytesIO
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import vcts_github_tree as github


class Response:
    def __init__(self, body: bytes, url: str, headers: dict[str, str], code: int = 200):
        self.body, self.url, self.headers, self.code, self.read_sizes = BytesIO(body), url, headers, code, []

    def close(self):
        self.body.close()

    def geturl(self):
        return self.url

    def getcode(self):
        return self.code

    def read(self, size=-1):
        self.read_sizes.append(size)
        return self.body.read(size)


class Opener:
    def __init__(self, rows):
        self.rows, self.requests, self.responses = copy.deepcopy(rows), [], []
        self.urls, self.headers, self.codes = {}, {}, {}

    def open(self, request, timeout):
        self.requests.append(request)
        body = self.rows[request.full_url] if isinstance(self.rows[request.full_url], bytes) else json.dumps(self.rows[request.full_url], separators=(",", ":")).encode()
        headers = {"Content-Length": str(len(body)), "Content-Type": "application/json"}
        headers.update(self.headers.get(request.full_url, {}))
        response = Response(body, self.urls.get(request.full_url, request.full_url), headers,
                            self.codes.get(request.full_url, 200))
        self.responses.append(response)
        return response


class GitHubTreeTest(unittest.TestCase):
    def setUp(self) -> None:
        self.identity = github.plan.SCHEMA / "vcts_root_identity.json"
        self.root = github.plan.checked_identity(self.identity)
        self.rows, self.urls = self.documents()

    def documents(self):
        expected, hashes = github.plan.identity.EXPECTED, iter(f"{item:040x}" for item in range(1, 300))
        tag_sha, commit_sha, commit_tree = expected["tag_object_sha1"], self.root.peeled_commit, next(hashes)
        ref = github.ref_endpoint(expected["tag_name"])
        rows = {ref: {"ref": expected["tag_ref"], "object": {"type": "tag", "sha": tag_sha}},
                github.endpoint("tags", tag_sha): {"sha": tag_sha, "tag": expected["tag_name"],
                                                    "object": {"type": "commit", "sha": commit_sha}},
                github.endpoint("commits", commit_sha): {"sha": commit_sha, "tree": {"sha": commit_tree}}}
        parent, tree_urls = commit_tree, []
        for name in github.plan.TREE_NAMES:
            child, url = next(hashes), github.endpoint("trees", parent)
            entries = [{"path": name, "mode": "040000", "type": "tree", "sha": child}]
            if name == github.plan.TREE_NAMES[-1]:
                entries.append({"path": "vk-default.txt", "mode": "100644", "type": "blob", "sha": next(hashes),
                                "size": github.plan.identity.ROOT["bytes"]})
                main_url = url
            rows[url], parent = {"sha": parent, "truncated": False, "tree": entries}, child
            tree_urls.append(url)
        prefix = self.root.root_path.removesuffix(".txt") + "/"
        names = [item.removeprefix(prefix) for item in self.root.direct_members]
        parents = sorted({"/".join(item.split("/")[:depth]) for item in names for depth in range(1, item.count("/") + 1)})
        entries = ([{"path": item, "mode": "040000", "type": "tree", "sha": next(hashes)} for item in parents]
                   + [{"path": item, "mode": "100644", "type": "blob", "sha": next(hashes), "size": count + 1}
                      for count, item in enumerate(names)])
        leaf_url = github.endpoint("trees", parent, True)
        rows[leaf_url] = {"sha": parent, "truncated": False, "tree": entries}
        return rows, {"ref": ref, "main": main_url, "leaf": leaf_url, "trees": tree_urls}

    def opener(self):
        return Opener(self.rows)

    def test_capture_binds_ref_metadata_and_ordered_plan(self) -> None:
        opener, result = self.opener(), None
        result = github.capture(self.identity, 5.0, opener)
        self.assertEqual((result.tree_plan["member_count"], result.tree_plan["member_total_bytes"]), (98, sum(range(1, 99))))
        self.assertEqual(tuple(row["path"] for row in result.api_input["members"]), self.root.direct_members)
        self.assertEqual(opener.requests[0].full_url, self.urls["ref"])
        self.assertEqual(opener.requests[0].get_header("Accept-encoding"), "identity")
        self.assertEqual(len(opener.requests), 9)
        self.assertTrue(all("/git/" in item.full_url for item in opener.requests))
        self.assertTrue(all(size <= 64 * 1024 for response in opener.responses for size in response.read_sizes))

    def test_rejects_ref_and_tree_substitution_partiality_and_bad_sizes(self) -> None:
        cases = (
            ("ref", lambda rows: rows[self.urls["ref"]]["object"].__setitem__("sha", "f" * 40)),
            ("tag", lambda rows: rows[github.endpoint("tags", github.plan.identity.EXPECTED["tag_object_sha1"])]["object"].__setitem__("sha", "f" * 40)),
            ("link", lambda rows: rows[self.urls["trees"][0]]["tree"][0].__setitem__("mode", "100644")),
            ("root", lambda rows: rows[self.urls["main"]]["tree"][1].__setitem__("size", 1)),
            ("member", lambda rows: rows[self.urls["leaf"]]["tree"][-1].__setitem__("size", github.plan.MAX_MEMBER_BYTES + 1)),
            ("missing", lambda rows: rows[self.urls["leaf"]]["tree"].pop()),
            ("truncated", lambda rows: rows[self.urls["leaf"]].__setitem__("truncated", True)),
        )
        for label, change in cases:
            with self.subTest(label=label):
                rows = copy.deepcopy(self.rows)
                change(rows)
                with self.assertRaises(github.GitHubTreeError):
                    github.capture(self.identity, 5.0, Opener(rows))

    def test_rejects_redirect_encoding_oversize_and_non_api_urls(self) -> None:
        for label, configure in (
            ("redirect", lambda opener: opener.urls.__setitem__(self.urls["ref"], "https://example.invalid/redirect")),
            ("encoding", lambda opener: opener.headers.__setitem__(self.urls["ref"], {"Content-Encoding": "gzip"})),
            ("oversize", lambda opener: opener.headers.__setitem__(self.urls["ref"], {"Content-Length": str(2 * 1024 * 1024 + 1)})),
        ):
            with self.subTest(label=label):
                opener = self.opener()
                configure(opener)
                with self.assertRaises(github.GitHubTreeError):
                    github.capture(self.identity, 5.0, opener)
        with self.assertRaises(github.GitHubTreeError):
            github.json_document("https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/x", 1.0, self.opener())

    def test_rejects_duplicate_json_keys(self) -> None:
        rows = copy.deepcopy(self.rows)
        rows[self.urls["ref"]] = b'{"ref":"refs/tags/a","ref":"refs/tags/b"}'
        with self.assertRaises(github.GitHubTreeError):
            github.capture(self.identity, 5.0, Opener(rows))


if __name__ == "__main__":
    unittest.main(verbosity=2)
