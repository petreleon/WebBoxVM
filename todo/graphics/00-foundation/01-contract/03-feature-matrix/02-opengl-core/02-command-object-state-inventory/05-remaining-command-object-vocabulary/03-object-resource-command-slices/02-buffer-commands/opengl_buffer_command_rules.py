#!/usr/bin/env python3
"""Fail-closed literal buffer declaration anchors for F03.2.2.5.3.2."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess

PROFILE, PAGES, PREFIX = "opengl-4.6-core", 851, "gl"
WINDOWS = (("6.1", tuple(range(81, 87)), 9), ("6.2", tuple(range(87, 95)), 10),
           ("6.3", tuple(range(95, 101)), 8), ("6.4", (101,), 0), ("6.5", (102,), 2),
           ("6.6", (102,), 2), ("6.7", tuple(range(103, 106)), 8))
SECTION_WITNESSES = {"6.1": (82, "6.1 Creating and Binding Buffer Objects"),
                     "6.2": (87, "6.2 Creating and Modifying Buffer Object Data Stores"),
                     "6.3": (95, "6.3 Mapping and Unmapping Buffer Data"),
                     "6.4": (101, "6.4 Effects of Accessing Outside Buffer Bounds"),
                     "6.5": (102, "6.5 Invalidating Buffer Data"), "6.6": (102, "6.6 Copying Between Buffers"),
                     "6.7": (103, "6.7 Buffer Object Queries")}
# fragment, physical-page span, section, bare source name, literal declaration, baseline duplicate
DECLARATIONS = (
    ("binding", (81,), "6.1", "GenBuffers", "void GenBuffers( sizei n, uint *buffers );", False),
    ("binding", (81,), "6.1", "CreateBuffers", "void CreateBuffers( sizei n, uint *buffers );", True),
    ("binding", (82,), "6.1", "DeleteBuffers", "void DeleteBuffers( sizei n, const uint *buffers );", False),
    ("binding", (82,), "6.1", "IsBuffer", "boolean IsBuffer( uint buffer );", False),
    ("binding", (82,), "6.1", "BindBuffer", "void BindBuffer( enum target, uint buffer );", False),
    ("binding", (84,), "6.1", "BindBufferRange", "void BindBufferRange( enum target, uint index, uint buffer, intptr offset, sizeiptr size );", False),
    ("binding", (84,), "6.1", "BindBufferBase", "void BindBufferBase( enum target, uint index, uint buffer );", False),
    ("binding", (85, 86), "6.1", "BindBuffersBase", "void BindBuffersBase( enum target, uint first, sizei count, const uint *buffers );", False),
    ("binding", (86,), "6.1", "BindBuffersRange", "void BindBuffersRange( enum target, uint first, sizei count, const uint *buffers, const intptr *offsets, const sizeiptr *sizes );", False),
    ("storage", (87,), "6.2", "BufferStorage", "void BufferStorage( enum target, sizeiptr size, const void *data, bitfield flags );", False),
    ("storage", (87,), "6.2", "NamedBufferStorage", "void NamedBufferStorage( uint buffer, sizeiptr size, const void *data, bitfield flags );", False),
    ("storage", (90,), "6.2", "BufferData", "void BufferData( enum target, sizeiptr size, const void *data, enum usage );", False),
    ("storage", (90,), "6.2", "NamedBufferData", "void NamedBufferData( uint buffer, sizeiptr size, const void *data, enum usage );", False),
    ("storage", (92,), "6.2", "BufferSubData", "void BufferSubData( enum target, intptr offset, sizeiptr size, const void *data );", False),
    ("storage", (92,), "6.2", "NamedBufferSubData", "void NamedBufferSubData( uint buffer, intptr offset, sizeiptr size, const void *data );", False),
    ("storage", (93,), "6.2", "ClearBufferSubData", "void ClearBufferSubData( enum target, enum internalformat, intptr offset, sizeiptr size, enum format, enum type, const void *data );", False),
    ("storage", (93,), "6.2", "ClearNamedBufferSubData", "void ClearNamedBufferSubData( uint buffer, enum internalformat, intptr offset, sizeiptr size, enum format, enum type, const void *data );", False),
    ("storage", (94,), "6.2", "ClearBufferData", "void ClearBufferData( enum target, enum internalformat, enum format, enum type, const void *data );", False),
    ("storage", (94,), "6.2", "ClearNamedBufferData", "void ClearNamedBufferData( uint buffer, enum internalformat, enum format, enum type, const void *data );", False),
    ("mapping", (95,), "6.3", "MapBufferRange", "void *MapBufferRange( enum target, intptr offset, sizeiptr length, bitfield acesss );", False),
    ("mapping", (95,), "6.3", "MapNamedBufferRange", "void *MapNamedBufferRange( uint buffer, intptr offset, sizeiptr length, bitfield access );", False),
    ("mapping", (98,), "6.3", "MapBuffer", "void *MapBuffer( enum target, enum access );", False),
    ("mapping", (98,), "6.3", "MapNamedBuffer", "void *MapNamedBuffer( uint buffer, enum access );", False),
    ("mapping", (99,), "6.3", "FlushMappedBufferRange", "void FlushMappedBufferRange( enum target, intptr offset, sizeiptr length );", False),
    ("mapping", (99,), "6.3", "FlushMappedNamedBufferRange", "void FlushMappedNamedBufferRange( uint buffer, intptr offset, sizeiptr length );", False),
    ("mapping", (100,), "6.3", "UnmapBuffer", "boolean UnmapBuffer( enum target );", False),
    ("mapping", (100,), "6.3", "UnmapNamedBuffer", "boolean UnmapNamedBuffer( uint buffer );", False),
    ("transfer", (102,), "6.5", "InvalidateBufferSubData", "void InvalidateBufferSubData( uint buffer, intptr offset, sizeiptr length );", False),
    ("transfer", (102,), "6.5", "InvalidateBufferData", "void InvalidateBufferData( uint buffer );", False),
    ("transfer", (102,), "6.6", "CopyBufferSubData", "void CopyBufferSubData( enum readTarget, enum writeTarget, intptr readOffset, intptr writeOffset, sizeiptr size );", False),
    ("transfer", (102,), "6.6", "CopyNamedBufferSubData", "void CopyNamedBufferSubData( uint readBuffer, uint writeBuffer, intptr readOffset, intptr writeOffset, sizeiptr size );", False),
    ("query", (103,), "6.7", "GetBufferParameteriv", "void GetBufferParameteriv( enum target, enum pname, int *data );", False),
    ("query", (103,), "6.7", "GetBufferParameteri64v", "void GetBufferParameteri64v( enum target, enum pname, int64 *data );", False),
    ("query", (103,), "6.7", "GetNamedBufferParameteriv", "void GetNamedBufferParameteriv( uint buffer, enum pname, int *data );", False),
    ("query", (104,), "6.7", "GetNamedBufferParameteri64v", "void GetNamedBufferParameteri64v( uint buffer, enum pname, int64 *data );", False),
    ("query", (104,), "6.7", "GetBufferSubData", "void GetBufferSubData( enum target, intptr offset, sizeiptr size, void *data );", False),
    ("query", (104,), "6.7", "GetNamedBufferSubData", "void GetNamedBufferSubData( uint buffer, intptr offset, sizeiptr size, void *data );", False),
    ("query", (105,), "6.7", "GetBufferPointerv", "void GetBufferPointerv( enum target, enum pname, const void **params );", False),
    ("query", (105,), "6.7", "GetNamedBufferPointerv", "void GetNamedBufferPointerv( uint buffer, enum pname, const void **params );", False),
)
CLOSED_DECLARATIONS_SHA256 = "a804755e705103d81dadd6154634c9d2f1e95803759de142934af1620e61e0cb"
SIGNATURE = re.compile(r"(?:(?:void|boolean) [A-Z][A-Za-z0-9]*|void \*[A-Z][A-Za-z0-9]*)\( [^;()]* \);")
DECLARATION = re.compile(r"^(?:(?:void|boolean) |void \*)(?P<name>[A-Z][A-Za-z0-9]*)\( (?P<arguments>[^;()]+) \);$")
SPAN_START, SPAN_END = "void BindBuffersBase( enum target, uint first, sizei count,", "const uint *buffers );"


class RuleError(ValueError):
    """A declaration is absent, ambiguous, outside this source slice, or promoted."""


def reject(message: str) -> None:
    raise RuleError(message)


def normalized(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-layout", "-", "-"], input=raw, capture_output=True, check=False)
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        reject(f"cannot read sealed buffer command page: {error}")
    if result.returncode: reject("cannot read sealed buffer command page")
    return normalized(value)


def locator(pages: tuple[int, ...], section: str) -> str:
    return f"opengl46-core-pdf-v1:page={pages[0]};section={section}"


def anchored_source(raw: bytes) -> dict[int, str]:
    if hashlib.sha256(json.dumps(DECLARATIONS, sort_keys=True, separators=(",", ":")).encode()).hexdigest() != CLOSED_DECLARATIONS_SHA256:
        reject("closed buffer declaration vector was changed")
    texts = {page: page_text(raw, page) for page in range(81, 106)}
    for section, pages, count in WINDOWS:
        witness = SECTION_WITNESSES[section]
        if witness[1] not in texts[witness[0]] or sum(item[2] == section for item in DECLARATIONS) != count:
            reject("assigned section window is missing, widened, or reordered")
    expected = {page: [item[4] for item in DECLARATIONS if item[1] == (page,)] for page in texts}
    if any(SIGNATURE.findall(texts[page]) != expected[page] for page in texts):
        reject("sealed PDF has missing, extra, reordered, or non-exact formal declarations")
    if texts[85].count(SPAN_START) != 1 or texts[86].count(SPAN_END) != 1:
        reject("page-spanning BindBuffersBase declaration is missing or ambiguous")
    return texts


def rows(raw: bytes, grammar_rules):
    anchored_source(raw)
    if getattr(grammar_rules, "PREFIX", None) != PREFIX: reject("normalization grammar has a different C binding prefix")
    extracted, baseline = [], []
    for sequence, (fragment, pages, section, name, declaration, in_baseline) in enumerate(DECLARATIONS, 1):
        match = DECLARATION.fullmatch(declaration)
        if (match is None or match.group("name") != name or "{" in declaration or grammar_rules.normalized(declaration) != declaration):
            reject("literal declaration is malformed, templated, prefixed, or outside the sealed grammar")
        common = {"fact_id": f"command:{PREFIX}{name}", "document_name": name, "declaration": declaration,
                  "physical_page": pages[0], "source_page_span": list(pages), "numeric_section": section,
                  "source_locator": locator(pages, section), "source_sequence_position": sequence}
        if in_baseline: baseline.append(common); continue
        extracted.append({**common, "c_name": PREFIX + name, "fragment": fragment, "source_order": len(extracted) + 1})
    if len(extracted) != 38 or [item["fact_id"] for item in baseline] != ["command:glCreateBuffers"]:
        reject("baseline exclusion or raw declaration count changed")
    windows = [{"numeric_section": section, "physical_pages": list(pages), "formal_declaration_count": count}
               for section, pages, count in WINDOWS]
    return extracted, baseline, windows
