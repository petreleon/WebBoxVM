#!/usr/bin/env python3
"""Fail-closed literal declaration anchors for F03.2.2.5.4.1."""

from __future__ import annotations

import re
import subprocess

PROFILE, PAGES, PREFIX = "opengl-4.6-core", 851, "gl"
# id, physical PDF page, section, literal formal declaration
DECLARATIONS = (
    ("get-error", 38, "2.3.1", "enum GetError( void );"),
    ("get-graphics-reset-status", 41, "2.3.2", "enum GetGraphicsResetStatus( void );"),
    ("flush", 42, "2.3.3", "void Flush( void );"),
    ("finish", 43, "2.3.3", "void Finish( void );"),
    ("memory-barrier", 183, "7.13.2", "void MemoryBarrier( bitfield barriers );"),
    ("memory-barrier-by-region", 187, "7.13.2", "void MemoryBarrierByRegion( bitfield barriers );"),
)
SECTION_WITNESSES = {
    "2.3.1": (38, "2.3.1 Errors"),
    "2.3.2": (41, "2.3.2 Graphics Reset Recovery"),
    "2.3.3": (42, "2.3.3 Flush and Finish"),
    "7.13.2": (183, "7.13.2 Shader Memory Access Synchronization"),
}
DECLARATION = re.compile(
    r"^(?:enum|void) (?P<name>[A-Z][A-Za-z0-9]*)\( (?P<arguments>void|bitfield barriers) \);$"
)


class RuleError(ValueError):
    """A proposed row is not an exact, admitted source declaration."""


def reject(message: str) -> None:
    raise RuleError(message)


def normalized(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-layout", "-", "-"],
                                input=raw, capture_output=True, check=False)
    except OSError as error:
        reject(f"pdftotext is required for literal declaration anchors: {error}")
    if result.returncode:
        reject("literal declaration page extraction failed")
    try:
        return normalized(result.stdout.decode("utf-8"))
    except UnicodeDecodeError as error:
        reject(f"literal declaration page is not UTF-8: {error}")


def require_section(raw: bytes, section: str) -> int:
    witness = SECTION_WITNESSES.get(section)
    if witness is None or witness[1] not in page_text(raw, witness[0]):
        reject("declaration locator does not name its source-section heading")
    return witness[0]


def occurrences(text: str, declaration: str) -> int:
    return len(re.findall(rf"(?<![A-Za-z0-9_]){re.escape(declaration)}(?![A-Za-z0-9_])", text))


def declaration_row(raw: bytes, source_order: int, item: tuple[str, int, str, str]) -> dict[str, object]:
    identifier, page, section, declaration = item
    match = DECLARATION.fullmatch(declaration)
    if (not match or not 1 <= page <= PAGES or section not in SECTION_WITNESSES
            or match.group("name").startswith(PREFIX)):
        reject("declaration rule is malformed, prefixed, or outside the sealed PDF")
    if occurrences(page_text(raw, page), declaration) != 1:
        reject("literal declaration is absent or ambiguous on its declared PDF page")
    witness_page = require_section(raw, section)
    name = match.group("name")
    return {"id": identifier, "document_name": name, "c_name": PREFIX + name,
            "declaration": declaration, "physical_page": page, "section": section,
            "section_witness_physical_page": witness_page, "source_order": source_order,
            "source_locator": f"opengl46-core-pdf-v1:page={page};section={section}"}


def inventory(raw: bytes) -> dict[str, object]:
    ids = [item[0] for item in DECLARATIONS]
    if len(DECLARATIONS) != 6 or len(ids) != len(set(ids)):
        reject("literal declaration list is not a closed unique set")
    rows = [declaration_row(raw, order, item) for order, item in enumerate(DECLARATIONS, 1)]
    pages = [{"physical_page": row["physical_page"], "section": row["section"],
              "declaration_ids": [row["id"]],
              "section_witness_physical_page": row["section_witness_physical_page"]} for row in rows]
    return {"source_pages": pages, "declarations": rows,
            "rejections": ["registry-header-lower-profile-extension-compatibility", "guessed-template",
                           "unanchored-or-ambiguous-declaration", "ordering-visibility-state-semantics"]}
