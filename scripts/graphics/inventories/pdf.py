"""Shared Poppler extraction for source-bound graphics inventory catalogs.

Source identity and page/section fences remain the caller's responsibility.
"""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

TRUSTED_EXTRACTORS = (
    Path("/opt/homebrew/bin/pdftotext"),
    Path("/usr/local/bin/pdftotext"),
    Path("/usr/bin/pdftotext"),
)


class PdfError(ValueError):
    """A physical PDF page could not be extracted by the fixed local tool."""


def compact(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def extractor() -> str:
    for candidate in TRUSTED_EXTRACTORS:
        resolved = candidate.resolve()
        if resolved.is_file() and os.access(resolved, os.X_OK):
            return str(resolved)
    raise PdfError("fixed PDF text extractor is unavailable")


def sealed_pdf_page(raw: bytes, page: int) -> str:
    if not isinstance(raw, bytes) or not raw:
        raise PdfError("PDF input must be nonempty bytes")
    if type(page) is not int or page < 1:
        raise PdfError("physical PDF page must be a positive integer")
    try:
        result = subprocess.run(
            [extractor(), "-f", str(page), "-l", str(page), "-raw", "-", "-"],
            input=raw,
            capture_output=True,
            check=False,
            timeout=30,
            env={"LC_ALL": "C", "PATH": "/usr/bin:/bin"},
        )
        if result.returncode != 0:
            raise PdfError(f"PDF text extraction failed with exit {result.returncode}")
        return compact(result.stdout.decode("utf-8"))
    except (OSError, UnicodeDecodeError, subprocess.TimeoutExpired) as error:
        raise PdfError(f"cannot read sealed PDF page: {error}") from error
