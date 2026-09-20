#!/usr/bin/env python3
"""Hermetic fresh-capture, cache-replay, and no-claim receipt tests."""

import copy
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import vulkan_full_suite_receipt as full
import vulkan_full_suite_contract as contract


def checked(observed):
    ledger = observed["ledger"]
    return {"claims": dict(full.ledger_map.NO_CLAIMS), "cts_executions": 0, "source_root": observed["source_root"],
            "states": observed["states"], "ledger_sha256": ledger["ledger_sha256"], "member_count": ledger["member_count"],
            "member_total_bytes": ledger["member_total_bytes"], "receipt_sha256": "c" * 64}


def network(observed):
    ledger, root = observed["ledger"], observed["source_root"]
    streams = ledger["member_count"] + 1
    return {"root_streams": 1, "member_streams": ledger["member_count"], "total_streams": streams,
            "raw_https_requests": streams, "sha256_verified_streams": streams,
            "git_blob_sha1_verified_streams": streams, "total_bytes": root["bytes"] + ledger["member_total_bytes"]}


class Local:
    def __enter__(self): return self
    def __exit__(self, *_): return False
    def assert_complete(self): self.complete = True


class Captured:
    def __init__(self, total, local=None): self.total_bytes, self.discarded, self.local = total, False, local or Local()
    def replay(self): return self.local
    def discard(self): self.discarded = True


class FreshFullSuiteTests(unittest.TestCase):
    def setUp(self):
        self.observed, self.root, self.tree = full.source()
        self.checked, self.network = checked(self.observed), network(self.observed)
        self.need = full.required_space(self.tree)
        self.disk = {"required_free_bytes": self.need, "available_free_bytes": self.need}

    def test_binds_exact_99_streams_oversize_and_no_claims(self):
        value = full.build_from(self.observed, self.checked, self.network, self.disk)
        self.assertEqual((value["capture"]["network"]["raw_https_requests"], value["ledger"]["member_count"],
                          value["capture"]["network"]["total_bytes"]), (99, 98, 434672695))
        self.assertEqual((value["oversize_members"]["count"], value["oversize_members"]["largest_bytes"],
                          value["oversize_members"]["members"][0]["path"]), (14, 61932251, "external/vulkancts/mustpass/main/vk-default/api.txt"))
        self.assertEqual(value["capture"]["failure_or_skip"], {"status": "none"})
        self.assertTrue(all(flag is False for flag in value["claims"].values()))
        self.assertEqual((value["cts_executions"], value["states"]["satisfies_vulkan_14_core_manifest"]), (0, False))
        full.validate_from(value, self.observed, self.checked, self.network, self.disk)

    def test_rejects_forged_positive_or_short_receipts(self):
        value = full.build_from(self.observed, self.checked, self.network, self.disk)
        forged = copy.deepcopy(value); forged["claims"]["conformance"] = True
        with self.assertRaises(full.FullReceiptError): full.validate_from(forged, self.observed, self.checked, self.network, self.disk)
        short = copy.deepcopy(self.network); short["raw_https_requests"] = 98
        with self.assertRaises(full.FullReceiptError): full.build_from(self.observed, self.checked, short, self.disk)
        malformed = copy.deepcopy(value); malformed["cts_executions"] = 1
        with self.assertRaises(full.FullReceiptError): full.validate_from(malformed, self.observed, self.checked, self.network, self.disk)
        for change in (lambda item: item["capture"]["cache"].__setitem__("fresh", 0),
                       lambda item: item["oversize_members"]["members"].pop(),
                       lambda item: item["oversize_members"]["members"].reverse(),
                       lambda item: item["source_root"].__setitem__("sha256", "0" * 64),
                       lambda item: item.__setitem__("receipt_sha256", "0" * 64)):
            malformed = copy.deepcopy(value); change(malformed)
            with self.assertRaises(full.FullReceiptError): full.validate_from(malformed, self.observed, self.checked, self.network, self.disk)

    def test_counter_requires_each_raw_url_once_in_order(self):
        rows = ("https://one", "https://two")
        delegate = SimpleNamespace(open=lambda request, timeout: request.full_url)
        counter = full.CountingOpener(rows, delegate)
        self.assertEqual(counter.open(SimpleNamespace(full_url=rows[0]), 1), rows[0])
        with self.assertRaises(full.FullReceiptError): counter.open(SimpleNamespace(full_url=rows[0]), 1)
        with self.assertRaises(full.FullReceiptError): counter.complete()

    def test_preflight_rejects_nonempty_symlink_and_low_space_before_network(self):
        with tempfile.TemporaryDirectory(dir="/private/tmp") as temporary:
            root = Path(temporary, "cache")
            repository = full.stage.cache.repository_root(full.HERE)
            self.assertGreaterEqual(full.preflight(root, repository, 1), 1)
            Path(root, "unexpected").write_text("x", encoding="utf-8")
            with self.assertRaises(full.FullReceiptError): full.preflight(root, repository, 1)
            link, target = Path(temporary, "link"), Path(temporary, "target")
            target.mkdir(); link.symlink_to(target, target_is_directory=True)
            with self.assertRaises(full.FullReceiptError): full.preflight(link, repository, 1)
        with (tempfile.TemporaryDirectory(dir="/private/tmp") as temporary,
              patch.object(full.shutil, "disk_usage", return_value=SimpleNamespace(free=0)),
              patch.object(full.stage, "capture") as live):
            with self.assertRaises(full.FullReceiptError): full.capture(Path(temporary, "cache"))
            live.assert_not_called()

    def test_capture_uses_99_ordered_raw_urls_then_offline_local_replay(self):
        cache_root, local = Path("/private/tmp/fresh-cache"), Local()
        staged = Captured(self.network["total_bytes"], local)
        raw, seen = SimpleNamespace(urls=[]), {}
        raw.open = lambda request, timeout: raw.urls.append(request.full_url)
        def live(*args):
            counter = args[-1]
            for url in counter.urls: counter.open(SimpleNamespace(full_url=url), 1)
            return staged
        def populate(*args):
            seen["opener"] = args[-1]
            return SimpleNamespace(reused=False)
        with patch.object(full, "preflight", return_value=self.need), patch.object(contract, "build_opener", return_value=raw), \
             patch.object(full.stage, "capture", side_effect=live), \
             patch.object(full.stage.cache, "populate", side_effect=populate), patch.object(full.replay, "replay", return_value=self.checked):
            self.assertEqual(full.capture(cache_root), full.build_from(self.observed, self.checked, self.network, self.disk))
        self.assertEqual(tuple(raw.urls), full.planned_urls(self.root, self.tree)); self.assertIs(local, seen["opener"])
        self.assertTrue(staged.discarded)

    def test_capture_rejects_reuse_and_discards_stage(self):
        cache_root = Path("/private/tmp/fresh-cache")
        counter = SimpleNamespace(index=99, complete=lambda: None)
        for result in (SimpleNamespace(reused=True), full.stage.StageError("injected")):
            staged = Captured(self.network["total_bytes"])
            option = {"side_effect": result} if isinstance(result, BaseException) else {"return_value": result}
            with patch.object(full, "preflight", return_value=self.need), patch.object(full, "CountingOpener", return_value=counter), \
                 patch.object(full.stage, "capture", return_value=staged), patch.object(full.stage.cache, "populate", **option):
                with self.assertRaises(full.FullReceiptError): full.capture(cache_root)
            self.assertTrue(staged.discarded)


if __name__ == "__main__":
    unittest.main()
