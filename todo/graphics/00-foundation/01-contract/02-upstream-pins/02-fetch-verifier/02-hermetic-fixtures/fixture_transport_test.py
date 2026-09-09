#!/usr/bin/env python3
"""Hermetic F02.2.2 transport tests for the F02.2.1 contract."""

from __future__ import annotations

import hashlib
import sys
import tempfile
import unittest
from pathlib import Path
from urllib.error import HTTPError, URLError

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "01-fetch-contract"))

from source_cache import fetch_to_cache
from source_model import ContractError, ExternalCache, SourceInput, load_manifest
from fixture_transport import FakeOpener, FakeResponse, PAYLOAD, PAYLOAD_BYTES, PAYLOAD_SHA256, REVISION, entry, source


class FixtureTransportTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.cache = ExternalCache.from_path(root / "cache", root / "repository")
        self.source = source()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def assert_unaccepted(self, source: SourceInput) -> None:
        self.assertFalse(self.cache.target(source).exists())
        self.assertFalse(self.cache.root.exists())

    def test_known_local_fixture_fetches_then_rehashes_offline(self) -> None:
        self.assertEqual((len(PAYLOAD), hashlib.sha256(PAYLOAD).hexdigest()), (PAYLOAD_BYTES, PAYLOAD_SHA256))
        online = FakeOpener(FakeResponse(PAYLOAD, self.source.url))
        path, reused = fetch_to_cache(self.cache, self.source, opener=online)
        self.assertEqual(path, self.cache.target(self.source))
        self.assertEqual((path.read_bytes(), reused, online.requests), (PAYLOAD, False, [(self.source.url, 30.0)]))
        offline = FakeOpener(AssertionError("offline cache reuse must not open transport"))
        self.assertEqual(fetch_to_cache(self.cache, self.source, opener=offline), (path, True))
        self.assertEqual(offline.requests, [])

    def test_wrong_sha_leaves_no_cache_entry(self) -> None:
        bad = source(sha256="f" * 64, local_cache=f"webboxvm-graphics/f02/fixture-transport/{'f' * 64}.source")
        with self.assertRaisesRegex(ContractError, "sha256 mismatch"):
            fetch_to_cache(self.cache, bad, opener=FakeOpener(FakeResponse(PAYLOAD, bad.url)))
        self.assert_unaccepted(bad)

    def test_wrong_revision_metadata_is_rejected_before_transport(self) -> None:
        with self.assertRaisesRegex(ContractError, "pinned raw source path"):
            SourceInput.from_manifest(entry(revision="d" * 40))
        self.assertFalse(self.cache.root.exists())

    def test_unavailable_fixture_leaves_no_cache_entry(self) -> None:
        opener = FakeOpener(URLError("fixture unavailable"))
        with self.assertRaisesRegex(ContractError, "network failure: fixture unavailable"):
            fetch_to_cache(self.cache, self.source, opener=opener)
        self.assertEqual(opener.requests, [(self.source.url, 30.0)])
        self.assert_unaccepted(self.source)

    def test_malformed_manifest_leaves_no_cache_entry(self) -> None:
        malformed = Path(self.temporary.name) / "malformed.toml"
        malformed.write_text("schema = [", encoding="utf-8")
        with self.assertRaisesRegex(ContractError, "manifest cannot be read"):
            load_manifest(malformed)
        self.assertFalse(self.cache.root.exists())

    def test_redirect_http_error_is_denied_without_cache_entry(self) -> None:
        opener = FakeOpener(HTTPError(self.source.url, 302, "Found", None, None))
        with self.assertRaisesRegex(ContractError, r"redirect denied \(302\)"):
            fetch_to_cache(self.cache, self.source, opener=opener)
        self.assert_unaccepted(self.source)

    def test_changed_final_url_is_denied_without_cache_entry(self) -> None:
        response = FakeResponse(PAYLOAD, self.source.url.replace(REVISION, "d" * 40))
        with self.assertRaisesRegex(ContractError, "response URL changed"):
            fetch_to_cache(self.cache, self.source, opener=FakeOpener(response))
        self.assert_unaccepted(self.source)


if __name__ == "__main__":
    unittest.main()
