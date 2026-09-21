"""Bounded formal GLES declarations for F03.3.2.2.3.1 only."""

from __future__ import annotations

import re
import subprocess

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "generic-sync-query"
# chunk, family, section, source-family order, anchor page, anchor declaration
FAMILY_BINDINGS = (
    ("declarations", "generic-context", "2.3", 8, 31, "enum GetError( void );"),
    ("declarations", "sync", "4.1", 12, 51, "sync FenceSync( enum condition, bitfield flags );"),
    ("declarations", "query", "4.2", 13, 57, "void GenQueries( sizei n, uint *ids );"),
    ("object-declarations", "memory-barrier", "7.11.2", 16, 141, "void MemoryBarrier( bitfield barriers );"),
)
SECTION_WITNESSES = {
    "2.3": (31, "2.3. COMMAND EXECUTION"), "4.1": (51, "4.1 Sync Objects and Fences"),
    "4.2": (57, "4.2 Query Objects and Asynchronous Queries"),
    "7.11.2": (141, "7.11.2 Shader Memory Access Synchronization"),
}
# family, physical page, section, unprefixed command, exact source declaration
DECLARATIONS = (
    ("generic-context", 31, "2.3", "GetError", "enum GetError( void );"),
    ("generic-context", 34, "2.3", "GetGraphicsResetStatus", "enum GetGraphicsResetStatus( void );"),
    ("generic-context", 36, "2.3", "Flush", "void Flush( void );"),
    ("generic-context", 36, "2.3", "Finish", "void Finish( void );"),
    ("sync", 51, "4.1", "FenceSync", "sync FenceSync( enum condition, bitfield flags );"),
    ("sync", 52, "4.1", "DeleteSync", "void DeleteSync( sync sync );"),
    ("sync", 53, "4.1", "ClientWaitSync", "enum ClientWaitSync( sync sync, bitfield flags, uint64 timeout );"),
    ("sync", 54, "4.1", "WaitSync", "void WaitSync( sync sync, bitfield flags, uint64 timeout );"),
    ("sync", 56, "4.1", "GetSynciv", "void GetSynciv( sync sync, enum pname, sizei count, sizei *length, int *values );"),
    ("sync", 56, "4.1", "IsSync", "boolean IsSync( sync sync );"),
    ("query", 57, "4.2", "GenQueries", "void GenQueries( sizei n, uint *ids );"),
    ("query", 58, "4.2", "DeleteQueries", "void DeleteQueries( sizei n, const uint *ids );"),
    ("query", 58, "4.2", "BeginQuery", "void BeginQuery( enum target, uint id );"),
    ("query", 59, "4.2", "EndQuery", "void EndQuery( enum target );"),
    ("query", 60, "4.2", "IsQuery", "boolean IsQuery( uint id );"),
    ("query", 60, "4.2", "GetQueryiv", "void GetQueryiv( enum target, enum pname, int *params );"),
    ("query", 60, "4.2", "GetQueryObjectuiv", "void GetQueryObjectuiv( uint id, enum pname, uint *params );"),
    ("memory-barrier", 141, "7.11.2", "MemoryBarrier", "void MemoryBarrier( bitfield barriers );"),
    ("memory-barrier", 144, "7.11.2", "MemoryBarrierByRegion", "void MemoryBarrierByRegion( bitfield barriers );"),
)
PROTOTYPE = re.compile(r"(?:void|enum|sync|boolean) ([A-Z][A-Za-z0-9]*)\(.*\);\Z")


class CatalogError(ValueError):
    """A proposed raw declaration lies outside this sealed source slice."""


def reject(message: str) -> None:
    raise CatalogError(message)


def compact(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw,
                                capture_output=True, check=False)
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        reject(f"cannot read sealed generic-sync-query page: {error}")
    if result.returncode != 0:
        reject("cannot read sealed generic-sync-query page")
    return compact(value)


def bound_families(chunk_paths: dict[str, object], document) -> dict[str, int]:
    result: dict[str, int] = {}
    for chunk, identifier, section, order, page, text in FAMILY_BINDINGS:
        value = document(chunk_paths[chunk])
        rows = [row for row in value.get("families", []) if row.get("id") == identifier]
        expected = {"id": identifier, "family_kind": "declaration", "source_scope": [section], "route": ROUTE,
                    "reason": "formal-declaration-only", "source_order": order,
                    "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": text}}
        if len(rows) != 1 or rows[0] != expected or identifier in result:
            reject("command-domain binding is stale, incomplete, rerouted, or promoted")
        result[identifier] = order
    if set(result) != {row[1] for row in FAMILY_BINDINGS}:
        reject("command-domain binding has an unexpected family set")
    return result


def facts(raw: bytes, pages: int, family_orders: dict[str, int], normalize) -> list[dict[str, object]]:
    if pages != PAGES or set(family_orders) != {row[0] for row in DECLARATIONS}:
        reject("GLES generic-sync-query source boundary is incomplete")
    text_cache: dict[int, str] = {}
    def text(page: int) -> str:
        text_cache.setdefault(page, page_text(raw, page))
        return text_cache[page]
    result, names = [], set()
    for order, (family, page, section, name, declaration) in enumerate(DECLARATIONS, 1):
        witness = SECTION_WITNESSES.get(section)
        match = PROTOTYPE.fullmatch(declaration)
        if (witness is None or witness[1] not in text(witness[0]) or not 1 <= page <= pages
                or declaration not in text(page) or match is None or match.group(1) != name or "{" in declaration):
            reject("formal declaration or section witness is missing, malformed, or outside this source slice")
        try:
            c_names = normalize([("literal", name)])
        except Exception as error:
            reject(f"declaration grammar rejected a literal source name: {error}")
        if c_names != [f"gl{name}"] or name in names:
            reject("literal declaration is duplicate, prefixed, or outside the sealed grammar")
        names.add(name)
        result.append({"raw_id": f"gles32-generic-sync-query-{order:02d}", "family_id": family,
                       "source_family_order": family_orders[family], "unprefixed_name": name, "c_name": c_names[0],
                       "declaration": declaration, "physical_page": page, "section": section,
                       "source_locator": f"gles32-pdf-v1:page={page};section={section}", "source_order": order,
                       "derivation_class": "literal"})
    if len(result) != len(DECLARATIONS) or [row["source_order"] for row in result] != list(range(1, len(result) + 1)):
        reject("raw declaration ordering is incomplete or unstable")
    return result
