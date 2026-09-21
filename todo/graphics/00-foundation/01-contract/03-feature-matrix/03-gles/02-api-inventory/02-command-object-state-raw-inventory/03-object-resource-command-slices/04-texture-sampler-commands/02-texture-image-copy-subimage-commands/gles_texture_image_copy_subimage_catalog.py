"""Bounded GLES texture image/copy/subimage declarations for F03.3.2.2.3.4.2."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "texture-sampler-commands"
FAMILY = ("object-declarations", "texture", ("8.1", "8.5-8.26"), 17, 157, "8.1", "void GenTextures( sizei n, uint *textures );;")
SOURCE_PAGES, DIRECT_PAGES = (175, 184, 185, 186, 187, 191, 192), (175, 184, 192)
SECTION_WITNESSES = {"8.5": (175, "8.5 Texture Image Specification"), "8.6": (185, "8.6 Alternate Texture Image Specification Commands")}
SECTION_PAGES = {"8.5": (175, 184), "8.6": (185, 187, 191, 192)}
FORMS = (
    ((175,), "8.5", "TexImage3D", "void TexImage3D( enum target, int level, int internalformat, sizei width, sizei height, sizei depth, int border, enum format, enum type, const void *data );"),
    ((184,), "8.5", "TexImage2D", "void TexImage2D( enum target, int level, int internalformat, sizei width, sizei height, int border, enum format, enum type, const void *data );"),
    ((185, 187), "8.6", "CopyTexImage2D", "void CopyTexImage2D( enum target, int level, enum internalformat, int x, int y, sizei width, sizei height, int border );"),
    ((191, 192), "8.6", "TexSubImage3D", "void TexSubImage3D( enum target, int level, int xoffset, int yoffset, int zoffset, sizei width, sizei height, sizei depth, enum format, enum type, const void *data );"),
    ((192,), "8.6", "TexSubImage2D", "void TexSubImage2D( enum target, int level, int xoffset, int yoffset, sizei width, sizei height, enum format, enum type, const void *data );"),
    ((192,), "8.6", "CopyTexSubImage3D", "void CopyTexSubImage3D( enum target, int level, int xoffset, int yoffset, int zoffset, int x, int y, sizei width, sizei height );"),
    ((192,), "8.6", "CopyTexSubImage2D", "void CopyTexSubImage2D( enum target, int level, int xoffset, int yoffset, int x, int y, sizei width, sizei height );"),
)
SPLIT_FRAGMENTS = (
    ("CopyTexImage2D", ((185, "void CopyTexImage2D( enum target, int level,"), (187, "enum internalformat, int x, int y, sizei width, sizei height, int border );"))),
    ("TexSubImage3D", ((191, "void TexSubImage3D( enum target, int level, int xoffset,"), (192, "int yoffset, int zoffset, sizei width, sizei height, sizei depth, enum format, enum type, const void *data );"))),
)
GAP_WITNESS = (185, 186, 187, "Figure 8.6. A texture image and the coordinates used to access it.")
SEALED_FORMS_SHA256 = "4ccd5a41f154b2a4ace01f7e251f337b1963e0c96b28ee59a6473163bbdcb0fb"
SEALED_FRAGMENTS_SHA256 = "cbc4a5703a40836884b7d966b38fe38ed4792e0e653adc71f97508ba3767aac8"
SOURCE_PROTOTYPE = re.compile(r"void [A-Z][A-Za-z0-9]*\([^;]*\);")
PROTOTYPE = re.compile(r"void ([A-Z][A-Za-z0-9]*)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A declaration lies outside the sealed texture image/copy/subimage slice."""


def reject(message: str) -> None:
    raise CatalogError(message)


def compact(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def sha256(value: object) -> str:
    return hashlib.sha256(json.dumps(value, separators=(",", ":"), ensure_ascii=True).encode("utf-8")).hexdigest()


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw,
                                capture_output=True, check=False)
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        reject(f"cannot read sealed texture image/copy page: {error}")
    if result.returncode != 0: reject("cannot read sealed texture image/copy page")
    return compact(value)


def direct_slice(text) -> tuple[tuple[int, str], ...]:
    return tuple((page, item) for page in DIRECT_PAGES for item in SOURCE_PROTOTYPE.findall(text(page)))


def split_declarations(text) -> tuple[tuple[str, str], ...]:
    values = []
    for name, pieces in SPLIT_FRAGMENTS:
        if any(part not in text(page) for page, part in pieces): reject("split texture declaration fragment is missing or rerouted")
        values.append((name, " ".join(part for _, part in pieces)))
    return tuple(values)


def bound_family(chunk_paths: dict[str, object], document) -> int:
    chunk, identifier, scopes, order, page, section, anchor = FAMILY
    rows = [row for row in document(chunk_paths[chunk]).get("families", []) if row.get("id") == identifier]
    expected = {"id": identifier, "family_kind": "declaration", "source_scope": list(scopes), "route": ROUTE,
                "reason": "formal-declaration-only", "source_order": order,
                "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": anchor}}
    if len(rows) != 1 or rows[0] != expected: reject("texture command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def facts(raw: bytes, pages: int, family_order: int, normalize) -> list[list[object]]:
    fixed_pages = (175, 184, 185, 186, 187, 191, 192)
    fixed_sections = {"8.5": (175, "8.5 Texture Image Specification"), "8.6": (185, "8.6 Alternate Texture Image Specification Commands")}
    if (pages != PAGES or family_order != 17 or SOURCE_PAGES != fixed_pages or DIRECT_PAGES != (175, 184, 192)
            or SECTION_WITNESSES != fixed_sections or SECTION_PAGES != {"8.5": (175, 184), "8.6": (185, 187, 191, 192)}
            or GAP_WITNESS != (185, 186, 187, "Figure 8.6. A texture image and the coordinates used to access it.")
            or sha256(FORMS) != SEALED_FORMS_SHA256 or sha256(SPLIT_FRAGMENTS) != SEALED_FRAGMENTS_SHA256):
        reject("GLES texture image/copy source boundary, fragments, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str:
        texts.setdefault(page, page_text(raw, page)); return texts[page]
    start, gap, end, needle = GAP_WITNESS
    if needle not in text(gap) or "void CopyTexImage2D" in text(gap): reject("figure-only CopyTexImage2D gap is missing or promoted")
    expected_direct = tuple((span[0], declaration) for span, _, _, declaration in FORMS if len(span) == 1)
    expected_splits = tuple((name, declaration) for span, _, name, declaration in FORMS if len(span) > 1)
    if direct_slice(text) != expected_direct or split_declarations(text) != expected_splits: reject("texture image/copy source window is incomplete, rerouted, or out of source order")
    result, names, split_values = [], set(), dict(expected_splits)
    for span, section, name, declaration in FORMS:
        witness, match = SECTION_WITNESSES.get(section), PROTOTYPE.fullmatch(declaration)
        present = declaration in text(span[0]) if len(span) == 1 else split_values.get(name) == declaration
        if (witness is None or any(page not in SECTION_PAGES.get(section, ()) for page in span) or witness[1] not in text(witness[0])
                or not present or match is None or match.group(1) != name): reject("texture image/copy declaration is malformed or outside this source slice")
        try: c_names = normalize([("literal", name)])
        except Exception as error: reject(f"declaration grammar rejected a texture image/copy literal: {error}")
        if c_names != ["gl" + name] or name in names: reject("texture image/copy literal is duplicate, prefixed, or cross-family")
        names.add(name); order = len(result) + 1
        result.append([f"gles32-texture-image-copy-subimage-{order:02d}", name, c_names[0], declaration, span[0], list(span), section, order])
    if len(result) != 7 or [row[-1] for row in result] != list(range(1, 8)): reject("texture image/copy raw ordering is incomplete or unstable")
    return result
