#!/usr/bin/env python3
"""Hostile notice checks for F02.5.2.1 source terms and attribution."""

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from normative_notices import NoticeError, verify_notices

PDF_TEXT = "Copyright © 2006-2022 The Khronos Group Inc. All Rights Reserved. Khronos grants a conditional copyright license"
VKSPEC = "Copyright 2014-2026 The Khronos Group Inc.\nSPDX-License-Identifier: CC-BY-4.0\n"
VKXML = "Copyright 2015-2026 The Khronos Group Inc.\nSPDX-License-Identifier: Apache-2.0 OR MIT\n"


class NoticeTests(unittest.TestCase):
    def paths(self, root: Path) -> dict[str, Path]:
        result = {identifier: root / identifier for identifier in ("opengl-46-core-spec", "gles-32-spec",
                  "vulkan-14-spec", "vulkan-registry")}
        result["opengl-46-core-spec"].write_bytes(b"pdf")
        result["gles-32-spec"].write_bytes(b"pdf")
        result["vulkan-14-spec"].write_text(VKSPEC, encoding="utf-8")
        result["vulkan-registry"].write_text(VKXML, encoding="utf-8")
        return result

    def test_verified_notices_cover_all_roots(self):
        with tempfile.TemporaryDirectory() as temporary, patch("normative_notices.pdf_text", return_value=PDF_TEXT) as extract:
            verify_notices(self.paths(Path(temporary)))
        self.assertEqual([call.args[1] for call in extract.call_args_list], [3, 2])

    def test_rejects_missing_coverage_or_a_changed_notice(self):
        with tempfile.TemporaryDirectory() as temporary, patch("normative_notices.pdf_text", return_value=PDF_TEXT):
            paths = self.paths(Path(temporary))
            missing = dict(paths); missing.pop("vulkan-registry")
            with self.assertRaises(NoticeError):
                verify_notices(missing)
            paths["vulkan-14-spec"].write_text("SPDX-License-Identifier: MIT\n", encoding="utf-8")
            with self.assertRaises(NoticeError):
                verify_notices(paths)

    def test_rejects_a_pdf_without_the_declared_terms(self):
        with tempfile.TemporaryDirectory() as temporary, patch("normative_notices.pdf_text", return_value="wrong"):
            with self.assertRaises(NoticeError):
                verify_notices(self.paths(Path(temporary)))


if __name__ == "__main__":
    unittest.main()
