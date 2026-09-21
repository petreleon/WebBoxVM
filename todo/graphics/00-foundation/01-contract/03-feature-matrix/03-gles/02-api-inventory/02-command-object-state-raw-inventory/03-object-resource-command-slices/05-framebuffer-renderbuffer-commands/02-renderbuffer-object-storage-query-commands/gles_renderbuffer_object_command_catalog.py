"""Bounded GLES renderbuffer-object declarations for F03.3.2.2.3.5.2."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "framebuffer-renderbuffer-commands"
FAMILY = ("object-declarations", "framebuffer-renderbuffer", ("9",), 20, 244, "9.2", "void GenFramebuffers( sizei n, uint *framebuffers );")
SOURCE_PAGES = (252, 253, 254, 255, 256)
SECTION_WITNESSES = {
    "9.2.4": (252, "9.2.4 Renderbuffer Objects"),
    "9.2.6": (256, "9.2.6 Renderbuffer Object Queries"),
}
SECTION_PAGES = {"9.2.4": (252, 253, 254, 255), "9.2.6": (256,)}
PAGE_FRAGMENTS = {
    252: ("after", "9.2.4 Renderbuffer Objects"),
    255: ("before", "9.2.5 Required Renderbuffer Formats"),
    256: ("before", "9.2.7 Attaching Renderbuffer Images to a Framebuffer"),
}
DECLARATIONS = (
    (252, "9.2.4", "BindRenderbuffer", "void BindRenderbuffer( enum target, uint renderbuffer );"),
    (253, "9.2.4", "GenRenderbuffers", "void GenRenderbuffers( sizei n, uint *renderbuffers );"),
    (253, "9.2.4", "DeleteRenderbuffers", "void DeleteRenderbuffers( sizei n, const uint *renderbuffers );"),
    (254, "9.2.4", "IsRenderbuffer", "boolean IsRenderbuffer( uint renderbuffer );"),
    (254, "9.2.4", "RenderbufferStorageMultisample", "void RenderbufferStorageMultisample( enum target, sizei samples, enum internalformat, sizei width, sizei height );"),
    (255, "9.2.4", "RenderbufferStorage", "void RenderbufferStorage( enum target, enum internalformat, sizei width, sizei height );"),
    (256, "9.2.6", "GetRenderbufferParameteriv", "void GetRenderbufferParameteriv( enum target, enum pname, int *params );"),
)
SEALED_DECLARATIONS_SHA256 = "f33bb8ef2548c2bf369e7cc4f4bcb5eb5f5628798359bc89c22d7f90ee047af6"
SOURCE_PROTOTYPE = re.compile(r"(?:void|boolean) [A-Z][A-Za-z0-9]*\([^;]*\);")
PROTOTYPE = re.compile(r"(?:void|boolean) ([A-Z][A-Za-z0-9]*)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A declaration lies outside the sealed renderbuffer-object source slice."""


def reject(message: str) -> None: raise CatalogError(message)


def compact(value: str) -> str: return " ".join(value.replace("\f", " ").split())


def sha256(value: object) -> str:
    return hashlib.sha256(json.dumps(value, separators=(",", ":"), ensure_ascii=True).encode("utf-8")).hexdigest()


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw, capture_output=True, check=False)
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error: reject(f"cannot read sealed renderbuffer-object page: {error}")
    if result.returncode != 0: reject("cannot read sealed renderbuffer-object page")
    return compact(value)


def page_fragment(page: int, value: str) -> str:
    boundary = PAGE_FRAGMENTS.get(page)
    if boundary is None: return value
    relation, marker = boundary; split = value.find(marker)
    if split < 0: reject("renderbuffer-object page fragment marker is missing")
    return value[split + len(marker):] if relation == "after" else value[:split]


def source_slice(text) -> tuple[tuple[int, str], ...]:
    return tuple((page, item) for page in SOURCE_PAGES for item in SOURCE_PROTOTYPE.findall(page_fragment(page, text(page))))


def bound_family(chunk_paths: dict[str, object], document) -> int:
    chunk, identifier, scopes, order, page, section, anchor = FAMILY
    rows = [row for row in document(chunk_paths[chunk]).get("families", []) if row.get("id") == identifier]
    expected = {"id": identifier, "family_kind": "declaration", "source_scope": list(scopes), "route": ROUTE,
                "reason": "formal-declaration-only", "source_order": order,
                "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": anchor}}
    if len(rows) != 1 or rows[0] != expected: reject("framebuffer/renderbuffer command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def facts(raw: bytes, pages: int, family_order: int, normalize) -> list[list[object]]:
    fixed_witnesses = {"9.2.4": (252, "9.2.4 Renderbuffer Objects"), "9.2.6": (256, "9.2.6 Renderbuffer Object Queries")}
    fixed_pages = {"9.2.4": (252, 253, 254, 255), "9.2.6": (256,)}
    fixed_fragments = {252: ("after", "9.2.4 Renderbuffer Objects"), 255: ("before", "9.2.5 Required Renderbuffer Formats"), 256: ("before", "9.2.7 Attaching Renderbuffer Images to a Framebuffer")}
    if (PROFILE != "gles-3.2" or pages != PAGES or family_order != 20 or SOURCE_PAGES != (252, 253, 254, 255, 256) or SECTION_WITNESSES != fixed_witnesses
            or SECTION_PAGES != fixed_pages or PAGE_FRAGMENTS != fixed_fragments or sha256(DECLARATIONS) != SEALED_DECLARATIONS_SHA256):
        reject("GLES renderbuffer-object source boundary, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str:
        texts.setdefault(page, page_text(raw, page)); return texts[page]
    expected = tuple((page, declaration) for page, _, _, declaration in DECLARATIONS)
    if source_slice(text) != expected: reject("renderbuffer-object source window is incomplete, rerouted, or out of source order")
    result, names = [], set()
    for page, section, name, declaration in DECLARATIONS:
        witness, match, value = SECTION_WITNESSES.get(section), PROTOTYPE.fullmatch(declaration), text(page)
        if (witness is None or page not in SECTION_PAGES.get(section, ()) or witness[1] not in text(witness[0]) or declaration not in page_fragment(page, value)
                or match is None or match.group(1) != name):
            reject("renderbuffer-object declaration is malformed or outside this source slice")
        try: c_names = normalize([("literal", name)])
        except Exception as error: reject(f"declaration grammar rejected a renderbuffer-object literal: {error}")
        if c_names != ["gl" + name] or name in names: reject("renderbuffer-object literal is duplicate, prefixed, or cross-family")
        names.add(name); order = len(result) + 1
        result.append([f"gles32-renderbuffer-object-command-{order:02d}", name, c_names[0], declaration, page, section, order])
    if len(result) != 7 or [row[-1] for row in result] != list(range(1, 8)): reject("renderbuffer-object raw ordering is incomplete or unstable")
    return result
