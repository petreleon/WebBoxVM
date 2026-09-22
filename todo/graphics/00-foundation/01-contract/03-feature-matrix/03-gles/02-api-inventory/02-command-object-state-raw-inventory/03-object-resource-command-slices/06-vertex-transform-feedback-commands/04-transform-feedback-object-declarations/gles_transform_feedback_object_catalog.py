"""Bounded GLES transform-feedback-object declarations for F03.3.2.2.3.6.4."""

from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
from pathlib import Path

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "vertex-transform-feedback-commands"
FAMILY = ("object-declarations", "transform-feedback", ("12.2",), 26, 357, "12.2",
          "void BeginTransformFeedback( enum primitiveMode );")
PRIMARY_PAGE, PRIMARY_SECTION = 354, "12.2.1"
SOURCE_PAGES = (354, 355, 356)
SECTION_WITNESSES = {"12.2.1": (354, "12.2.1 Transform Feedback Objects")}
SECTION_PAGES = {"12.2.1": (354, 355, 356)}
SECTION_START = (354, "12.2.1 Transform Feedback Objects")
SECTION_BOUNDARY = (357, "12.2.2 Transform Feedback Primitive Capture")
DECLARATIONS = (
    (354, "12.2.1", "GenTransformFeedbacks", "void GenTransformFeedbacks( sizei n, uint *ids );"),
    (355, "12.2.1", "DeleteTransformFeedbacks", "void DeleteTransformFeedbacks( sizei n, const uint *ids );"),
    (355, "12.2.1", "IsTransformFeedback", "boolean IsTransformFeedback( uint id );"),
    (356, "12.2.1", "BindTransformFeedback", "void BindTransformFeedback( enum target, uint id );"),
)
SEALED_DECLARATIONS_SHA256 = "a38b79afe129494cd7bf30b66f0e1f1be3f77d07666be87e9ec8e030b1e67a72"
TRUSTED_EXTRACTORS = (Path("/opt/homebrew/bin/pdftotext"), Path("/usr/local/bin/pdftotext"), Path("/usr/bin/pdftotext"))
SOURCE_PROTOTYPE = re.compile(r"(?:void|boolean) [A-Z][A-Za-z0-9]*\([^;]*\);")
PROTOTYPE = re.compile(r"(?:void|boolean) ([A-Z][A-Za-z0-9]*)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A declaration lies outside the sealed transform-feedback-object slice."""


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
    except (OSError, UnicodeDecodeError) as error: reject(f"cannot read sealed transform-feedback-object page: {error}")
    if result.returncode != 0: reject("cannot read sealed transform-feedback-object page")
    return compact(value)


def source_window(page: int, value: str) -> str:
    if page != SECTION_START[0]: return value
    split = value.find(SECTION_START[1])
    if split < 0: reject("transform-feedback-object source start fence is missing")
    return value[split:]


def source_slice(text) -> tuple[tuple[int, str], ...]:
    return tuple((page, item) for page in SOURCE_PAGES for item in SOURCE_PROTOTYPE.findall(source_window(page, text(page))))


def in_section(page: int, declaration: str, value: str) -> bool:
    if page != SECTION_START[0]: return True
    return value.find(SECTION_START[1]) >= 0 and value.find(declaration) >= value.find(SECTION_START[1])


def bound_family(chunk_paths: dict[str, object], document) -> int:
    chunk, identifier, scopes, order, page, section, anchor = FAMILY
    rows = [row for row in document(chunk_paths[chunk]).get("families", []) if row.get("id") == identifier]
    expected = {"id": identifier, "family_kind": "declaration", "source_scope": list(scopes), "route": ROUTE,
                "reason": "formal-declaration-only", "source_order": order,
                "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": anchor}}
    if len(rows) != 1 or rows[0] != expected: reject("transform-feedback command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def facts(raw: bytes, pages: int, family_order: int, normalize) -> list[list[object]]:
    fixed_witnesses = {"12.2.1": (354, "12.2.1 Transform Feedback Objects")}
    fixed_pages, fixed_start = {"12.2.1": (354, 355, 356)}, (354, "12.2.1 Transform Feedback Objects")
    fixed_boundary = (357, "12.2.2 Transform Feedback Primitive Capture")
    if (PROFILE != "gles-3.2" or pages != PAGES or family_order != 26 or PRIMARY_PAGE != 354 or PRIMARY_SECTION != "12.2.1"
            or SOURCE_PAGES != (354, 355, 356) or SECTION_WITNESSES != fixed_witnesses or SECTION_PAGES != fixed_pages or SECTION_START != fixed_start
            or SECTION_BOUNDARY != fixed_boundary or sha256(DECLARATIONS) != SEALED_DECLARATIONS_SHA256):
        reject("GLES transform-feedback-object source boundary, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str: texts.setdefault(page, page_text(raw, page)); return texts[page]
    expected = tuple((page, declaration) for page, _, _, declaration in DECLARATIONS)
    if source_slice(text) != expected: reject("transform-feedback-object source window is incomplete, rerouted, or out of source order")
    if SECTION_BOUNDARY[1] not in text(SECTION_BOUNDARY[0]): reject("transform-feedback capture boundary is missing")
    result, names = [], set()
    for page, section, name, declaration in DECLARATIONS:
        witness, match, value = SECTION_WITNESSES.get(section), PROTOTYPE.fullmatch(declaration), text(page)
        if (witness is None or page not in SECTION_PAGES.get(section, ()) or witness[1] not in text(witness[0]) or declaration not in value or not in_section(page, declaration, value)
                or match is None or match.group(1) != name):
            reject("transform-feedback-object declaration is malformed or outside this source slice")
        try: c_names = normalize([("literal", name)])
        except Exception as error: reject(f"declaration grammar rejected a transform-feedback-object literal: {error}")
        if c_names != ["gl" + name] or name in names: reject("transform-feedback-object literal is duplicate, prefixed, or cross-family")
        names.add(name); result.append([f"gles32-transform-feedback-object-{len(result) + 1:02d}", name, c_names[0], declaration, page, section, len(result) + 1])
    if len(result) != 4 or [row[-1] for row in result] != list(range(1, 5)): reject("transform-feedback-object raw ordering is incomplete or unstable")
    return result
