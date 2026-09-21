"""Bounded formal GLES buffer declarations for F03.3.2.2.3.2 only."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "buffer-commands"
FAMILY = ("declarations", "buffer", "6", 14, 68, "void GenBuffers( sizei n, uint *buffers );")
SECTION_WITNESSES = {
    "6": (68, "Chapter 6 Buffer Objects"), "6.1": (69, "6.1 Creating and Binding Buffer Objects"),
    "6.1.1": (71, "6.1.1 Binding Buffer Objects to Indexed Targets"),
    "6.2": (72, "6.2 Creating and Modifying Buffer Object Data Stores"),
    "6.3": (74, "6.3 Mapping and Unmapping Buffer Data"), "6.3.1": (78, "6.3.1 Unmapping Buffers"),
    "6.5": (79, "6.5 Copying Between Buffers"), "6.6": (80, "6.6 Buffer Object Queries"),
}
# physical page, section, unprefixed command, exact normalized source declaration
DECLARATIONS = (
    (68, "6", "GenBuffers", "void GenBuffers( sizei n, uint *buffers );"),
    (68, "6", "DeleteBuffers", "void DeleteBuffers( sizei n, const uint *buffers );"),
    (69, "6", "IsBuffer", "boolean IsBuffer( uint buffer );"),
    (69, "6.1", "BindBuffer", "void BindBuffer( enum target, uint buffer );"),
    (71, "6.1.1", "BindBufferRange", "void BindBufferRange( enum target, uint index, uint buffer, intptr offset, sizeiptr size );"),
    (71, "6.1.1", "BindBufferBase", "void BindBufferBase( enum target, uint index, uint buffer );"),
    (72, "6.2", "BufferData", "void BufferData( enum target, sizeiptr size, const void *data, enum usage );"),
    (74, "6.2", "BufferSubData", "void BufferSubData( enum target, intptr offset, sizeiptr size, const void *data );"),
    (75, "6.3", "MapBufferRange", "void *MapBufferRange( enum target, intptr offset, sizeiptr length, bitfield access );"),
    (77, "6.3", "FlushMappedBufferRange", "void FlushMappedBufferRange( enum target, intptr offset, sizeiptr length );"),
    (78, "6.3.1", "UnmapBuffer", "boolean UnmapBuffer( enum target );"),
    (79, "6.5", "CopyBufferSubData", "void CopyBufferSubData( enum readtarget, enum writetarget, intptr readoffset, intptr writeoffset, sizeiptr size );"),
    (80, "6.6", "GetBufferParameteriv", "void GetBufferParameteriv( enum target, enum pname, int *data );"),
    (80, "6.6", "GetBufferParameteri64v", "void GetBufferParameteri64v( enum target, enum pname, int64 *data );"),
    (80, "6.6", "GetBufferPointerv", "void GetBufferPointerv( enum target, enum pname, void **params );"),
)
SEALED_DECLARATIONS_SHA256 = "7a5fa27c3cbae2aa696319f8c2f796385d019d67336d7d9f83874f59a9e9154c"
SECTION_PAGES = {"6": (68, 69), "6.1": (69,), "6.1.1": (71,), "6.2": (72, 74),
                 "6.3": (75, 77), "6.3.1": (78,), "6.5": (79,), "6.6": (80,)}
SECTION_BOUNDARIES = {(69, "6"): ("before", "6.1 Creating and Binding Buffer Objects"),
                      (69, "6.1"): ("after", "6.1 Creating and Binding Buffer Objects")}
PROTOTYPE = re.compile(r"(?:void |void \*|boolean )([A-Z][A-Za-z0-9]*)\(.*\);\Z")
SOURCE_PROTOTYPE = re.compile(r"(?:void \*|void |boolean )[A-Z][A-Za-z0-9]*\(.*?\);")


class CatalogError(ValueError):
    """A proposed buffer declaration lies outside the sealed source slice."""


def reject(message: str) -> None:
    raise CatalogError(message)


def compact(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def declaration_sha256(rows: object) -> str:
    return hashlib.sha256(json.dumps(rows, separators=(",", ":"), ensure_ascii=True).encode("utf-8")).hexdigest()


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw,
                                capture_output=True, check=False)
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        reject(f"cannot read sealed buffer page: {error}")
    if result.returncode != 0:
        reject("cannot read sealed buffer page")
    return compact(value)


def in_section_window(page: int, section: str, declaration: str, value: str) -> bool:
    boundary = SECTION_BOUNDARIES.get((page, section))
    if boundary is None:
        return True
    relation, marker = boundary
    start, split = value.find(declaration), value.find(marker)
    return start >= 0 and split >= 0 and ((relation == "before" and start < split) or (relation == "after" and start > split))


def source_slice(text) -> tuple[tuple[int, str], ...]:
    return tuple((page, item) for page in range(68, 81) for item in SOURCE_PROTOTYPE.findall(text(page)))


def bound_family(chunk_paths: dict[str, object], document) -> int:
    chunk, identifier, section, order, page, text = FAMILY
    value = document(chunk_paths[chunk])
    rows = [row for row in value.get("families", []) if row.get("id") == identifier]
    expected = {"id": identifier, "family_kind": "declaration", "source_scope": [section], "route": ROUTE,
                "reason": "formal-declaration-only", "source_order": order,
                "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": text}}
    if len(rows) != 1 or rows[0] != expected:
        reject("buffer command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def facts(raw: bytes, pages: int, family_order: int, normalize) -> list[dict[str, object]]:
    if (pages != PAGES or family_order != FAMILY[3]
            or declaration_sha256(DECLARATIONS) != SEALED_DECLARATIONS_SHA256):
        reject("GLES buffer source boundary, literal set, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str:
        texts.setdefault(page, page_text(raw, page))
        return texts[page]
    expected_slice = tuple((page, declaration) for page, _, _, declaration in DECLARATIONS)
    if source_slice(text) != expected_slice:
        reject("formal buffer source window is incomplete, rerouted, or out of source order")
    result, names = [], set()
    for order, (page, section, name, declaration) in enumerate(DECLARATIONS, 1):
        witness, match = SECTION_WITNESSES.get(section), PROTOTYPE.fullmatch(declaration)
        page_value = text(page)
        if (witness is None or page not in SECTION_PAGES.get(section, ()) or witness[1] not in text(witness[0])
                or declaration not in page_value or not in_section_window(page, section, declaration, page_value)
                or match is None or match.group(1) != name or "{" in declaration):
            reject("formal buffer declaration or section witness is missing, malformed, or outside this source slice")
        try:
            c_names = normalize([("literal", name)])
        except Exception as error:
            reject(f"declaration grammar rejected a literal buffer name: {error}")
        if c_names != [f"gl{name}"] or name in names:
            reject("literal buffer declaration is duplicate, prefixed, or outside the sealed grammar")
        names.add(name)
        result.append({"raw_id": f"gles32-buffer-command-{order:02d}", "family_id": "buffer",
                       "source_family_order": family_order, "unprefixed_name": name, "c_name": c_names[0],
                       "declaration": declaration, "physical_page": page, "section": section,
                       "source_locator": f"gles32-pdf-v1:page={page};section={section}", "source_order": order,
                       "derivation_class": "literal"})
    if len(result) != len(DECLARATIONS) or [row["source_order"] for row in result] != list(range(1, len(result) + 1)):
        reject("raw buffer declaration ordering is incomplete or unstable")
    return result
