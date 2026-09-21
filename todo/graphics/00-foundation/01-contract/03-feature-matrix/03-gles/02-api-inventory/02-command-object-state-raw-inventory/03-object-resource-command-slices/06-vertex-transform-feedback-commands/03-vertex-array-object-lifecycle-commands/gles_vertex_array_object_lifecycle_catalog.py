"""Bounded GLES vertex-array-object lifecycle declarations for F03.3.2.2.3.6.3."""

from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
from pathlib import Path

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "vertex-transform-feedback-commands"
FAMILY = ("object-declarations", "vertex-array", ("10.2", "10.3.1-10.3.8", "10.4"), 22,
          294, "10.4", "void GenVertexArrays( sizei n, uint *arrays );")
PRIMARY_PAGE, PRIMARY_SECTION = 294, "10.4"
SOURCE_PAGES = (294, 295)
SECTION_WITNESSES = {"10.4": (294, "10.4 Vertex Array Objects")}
SECTION_PAGES = {"10.4": (294, 295)}
SECTION_BOUNDARIES = {(295, "10.4"): ("before", "10.5 Drawing Commands Using Vertex Arrays")}
DECLARATIONS = (
    (294, "10.4", "GenVertexArrays", "void GenVertexArrays( sizei n, uint *arrays );"),
    (294, "10.4", "DeleteVertexArrays", "void DeleteVertexArrays( sizei n, const uint *arrays );"),
    (295, "10.4", "BindVertexArray", "void BindVertexArray( uint array );"),
    (295, "10.4", "IsVertexArray", "boolean IsVertexArray( uint array );"),
)
SEALED_DECLARATIONS_SHA256 = "c7a5dd7a2a1d98a17c603ef3f2d35396d943548890cd51c041b5f39ddab43ac1"
TRUSTED_EXTRACTORS = (Path("/opt/homebrew/bin/pdftotext"), Path("/usr/local/bin/pdftotext"), Path("/usr/bin/pdftotext"))
SOURCE_PROTOTYPE = re.compile(r"(?:void|boolean) [A-Z][A-Za-z0-9]*\([^;]*\);")
PROTOTYPE = re.compile(r"(?:void|boolean) ([A-Z][A-Za-z0-9]*)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A declaration lies outside the sealed vertex-array-object lifecycle slice."""


def reject(message: str) -> None: raise CatalogError(message)


def compact(value: str) -> str: return " ".join(value.replace("\f", " ").split())


def sha256(value: object) -> str:
    return hashlib.sha256(json.dumps(value, separators=(",", ":"), ensure_ascii=True).encode("utf-8")).hexdigest()


def extractor() -> str:
    for candidate in TRUSTED_EXTRACTORS:
        resolved = candidate.resolve()
        if resolved.is_file() and os.access(resolved, os.X_OK): return str(resolved)
    reject("fixed PDF text extractor is unavailable")


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run([extractor(), "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw, capture_output=True, check=False,
                                env={"LC_ALL": "C", "PATH": "/usr/bin:/bin"})
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error: reject(f"cannot read sealed vertex-array-object page: {error}")
    if result.returncode != 0: reject("cannot read sealed vertex-array-object page")
    return compact(value)


def source_window(page: int, value: str) -> str:
    boundary = SECTION_BOUNDARIES.get((page, "10.4"))
    if boundary is None: return value
    relation, marker = boundary; split = value.find(marker)
    if split < 0 or relation != "before": reject("vertex-array-object source fence is missing or malformed")
    return value[:split]


def source_slice(text) -> tuple[tuple[int, str], ...]:
    return tuple((page, item) for page in SOURCE_PAGES for item in SOURCE_PROTOTYPE.findall(source_window(page, text(page))))


def in_section(page: int, section: str, declaration: str, value: str) -> bool:
    boundary = SECTION_BOUNDARIES.get((page, section))
    if boundary is None: return True
    relation, marker = boundary; start, split = value.find(declaration), value.find(marker)
    return relation == "before" and start >= 0 and split >= 0 and start < split


def bound_family(chunk_paths: dict[str, object], document) -> int:
    chunk, identifier, scopes, order, page, section, anchor = FAMILY
    rows = [row for row in document(chunk_paths[chunk]).get("families", []) if row.get("id") == identifier]
    expected = {"id": identifier, "family_kind": "declaration", "source_scope": list(scopes), "route": ROUTE,
                "reason": "formal-declaration-only", "source_order": order,
                "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": anchor}}
    if len(rows) != 1 or rows[0] != expected: reject("vertex-array command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def facts(raw: bytes, pages: int, family_order: int, normalize) -> list[list[object]]:
    fixed_witnesses = {"10.4": (294, "10.4 Vertex Array Objects")}
    fixed_pages = {"10.4": (294, 295)}
    fixed_boundaries = {(295, "10.4"): ("before", "10.5 Drawing Commands Using Vertex Arrays")}
    if (PROFILE != "gles-3.2" or pages != PAGES or family_order != 22 or PRIMARY_PAGE != 294 or PRIMARY_SECTION != "10.4"
            or SOURCE_PAGES != (294, 295) or SECTION_WITNESSES != fixed_witnesses or SECTION_PAGES != fixed_pages
            or SECTION_BOUNDARIES != fixed_boundaries or sha256(DECLARATIONS) != SEALED_DECLARATIONS_SHA256):
        reject("GLES vertex-array-object source boundary, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str: texts.setdefault(page, page_text(raw, page)); return texts[page]
    expected = tuple((page, declaration) for page, _, _, declaration in DECLARATIONS)
    if source_slice(text) != expected: reject("vertex-array-object source window is incomplete, rerouted, or out of source order")
    result, names = [], set()
    for page, section, name, declaration in DECLARATIONS:
        witness, match, value = SECTION_WITNESSES.get(section), PROTOTYPE.fullmatch(declaration), text(page)
        if (witness is None or page not in SECTION_PAGES.get(section, ()) or witness[1] not in text(witness[0]) or declaration not in value
                or not in_section(page, section, declaration, value) or match is None or match.group(1) != name):
            reject("vertex-array-object declaration is malformed or outside this source slice")
        try: c_names = normalize([("literal", name)])
        except Exception as error: reject(f"declaration grammar rejected a vertex-array-object literal: {error}")
        if c_names != ["gl" + name] or name in names: reject("vertex-array-object literal is duplicate, prefixed, or cross-family")
        names.add(name); result.append([f"gles32-vertex-array-object-{len(result) + 1:02d}", name, c_names[0], declaration, page, section, len(result) + 1])
    if len(result) != 4 or [row[-1] for row in result] != list(range(1, 5)): reject("vertex-array-object raw ordering is incomplete or unstable")
    return result
