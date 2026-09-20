#!/usr/bin/env python3
"""Focused atomic fresh-cache checks for role-aware root selectors."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import role_aware_source_cache as cache
import role_aware_source_contract as contract


class RoleAwareSourceCacheTests(unittest.TestCase):
    def test_selector_inputs_are_exactly_seven_bounded_roots(self):
        value = contract.contract()
        inputs = cache.selector_inputs(value)
        self.assertEqual([item.identifier for item in inputs], [
            "opengl-46-core-spec", "gles-32-spec", "vulkan-14-spec", "vulkan-registry",
            "opengl-cts-gl46-main", "gles-cts-main", "vulkan-cts-default"])
        self.assertTrue(all(item.byte_count <= 8 * 1024 * 1024 for item in inputs))
        self.assertEqual(len(cache.expected_files(value)), 9)
        self.assertFalse(any("api" in item.identifier for item in inputs))

    def test_preexisting_or_repository_cache_fails_before_fetch(self):
        value = contract.contract()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary, "selector-cache")
            root.mkdir(); Path(root, "old").write_bytes(b"old")
            with patch.object(cache, "fetch_to_cache") as fetch:
                with self.assertRaises(cache.SelectorCacheError):
                    cache.refresh_root_selectors(value, root, 1)
                fetch.assert_not_called()
        with self.assertRaises(cache.SelectorCacheError):
            cache.refresh_root_selectors(value, cache.repository_root(cache.HERE), 1)

    def test_composite_refresh_fetches_nine_records_before_atomic_publish(self):
        value, calls, checks = contract.contract(), [], []
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary, "selector-cache")

            def fetched(staged, source, timeout, opener):
                calls.append(source.identifier)
                path = staged.target(source)
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"staged")
                return path, False

            def checked(_value, root):
                checks.append(root)
                if len(checks) == 1:
                    self.assertFalse(destination.exists())
                return {"checked": str(root)}

            with patch.object(cache, "fetch_to_cache", side_effect=fetched), \
                    patch.object(cache, "verify_selector_cache", side_effect=checked):
                result = cache.refresh_root_selectors(value, destination, 1, opener=object())
            self.assertEqual(calls, [
                "opengl-46-core-spec", "gles-32-spec", "vulkan-14-spec", "vulkan-registry",
                "opengl-cts-gl46-main", "gles-cts-main", "vulkan-cts-default",
                "opengl-cts-4681-license", "vulkan-cts-1462-license"])
            self.assertEqual(len(checks), 2)
            self.assertNotEqual(checks[0], destination.resolve())
            self.assertTrue(destination.is_dir())
            self.assertEqual(result["checked"], str(destination.resolve()))

    def test_timeout_and_symlink_target_fail_closed(self):
        value = contract.contract()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary, "target")
            with self.assertRaises(cache.SelectorCacheError):
                cache.refresh_root_selectors(value, root, 0)
            root.mkdir()
            link = Path(temporary, "link")
            link.symlink_to(root, target_is_directory=True)
            with self.assertRaises(cache.SelectorCacheError):
                cache.refresh_root_selectors(value, link, 1)


if __name__ == "__main__":
    unittest.main()
