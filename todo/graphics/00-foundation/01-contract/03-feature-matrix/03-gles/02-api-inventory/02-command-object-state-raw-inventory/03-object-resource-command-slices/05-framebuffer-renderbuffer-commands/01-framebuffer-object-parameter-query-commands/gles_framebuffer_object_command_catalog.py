"""Bounded GLES framebuffer-object declarations for F03.3.2.2.3.5.1."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "framebuffer-renderbuffer-commands"
FAMILY = ("object-declarations", "framebuffer-renderbuffer", ("9",), 20, 244, "9.2", "void GenFramebuffers( sizei n, uint *framebuffers );")
SOURCE_PAGES = (242, 244, 245, 248)
SECTION_WITNESSES = {
    "9.2": (242, "9.2 Binding and Managing Framebuffer Objects"),
    "9.2.1": (245, "9.2.1 Framebuffer Object Parameters"),
    "9.2.3": (248, "9.2.3 Framebuffer Object Queries"),
}
SECTION_PAGES = {"9.2": (242, 244, 245), "9.2.1": (245,), "9.2.3": (248,)}
SECTION_BOUNDARIES = {
    (245, "9.2"): ("before", "9.2.1 Framebuffer Object Parameters"),
    (245, "9.2.1"): ("after", "9.2.1 Framebuffer Object Parameters"),
}
DECLARATIONS = (
    (242, "9.2", "BindFramebuffer", "void BindFramebuffer( enum target, uint framebuffer );"),
    (244, "9.2", "GenFramebuffers", "void GenFramebuffers( sizei n, uint *framebuffers );"),
    (245, "9.2", "DeleteFramebuffers", "void DeleteFramebuffers( sizei n, const uint *framebuffers );"),
    (245, "9.2", "IsFramebuffer", "boolean IsFramebuffer( uint framebuffer );"),
    (245, "9.2.1", "FramebufferParameteri", "void FramebufferParameteri( enum target, enum pname, int param );"),
    (248, "9.2.3", "GetFramebufferParameteriv", "void GetFramebufferParameteriv( enum target, enum pname, int *params );"),
    (248, "9.2.3", "GetFramebufferAttachmentParameteriv", "void GetFramebufferAttachmentParameteriv( enum target, enum attachment, enum pname, int *params );"),
)
SEALED_DECLARATIONS_SHA256 = "07fb9e552812d21d850329085daae82c1c06b88cf1e697f6e0ad204fefc6a94d"
SOURCE_PROTOTYPE = re.compile(r"(?:void|boolean) [A-Z][A-Za-z0-9]*\([^;]*\);")
PROTOTYPE = re.compile(r"(?:void|boolean) ([A-Z][A-Za-z0-9]*)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A declaration lies outside the sealed framebuffer-object source slice."""


def reject(message: str) -> None: raise CatalogError(message)


def compact(value: str) -> str: return " ".join(value.replace("\f", " ").split())


def sha256(value: object) -> str:
    return hashlib.sha256(json.dumps(value, separators=(",", ":"), ensure_ascii=True).encode("utf-8")).hexdigest()


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw, capture_output=True, check=False)
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error: reject(f"cannot read sealed framebuffer-object page: {error}")
    if result.returncode != 0: reject("cannot read sealed framebuffer-object page")
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
    if len(rows) != 1 or rows[0] != expected: reject("framebuffer/renderbuffer command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def facts(raw: bytes, pages: int, family_order: int, normalize) -> list[list[object]]:
    fixed_witnesses = {"9.2": (242, "9.2 Binding and Managing Framebuffer Objects"), "9.2.1": (245, "9.2.1 Framebuffer Object Parameters"), "9.2.3": (248, "9.2.3 Framebuffer Object Queries")}
    fixed_pages = {"9.2": (242, 244, 245), "9.2.1": (245,), "9.2.3": (248,)}
    fixed_boundaries = {(245, "9.2"): ("before", "9.2.1 Framebuffer Object Parameters"), (245, "9.2.1"): ("after", "9.2.1 Framebuffer Object Parameters")}
    if (pages != PAGES or family_order != 20 or SOURCE_PAGES != (242, 244, 245, 248) or SECTION_WITNESSES != fixed_witnesses
            or SECTION_PAGES != fixed_pages or SECTION_BOUNDARIES != fixed_boundaries or sha256(DECLARATIONS) != SEALED_DECLARATIONS_SHA256):
        reject("GLES framebuffer-object source boundary, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str:
        texts.setdefault(page, page_text(raw, page)); return texts[page]
    expected = tuple((page, declaration) for page, _, _, declaration in DECLARATIONS)
    if source_slice(text) != expected: reject("framebuffer-object source window is incomplete, rerouted, or out of source order")
    result, names = [], set()
    for page, section, name, declaration in DECLARATIONS:
        witness, match, value = SECTION_WITNESSES.get(section), PROTOTYPE.fullmatch(declaration), text(page)
        if (witness is None or page not in SECTION_PAGES.get(section, ()) or witness[1] not in text(witness[0]) or declaration not in value
                or not in_section(page, section, declaration, value) or match is None or match.group(1) != name):
            reject("framebuffer-object declaration is malformed or outside this source slice")
        try: c_names = normalize([("literal", name)])
        except Exception as error: reject(f"declaration grammar rejected a framebuffer-object literal: {error}")
        if c_names != ["gl" + name] or name in names: reject("framebuffer-object literal is duplicate, prefixed, or cross-family")
        names.add(name); order = len(result) + 1
        result.append([f"gles32-framebuffer-object-command-{order:02d}", name, c_names[0], declaration, page, section, order])
    if len(result) != 7 or [row[-1] for row in result] != list(range(1, 8)): reject("framebuffer-object raw ordering is incomplete or unstable")
    return result
