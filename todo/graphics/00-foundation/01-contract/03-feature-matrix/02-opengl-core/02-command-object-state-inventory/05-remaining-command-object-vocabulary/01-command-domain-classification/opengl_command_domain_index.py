"""Read the sealed OpenGL index only to cross-check omitted source families."""

from __future__ import annotations

import subprocess


def verify(raw: bytes, rules, crosschecks, reject) -> dict[str, object]:
    pages = tuple(rules.INDEX_PHYSICAL_PAGES)
    if pages != tuple(range(801, 852)):
        reject("index omission crosscheck must cover physical PDF pages 801 through 851")
    try:
        result = subprocess.run(["pdftotext", "-f", "801", "-l", "851", "-layout", "-", "-"],
                                input=raw, capture_output=True, check=False)
        readable = [chunk for chunk in result.stdout.decode("utf-8").split("\f") if chunk.strip()]
    except (OSError, UnicodeDecodeError) as error:
        reject(f"physical PDF index crosscheck cannot be read: {error}")
    if result.returncode or len(readable) != len(pages):
        reject("physical PDF index crosscheck is incomplete or unreadable")
    witnesses = crosschecks.index_witnesses(readable, pages, rules, reject)
    return {"role": "omission-crosscheck-only", "physical_page_range": [pages[0], pages[-1]],
            "checked_page_count": len(pages), "normalized_index_witnesses": witnesses,
            "excluded_source_families": list(rules.INDEX_ONLY_EXCLUSIONS)}
