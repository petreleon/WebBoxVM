"""Held-parent cache publication regressions for Docs input staging."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

import vulkan_docs_input_cache as CACHE
from vulkan_docs_input_cache import cache_session
from vulkan_docs_stage_bind import PROJECT_ROOT, build_plan
from vulkan_docs_stage_model import StageError

HERE = Path(__file__).resolve().parent
BIND = HERE.parent.parent
OBSERVATION = BIND / "03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json"
ARTIFACTS = PROJECT_ROOT / ".artifacts/graphics/f02.4.4.1.5.2.3.3.1"


class InputCacheTests(unittest.TestCase):
    def test_nested_parent_swap_cannot_redirect_publication_or_rehash(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp", prefix="webboxvm-input-cache-") as temporary:
            root = Path(temporary).resolve()
            plan = build_plan(OBSERVATION, ARTIFACTS, root)
            original = CACHE.os.link

            def swap(*args, **kwargs):
                result = original(*args, **kwargs)
                safe = root / "safe"
                safe.rename(root / "moved-safe")
                safe.mkdir()
                return result

            with cache_session(plan, create=True) as cache, mock.patch.object(CACHE.os, "link", side_effect=swap):
                with self.assertRaisesRegex(StageError, "parent changed"):
                    cache.atomic("safe/member", b"x", "cache member", maximum_bytes=1)
            self.assertEqual((root / "moved-safe" / "member").read_bytes(), b"x")
            self.assertFalse((root / "safe" / "member").exists())


if __name__ == "__main__":
    unittest.main()
