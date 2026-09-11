#!/usr/bin/env python3
"""Verify licence and attribution notices inside F02.5.2.1 payloads."""

from __future__ import annotations

import subprocess
from pathlib import Path

PDF_PHRASES = ("Copyright © 2006-2022 The Khronos Group Inc. All Rights Reserved.",
               "Khronos grants a conditional copyright license")
NOTICES = {
    "opengl-46-core-spec": {"kind": "pdf", "page": 3, "phrases": PDF_PHRASES},
    "gles-32-spec": {"kind": "pdf", "page": 2, "phrases": PDF_PHRASES},
    "vulkan-14-spec": {"kind": "text", "phrases":
        ("Copyright 2014-2026 The Khronos Group Inc.", "SPDX-License-Identifier: CC-BY-4.0")},
    "vulkan-registry": {"kind": "text", "phrases":
        ("Copyright 2015-2026 The Khronos Group Inc.", "SPDX-License-Identifier: Apache-2.0 OR MIT")},
}


class NoticeError(ValueError):
    """A verified source payload lacks its declared terms or attribution."""


def pdf_text(path: Path, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), str(path), "-"],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, check=False)
    except OSError as error:
        raise NoticeError(f"cannot extract PDF notice: {error}") from error
    if result.returncode != 0 or not isinstance(result.stdout, str):
        raise NoticeError("PDF notice extraction failed")
    return result.stdout


def text_for(identifier: str, path: Path, notice: dict[str, object]) -> str:
    if not path.is_file():
        raise NoticeError(f"{identifier} notice source is not a regular file")
    if notice["kind"] == "pdf":
        return pdf_text(path, notice["page"])
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError as error:
        raise NoticeError(f"{identifier} notice source is not UTF-8") from error


def verify_notices(paths: dict[str, Path]) -> None:
    if set(paths) != set(NOTICES):
        raise NoticeError("notice verification has incomplete source coverage")
    for identifier, notice in NOTICES.items():
        content = text_for(identifier, paths[identifier], notice)
        if any(phrase not in content for phrase in notice["phrases"]):
            raise NoticeError(f"{identifier} payload lacks its declared licence or attribution notice")
