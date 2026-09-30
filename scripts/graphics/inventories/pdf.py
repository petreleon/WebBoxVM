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


def _extract(raw: bytes, first: int, last: int) -> str:
    if not isinstance(raw, bytes) or not raw:
        raise PdfError("PDF input must be nonempty bytes")
    if type(first) is not int or type(last) is not int or first < 1 or last < first:
        raise PdfError("physical PDF page range must be positive and ordered")
    try:
        result = subprocess.run(
            [extractor(), "-f", str(first), "-l", str(last), "-raw", "-", "-"],
            input=raw,
            capture_output=True,
            check=False,
            timeout=30,
            env={"LC_ALL": "C", "PATH": "/usr/bin:/bin"},
        )
        if result.returncode != 0:
            raise PdfError(f"PDF text extraction failed with exit {result.returncode}")
        return result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError, subprocess.TimeoutExpired) as error:
        raise PdfError(f"cannot read sealed PDF page: {error}") from error


def sealed_pdf_page(raw: bytes, page: int) -> str:
    return compact(_extract(raw, page, page))


def sealed_pdf_pages(raw: bytes, first: int, last: int) -> dict[int, str]:
    """Preserve physical-page breaks and headings for finite declaration windows."""
    pages = _extract(raw, first, last).split("\f")
    if pages[-1].strip() == "":
        pages.pop()
    if len(pages) != last - first + 1 or any(not page.strip() for page in pages):
        raise PdfError("PDF page range is empty, truncated, or lacks physical breaks")
    return {first + offset: page for offset, page in enumerate(pages)}
