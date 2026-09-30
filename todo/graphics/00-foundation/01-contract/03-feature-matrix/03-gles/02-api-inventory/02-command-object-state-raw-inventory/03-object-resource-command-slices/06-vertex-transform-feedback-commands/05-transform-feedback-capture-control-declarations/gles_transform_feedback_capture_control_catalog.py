"""Pinned p.357 capture-control literals for F03.3.2.2.3.6.5."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import re
from pathlib import Path

PDF_PATH = Path(__file__).resolve().parents[11] / "scripts/graphics/inventories/pdf.py"
if PDF_PATH.is_symlink() or not PDF_PATH.is_file():
    raise ValueError("shared PDF extractor must be a regular file")
PDF_SPEC = importlib.util.spec_from_file_location("webboxvm_inventory_pdf", PDF_PATH)
if PDF_SPEC is None or PDF_SPEC.loader is None:
    raise ValueError("cannot load shared PDF extractor")
PDF = importlib.util.module_from_spec(PDF_SPEC)
PDF_SPEC.loader.exec_module(PDF)

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "vertex-transform-feedback-commands"
FAMILY = ("object-declarations", "transform-feedback", ("12.2",), 26, 357, "12.2",
          "void BeginTransformFeedback( enum primitiveMode );")
PRIMARY_PAGE, PRIMARY_SECTION = 357, "12.2.2"
SOURCE_PAGES = (357,)
SECTION_START = (357, "12.2.2 Transform Feedback Primitive Capture")
SECTION_BOUNDARY = (357, "OpenGL ES 3.2 (May 5, 2022)")
DECLARATIONS = (
    (357, "12.2.2", "BeginTransformFeedback", "void BeginTransformFeedback( enum primitiveMode );"),
    (357, "12.2.2", "EndTransformFeedback", "void EndTransformFeedback( void );"),
    (357, "12.2.2", "PauseTransformFeedback", "void PauseTransformFeedback( void );"),
    (357, "12.2.2", "ResumeTransformFeedback", "void ResumeTransformFeedback( void );"),
)
SEALED_DECLARATIONS_SHA256 = "949c86118eb3858556249d86f25fb3feb712b345377f18db28aa28811a3b350f"
SOURCE_PROTOTYPE = re.compile(r"(?:void|boolean) [A-Z][A-Za-z0-9]*\([^;]*\);")
PROTOTYPE = re.compile(r"void ([A-Z][A-Za-z0-9]*)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A declaration lies outside the sealed capture-control slice."""


def reject(message: str) -> None:
    raise CatalogError(message)


def sha256(value: object) -> str:
    return hashlib.sha256(json.dumps(value, separators=(",", ":"), ensure_ascii=True).encode()).hexdigest()


def page_text(raw: bytes, page: int) -> str:
    try:
        return PDF.sealed_pdf_page(raw, page)
    except PDF.PdfError as error:
        reject(str(error))


def source_window(page: int, value: str) -> str:
    if page != 357:
        reject("capture-control declarations require physical page 357")
    start, end = value.find(SECTION_START[1]), value.find(SECTION_BOUNDARY[1])
    if start < 0 or end <= start:
        reject("capture-control section heading or physical-page end fence is missing")
    return value[start:end]


def source_slice(text) -> tuple[tuple[int, str], ...]:
    return tuple((page, item) for page in SOURCE_PAGES
                 for item in SOURCE_PROTOTYPE.findall(source_window(page, text(page))))


def bound_family(chunk_paths: dict[str, object], document) -> int:
    chunk, identifier, scopes, order, page, section, anchor = FAMILY
    rows = [row for row in document(chunk_paths[chunk]).get("families", []) if row.get("id") == identifier]
    expected = {"id": identifier, "family_kind": "declaration", "source_scope": list(scopes), "route": ROUTE,
                "reason": "formal-declaration-only", "source_order": order,
                "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": anchor}}
    if len(rows) != 1 or rows[0] != expected:
        reject("transform-feedback command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def facts(raw: bytes, pages: int, family_order: int, normalize) -> list[list[object]]:
    if (PROFILE != "gles-3.2" or pages != 601 or PAGES != 601 or family_order != 26
            or PRIMARY_PAGE != 357 or PRIMARY_SECTION != "12.2.2" or SOURCE_PAGES != (357,)
            or SECTION_START != (357, "12.2.2 Transform Feedback Primitive Capture")
            or SECTION_BOUNDARY != (357, "OpenGL ES 3.2 (May 5, 2022)")
            or sha256(DECLARATIONS) != SEALED_DECLARATIONS_SHA256):
        reject("capture-control source boundary, locations, or order is incomplete")
    texts: dict[int, str] = {}

    def text(page: int) -> str:
        if page not in texts:
            texts[page] = page_text(raw, page)
        return texts[page]

    expected = tuple((page, declaration) for page, _, _, declaration in DECLARATIONS)
    if source_slice(text) != expected:
        reject("capture-control source window is incomplete, rerouted, or out of source order")
    result, names = [], set()
    for page, section, name, declaration in DECLARATIONS:
        match = PROTOTYPE.fullmatch(declaration)
        if page != 357 or section != "12.2.2" or match is None or match.group(1) != name:
            reject("capture-control declaration is malformed or outside this source slice")
        try:
            c_names = normalize([("literal", name)])
        except Exception as error:
            reject(f"declaration grammar rejected a capture-control literal: {error}")
        if c_names != ["gl" + name] or name in names:
            reject("capture-control literal is duplicate, prefixed, or cross-family")
        names.add(name)
        result.append([f"gles32-transform-feedback-capture-control-{len(result) + 1:02d}",
                       name, c_names[0], declaration, page, section, len(result) + 1])
    if len(result) != 4 or [row[-1] for row in result] != list(range(1, 5)):
        reject("capture-control raw ordering is incomplete or unstable")
    return result
