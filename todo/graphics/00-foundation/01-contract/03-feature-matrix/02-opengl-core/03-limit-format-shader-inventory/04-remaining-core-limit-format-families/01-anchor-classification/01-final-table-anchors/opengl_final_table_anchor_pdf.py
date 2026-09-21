#!/usr/bin/env python3
"""Exact raw-PDF table-row witnesses for F03.2.3.4.1.1."""

from __future__ import annotations

import re

TYPE_MARKER = r"(?:[BERSY](?=\s|$)|Z(?:[+2])?(?=\s|$)|n\s+×|3\s+×)"
COMMAND = re.compile(r"(?<![A-Za-z0-9_])(?:Get[A-Za-z0-9]+(?: v)?|IsEnabled)(?![A-Za-z0-9_])")


def body(table: str, text: str, reject) -> str:
    captions = list(re.finditer(rf"^Table {re.escape(table)}:", text, re.MULTILINE))
    if len(captions) != 1:
        reject("sealed PDF has a missing or ambiguous table caption")
    value = text[:captions[0].start()]
    if not all(token in value for token in ("Get value", "Command", "Sec.")):
        reject("sealed PDF has a missing table-column anchor")
    return value


def value_matches(rows: list[dict[str, object]], text: str, reject):
    values = [row for row in rows if row["table_column"] == "Get value"]
    labels = sorted({str(row["cell_text"]) for row in values}, key=lambda item: (-len(item), item))
    pattern = re.compile(rf"^(?P<cell>{'|'.join(map(re.escape, labels))})(?=\s+{TYPE_MARKER})", re.MULTILINE)
    matches = list(pattern.finditer(text))
    if [match.group("cell") for match in matches] != [row["cell_text"] for row in values]:
        reject("sealed PDF has a missing, ambiguous, reordered, or non-exact source-cell vector")
    return values, matches


def section_at_end(block: str, section: str) -> bool:
    return bool(re.search(rf"(?<![0-9.]){re.escape(section)}(?![0-9.])\s*\Z", block))


def value_blocks(rows: list[dict[str, object]], text: str, reject) -> None:
    values, matches = value_matches(rows, text, reject)
    ends = [match.start() for match in matches[1:]] + [len(text)]
    for row, match, end in zip(values, matches, ends):
        block, command, section = text[match.start():end], row["get_command"], row["numeric_section"]
        if COMMAND.findall(block) != [command] or not section_at_end(block, str(section)):
            reject("sealed PDF source cell lacks its exact command or numeric-section anchor")


def description_rows(rows: list[dict[str, object]], text: str, reject) -> None:
    expected = [row for row in rows if row["table_column"] == "Description"]
    witnesses = []
    for row in expected:
        cell, section = str(row["cell_text"]), str(row["numeric_section"])
        pattern = re.compile(rf"^(?P<line>.*{re.escape(cell)}\s+{re.escape(section)})\s*$", re.MULTILINE)
        found = list(pattern.finditer(text))
        if len(found) != 1:
            reject("sealed PDF has a missing or ambiguous description-cell anchor")
        witnesses.append(found[0])
    if [match.start() for match in witnesses] != sorted(match.start() for match in witnesses):
        reject("sealed PDF has reordered description-cell anchors")
    for row, match in zip(expected, witnesses):
        line, command = match.group("line"), row["get_command"]
        if command == "–":
            valid = bool(re.match(r"^–\s+.+\s+–\s+.+$", line))
        else:
            valid = COMMAND.findall(line) == [command]
        if not valid:
            reject("sealed PDF description cell lacks its exact command anchor")


def anchored(rows: list[dict[str, object]], raw: bytes, pages: int, source, reject) -> None:
    for table in dict.fromkeys(row["table"] for row in rows):
        table_rows = [row for row in rows if row["table"] == table]
        page = table_rows[0]["physical_page"]
        if not isinstance(page, int) or not 1 <= page <= pages:
            reject("catalog physical page is outside the sealed PDF")
        text = body(str(table), source.normalized_page(raw, page), reject)
        value_blocks(table_rows, text, reject)
        description_rows(table_rows, text, reject)
