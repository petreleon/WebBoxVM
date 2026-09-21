#!/usr/bin/env python3
"""Exact physical-page anchor checks for F03.2.3.4.1.2."""

from __future__ import annotations

import re


def section(text: str, numeric: str) -> bool:
    return bool(re.search(rf"(?m)^{re.escape(numeric)}\.\s+[A-Z]", text))


def table(text: str, identifier: str) -> bool:
    return bool(re.search(rf"(?<![A-Za-z0-9])Table\s*{re.escape(identifier)}:(?![A-Za-z0-9])", text))


def anchored(rows: list[dict[str, object]], raw: bytes, pages: int, source, reject) -> None:
    previous = (0, "")
    for row in rows:
        page, identifier = row.get("physical_page"), str(row.get("candidate_id", ""))
        numeric, kind = row.get("numeric_section"), row.get("anchor_kind")
        if (not isinstance(page, int) or not 1 <= page <= pages or not isinstance(numeric, str)
                or (page, identifier) <= previous):
            reject("candidate has an invalid physical source order")
        previous = (page, identifier)
        text = source.page_text(raw, page)
        if not section(text, numeric):
            reject("sealed PDF has a missing or non-exact page-section anchor")
        if kind == "table-caption":
            table_id = row.get("table")
            if not isinstance(table_id, str) or not table(text, table_id):
                reject("sealed PDF has a missing or non-exact table-caption anchor")
        elif kind != "section-heading":
            reject("candidate has an unknown anchor kind")
