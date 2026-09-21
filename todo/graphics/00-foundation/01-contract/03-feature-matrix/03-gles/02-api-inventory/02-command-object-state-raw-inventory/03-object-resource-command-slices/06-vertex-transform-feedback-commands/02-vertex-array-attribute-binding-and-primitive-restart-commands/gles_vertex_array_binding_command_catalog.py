"""Bounded GLES vertex-array binding declarations for F03.3.2.2.3.6.2."""

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
PRIMARY_PAGE, PRIMARY_SECTION = 285, "10.3.1"
SOURCE_PAGES = (285, 286, 287, 288, 289, 290, 291)
SECTION_WITNESSES = {
    "10.3.1": (285, "10.3.1 Specifying Arrays for Generic Vertex Attributes"),
    "10.3.2": (289, "10.3.2 Vertex Attribute Divisors"),
    "10.3.4": (291, "10.3.4 Primitive Restart"),
}
SECTION_PAGES = {"10.3.1": (285, 286, 287, 288, 289), "10.3.2": (289, 290), "10.3.4": (291,)}
SECTION_BOUNDARIES = {(289, "10.3.1"): ("before", "10.3.2 Vertex Attribute Divisors"),
                      (289, "10.3.2"): ("after", "10.3.2 Vertex Attribute Divisors"),
                      (290, "10.3.2"): ("before", "10.3.3 Transferring Array Elements"),
                      (291, "10.3.4"): ("before", "10.3.5 Robust Buffer Access")}
PRIMITIVE_RESTART_WITNESS = "with target PRIMITIVE_RESTART_FIXED_INDEX."
DECLARATIONS = (
    (285, "10.3.1", "VertexAttribFormat", "void VertexAttribFormat( uint attribindex, int size, enum type, boolean normalized, uint relativeoffset );"),
    (285, "10.3.1", "VertexAttribIFormat", "void VertexAttribIFormat( uint attribindex, int size, enum type, uint relativeoffset );"),
    (286, "10.3.1", "BindVertexBuffer", "void BindVertexBuffer( uint bindingindex, uint buffer, intptr offset, sizei stride );"),
    (287, "10.3.1", "VertexAttribBinding", "void VertexAttribBinding( uint attribindex, uint bindingindex );"),
    (288, "10.3.1", "VertexAttribPointer", "void VertexAttribPointer( uint index, int size, enum type, boolean normalized, sizei stride, const void *pointer );"),
    (288, "10.3.1", "VertexAttribIPointer", "void VertexAttribIPointer( uint index, int size, enum type, sizei stride, const void *pointer );"),
    (289, "10.3.1", "EnableVertexAttribArray", "void EnableVertexAttribArray( uint index );"),
    (289, "10.3.1", "DisableVertexAttribArray", "void DisableVertexAttribArray( uint index );"),
    (290, "10.3.2", "VertexBindingDivisor", "void VertexBindingDivisor( uint bindingindex, uint divisor );"),
    (290, "10.3.2", "VertexAttribDivisor", "void VertexAttribDivisor( uint index, uint divisor );"),
    (291, "10.3.4", "Enable", "void Enable( enum target );"),
    (291, "10.3.4", "Disable", "void Disable( enum target );"),
)
SEALED_DECLARATIONS_SHA256 = "ef7e7898973d1e00021ccbf11e8a24392d4eb030473b2646b6158303a1d6c067"
TRUSTED_EXTRACTORS = (Path("/opt/homebrew/bin/pdftotext"), Path("/usr/local/bin/pdftotext"), Path("/usr/bin/pdftotext"))
SOURCE_PROTOTYPE = re.compile(r"void [A-Z][A-Za-z0-9]*\([^;]*\);")
PROTOTYPE = re.compile(r"void ([A-Z][A-Za-z0-9]*)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A declaration lies outside the sealed vertex-array binding slice."""


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
    except (OSError, UnicodeDecodeError) as error: reject(f"cannot read sealed vertex-array binding page: {error}")
    if result.returncode != 0: reject("cannot read sealed vertex-array binding page")
    return compact(value)


def source_slice(text) -> tuple[tuple[int, str], ...]:
    return tuple((page, item) for page in SOURCE_PAGES for item in SOURCE_PROTOTYPE.findall(text(page)))


def in_section(page: int, section: str, declaration: str, value: str) -> bool:
    boundary = SECTION_BOUNDARIES.get((page, section))
    if boundary is None: return True
    relation, marker = boundary; start, split = value.find(declaration), value.find(marker)
    return start >= 0 and split >= 0 and ((relation == "before" and start < split) or (relation == "after" and start > split))


def bound_family(chunk_paths: dict[str, object], document) -> int:
    chunk, identifier, scopes, order, page, section, anchor = FAMILY
    rows = [row for row in document(chunk_paths[chunk]).get("families", []) if row.get("id") == identifier]
    expected = {"id": identifier, "family_kind": "declaration", "source_scope": list(scopes), "route": ROUTE,
                "reason": "formal-declaration-only", "source_order": order,
                "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": anchor}}
    if len(rows) != 1 or rows[0] != expected: reject("vertex-array command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def facts(raw: bytes, pages: int, family_order: int, normalize) -> list[list[object]]:
    fixed_witnesses = {"10.3.1": (285, "10.3.1 Specifying Arrays for Generic Vertex Attributes"),
                       "10.3.2": (289, "10.3.2 Vertex Attribute Divisors"), "10.3.4": (291, "10.3.4 Primitive Restart")}
    fixed_pages = {"10.3.1": (285, 286, 287, 288, 289), "10.3.2": (289, 290), "10.3.4": (291,)}
    fixed_boundaries = {(289, "10.3.1"): ("before", "10.3.2 Vertex Attribute Divisors"),
                        (289, "10.3.2"): ("after", "10.3.2 Vertex Attribute Divisors"),
                        (290, "10.3.2"): ("before", "10.3.3 Transferring Array Elements"),
                        (291, "10.3.4"): ("before", "10.3.5 Robust Buffer Access")}
    if (PROFILE != "gles-3.2" or pages != PAGES or family_order != 22 or PRIMARY_PAGE != 285 or PRIMARY_SECTION != "10.3.1"
            or SOURCE_PAGES != (285, 286, 287, 288, 289, 290, 291) or SECTION_WITNESSES != fixed_witnesses or SECTION_PAGES != fixed_pages
            or SECTION_BOUNDARIES != fixed_boundaries or PRIMITIVE_RESTART_WITNESS != "with target PRIMITIVE_RESTART_FIXED_INDEX."
            or sha256(DECLARATIONS) != SEALED_DECLARATIONS_SHA256): reject("GLES vertex-array source boundary, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str: texts.setdefault(page, page_text(raw, page)); return texts[page]
    expected = tuple((page, declaration) for page, _, _, declaration in DECLARATIONS)
    if source_slice(text) != expected or PRIMITIVE_RESTART_WITNESS not in text(291):
        reject("vertex-array binding source window is incomplete, rerouted, or out of source order")
    result, names = [], set()
    for page, section, name, declaration in DECLARATIONS:
        witness, match, value = SECTION_WITNESSES.get(section), PROTOTYPE.fullmatch(declaration), text(page)
        if (witness is None or page not in SECTION_PAGES.get(section, ()) or witness[1] not in text(witness[0]) or declaration not in value
                or not in_section(page, section, declaration, value) or match is None or match.group(1) != name):
            reject("vertex-array binding declaration is malformed or outside this source slice")
        try: c_names = normalize([("literal", name)])
        except Exception as error: reject(f"declaration grammar rejected a vertex-array literal: {error}")
        if c_names != ["gl" + name] or name in names: reject("vertex-array literal is duplicate, prefixed, or cross-family")
        names.add(name); result.append([f"gles32-vertex-array-binding-{len(result) + 1:02d}", name, c_names[0], declaration, page, section, len(result) + 1])
    if len(result) != 12 or [row[-1] for row in result] != list(range(1, 13)): reject("vertex-array binding raw ordering is incomplete or unstable")
    return result
