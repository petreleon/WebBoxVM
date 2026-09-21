"""Bounded GLES texture extended declarations for F03.3.2.2.3.4.3."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "texture-sampler-commands"
FAMILY = ("object-declarations", "texture", ("8.1", "8.5-8.26"), 17, 157, "8.1", "void GenTextures( sizei n, uint *textures );;")
SOURCE_PAGES = (195, 199, 201, 203, 204, 222, 226, 227, 233)
SECTION_WITNESSES = {
    "8.7": (195, "8.7 Compressed Texture Images"), "8.8": (201, "8.8 Multisample Textures"),
    "8.9": (203, "8.9 Buffer Textures"), "8.14.4": (222, "8.14.4 Manual Mipmap Generation"),
    "8.18": (225, "8.18 Immutable-Format Texture Images"), "8.23": (233, "8.23 Texture Image Loads and Stores"),
}
SECTION_PAGES = {"8.7": (195, 199), "8.8": (201,), "8.9": (203, 204), "8.14.4": (222,), "8.18": (226, 227), "8.23": (233,)}
DECLARATIONS = (
    (195, "8.7", "CompressedTexImage2D", "void CompressedTexImage2D( enum target, int level, enum internalformat, sizei width, sizei height, int border, sizei imageSize, const void *data );"),
    (195, "8.7", "CompressedTexImage3D", "void CompressedTexImage3D( enum target, int level, enum internalformat, sizei width, sizei height, sizei depth, int border, sizei imageSize, const void *data );"),
    (199, "8.7", "CompressedTexSubImage2D", "void CompressedTexSubImage2D( enum target, int level, int xoffset, int yoffset, sizei width, sizei height, enum format, sizei imageSize, const void *data );"),
    (199, "8.7", "CompressedTexSubImage3D", "void CompressedTexSubImage3D( enum target, int level, int xoffset, int yoffset, int zoffset, sizei width, sizei height, sizei depth, enum format, sizei imageSize, const void *data );"),
    (201, "8.8", "TexStorage2DMultisample", "void TexStorage2DMultisample( enum target, sizei samples, enum internalformat, sizei width, sizei height, boolean fixedsamplelocations );"),
    (201, "8.8", "TexStorage3DMultisample", "void TexStorage3DMultisample( enum target, sizei samples, enum internalformat, sizei width, sizei height, sizei depth, boolean fixedsamplelocations );"),
    (203, "8.9", "TexBufferRange", "void TexBufferRange( enum target, enum internalformat, uint buffer, intptr offset, sizeiptr size );"),
    (204, "8.9", "TexBuffer", "void TexBuffer( enum target, enum internalformat, uint buffer );"),
    (222, "8.14.4", "GenerateMipmap", "void GenerateMipmap( enum target );"),
    (226, "8.18", "TexStorage2D", "void TexStorage2D( enum target, sizei levels, enum internalformat, sizei width, sizei height );"),
    (227, "8.18", "TexStorage3D", "void TexStorage3D( enum target, sizei levels, enum internalformat, sizei width, sizei height, sizei depth );"),
    (233, "8.23", "BindImageTexture", "void BindImageTexture( uint unit, uint texture, int level, boolean layered, int layer, enum access, enum format );"),
)
SEALED_DECLARATIONS_SHA256 = "f2d13daa4bc6feb118c6381ad634baebc3f899801e878481cc6e5e518f2eee23"
SOURCE_PROTOTYPE = re.compile(r"void [A-Z][A-Za-z0-9]*\([^;]*\);")
PROTOTYPE = re.compile(r"void ([A-Z][A-Za-z0-9]*)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A declaration lies outside the sealed texture extended command slice."""


def reject(message: str) -> None: raise CatalogError(message)


def compact(value: str) -> str: return " ".join(value.replace("\f", " ").split())


def sha256(value: object) -> str:
    return hashlib.sha256(json.dumps(value, separators=(",", ":"), ensure_ascii=True).encode("utf-8")).hexdigest()


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw, capture_output=True, check=False)
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error: reject(f"cannot read sealed texture extended page: {error}")
    if result.returncode != 0: reject("cannot read sealed texture extended page")
    return compact(value)


def source_slice(text) -> tuple[tuple[int, str], ...]:
    return tuple((page, item) for page in SOURCE_PAGES for item in SOURCE_PROTOTYPE.findall(text(page)))


def bound_family(chunk_paths: dict[str, object], document) -> int:
    chunk, identifier, scopes, order, page, section, anchor = FAMILY
    rows = [row for row in document(chunk_paths[chunk]).get("families", []) if row.get("id") == identifier]
    expected = {"id": identifier, "family_kind": "declaration", "source_scope": list(scopes), "route": ROUTE,
                "reason": "formal-declaration-only", "source_order": order,
                "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": anchor}}
    if len(rows) != 1 or rows[0] != expected: reject("texture command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def facts(raw: bytes, pages: int, family_order: int, normalize) -> list[list[object]]:
    fixed_sections = {"8.7": (195, "8.7 Compressed Texture Images"), "8.8": (201, "8.8 Multisample Textures"), "8.9": (203, "8.9 Buffer Textures"), "8.14.4": (222, "8.14.4 Manual Mipmap Generation"), "8.18": (225, "8.18 Immutable-Format Texture Images"), "8.23": (233, "8.23 Texture Image Loads and Stores")}
    if (pages != PAGES or family_order != 17 or SOURCE_PAGES != (195, 199, 201, 203, 204, 222, 226, 227, 233)
            or SECTION_WITNESSES != fixed_sections or SECTION_PAGES != {"8.7": (195, 199), "8.8": (201,), "8.9": (203, 204), "8.14.4": (222,), "8.18": (226, 227), "8.23": (233,)}
            or sha256(DECLARATIONS) != SEALED_DECLARATIONS_SHA256): reject("GLES texture extended source boundary, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str:
        texts.setdefault(page, page_text(raw, page)); return texts[page]
    expected = tuple((page, declaration) for page, _, _, declaration in DECLARATIONS)
    if source_slice(text) != expected: reject("texture extended source window is incomplete, rerouted, or out of source order")
    result, names = [], set()
    for page, section, name, declaration in DECLARATIONS:
        witness, match, value = SECTION_WITNESSES.get(section), PROTOTYPE.fullmatch(declaration), text(page)
        if (witness is None or page not in SECTION_PAGES.get(section, ()) or witness[1] not in text(witness[0]) or declaration not in value or match is None or match.group(1) != name): reject("texture extended declaration is malformed or outside this source slice")
        try: c_names = normalize([("literal", name)])
        except Exception as error: reject(f"declaration grammar rejected a texture extended literal: {error}")
        if c_names != ["gl" + name] or name in names: reject("texture extended literal is duplicate, prefixed, or cross-family")
        names.add(name); order = len(result) + 1
        result.append([f"gles32-texture-extended-command-{order:02d}", name, c_names[0], declaration, page, section, order])
    if len(result) != 12 or [row[-1] for row in result] != list(range(1, 13)): reject("texture extended raw ordering is incomplete or unstable")
    return result
