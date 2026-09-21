#!/usr/bin/env python3
"""Closed, source-only declarations for F03.2.2.5.3.1."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess

PROFILE, PAGES, PREFIX = "opengl-4.6-core", 851, "gl"
ASSIGNED_PAGES = (50, 51) + tuple(range(58, 74))
WINDOWS = (("2.6.1", (50, 51), 0, "2.6.1 Object Management"), ("4.1", tuple(range(58, 64)), 6, "4.1 Sync Objects and Fences"),
           ("4.2", tuple(range(64, 73)), 18, "4.2 Query Objects and Asynchronous Queries"), ("4.3", (73,), 1, "4.3 Time Queries"))
# document name, page, numeric section, formal declaration, readable fragment, returned-state query, F03.2.2.2 baseline
DECLARATIONS = (
    ("FenceSync", 58, "4.1", "sync FenceSync( enum condition, bitfield flags );", "sync", False, True),
    ("DeleteSync", 59, "4.1", "void DeleteSync( sync sync );", "sync", False, False),
    ("ClientWaitSync", 60, "4.1.1", "enum ClientWaitSync( sync sync, bitfield flags, uint64 timeout );", "sync", False, False),
    ("WaitSync", 61, "4.1.1", "void WaitSync( sync sync, bitfield flags, uint64 timeout );", "sync", False, False),
    ("GetSynciv", 63, "4.1.3", "void GetSynciv( sync sync, enum pname, sizei count, sizei *length, int *values );", "sync", True, False),
    ("IsSync", 63, "4.1.3", "boolean IsSync( sync sync );", "sync", True, False),
    ("GenQueries", 65, "4.2.2", "void GenQueries( sizei n, uint *ids );", "query-lifecycle", False, False),
    ("CreateQueries", 66, "4.2.2", "void CreateQueries( enum target, sizei n, uint *ids );", "query-lifecycle", False, True),
    ("DeleteQueries", 66, "4.2.2", "void DeleteQueries( sizei n, const uint *ids );", "query-lifecycle", False, False),
    ("BeginQueryIndexed", 67, "4.2.2", "void BeginQueryIndexed( enum target, uint index, uint id );", "query-lifecycle", False, False),
    ("BeginQuery", 68, "4.2.2", "void BeginQuery( enum target, uint id );", "query-lifecycle", False, False),
    ("EndQueryIndexed", 68, "4.2.2", "void EndQueryIndexed( enum target, uint index );", "query-lifecycle", False, False),
    ("EndQuery", 69, "4.2.2", "void EndQuery( enum target );", "query-lifecycle", False, False),
    ("IsQuery", 69, "4.2.3", "boolean IsQuery( uint id );", "query-lifecycle", True, False),
    ("GetQueryIndexediv", 69, "4.2.3", "void GetQueryIndexediv( enum target, uint index, enum pname, int *params );", "query-state", True, False),
    ("GetQueryiv", 71, "4.2.3", "void GetQueryiv( enum target, enum pname, int *params );", "query-state", True, False),
    ("GetQueryObjectiv", 71, "4.2.3", "void GetQueryObjectiv( uint id, enum pname, int *params );", "query-state", True, False),
    ("GetQueryObjectuiv", 71, "4.2.3", "void GetQueryObjectuiv( uint id, enum pname, uint *params );", "query-state", True, False),
    ("GetQueryObjecti64v", 71, "4.2.3", "void GetQueryObjecti64v( uint id, enum pname, int64 *params );", "query-state", True, False),
    ("GetQueryObjectui64v", 71, "4.2.3", "void GetQueryObjectui64v( uint id, enum pname, uint64 *params );", "query-state", True, False),
    ("GetQueryBufferObjectiv", 71, "4.2.3", "void GetQueryBufferObjectiv( uint id, uint buffer, enum pname, intptr offset );", "query-buffer-state", True, False),
    ("GetQueryBufferObjectuiv", 71, "4.2.3", "void GetQueryBufferObjectuiv( uint id, uint buffer, enum pname, intptr offset );", "query-buffer-state", True, False),
    ("GetQueryBufferObjecti64v", 71, "4.2.3", "void GetQueryBufferObjecti64v( uint id, uint buffer, enum pname, intptr offset );", "query-buffer-state", True, False),
    ("GetQueryBufferObjectui64v", 71, "4.2.3", "void GetQueryBufferObjectui64v( uint id, uint buffer, enum pname, intptr offset );", "query-buffer-state", True, False),
    ("QueryCounter", 73, "4.3", "void QueryCounter( uint id, enum target );", "query-lifecycle", False, False),
)
CLOSED_DECLARATIONS_SHA256 = "6edeca792c020cb02bb3750313b19d173b590252839431bd82a448e82ddc6367"
SIGNATURE = re.compile(r"(?<![A-Za-z0-9_])(?:void|enum|boolean|sync) [A-Z][A-Za-z0-9_]*\(.*?\);")


class RuleError(ValueError):
    """An assigned source window is incomplete, guessed, or out of profile."""


def reject(message: str) -> None:
    raise RuleError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def digest(value: object) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def normalized(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw, capture_output=True, check=False)
        text = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        reject(f"cannot read sealed PDF page: {error}")
    if result.returncode:
        reject("cannot read sealed PDF page")
    return normalized(text)


def anchored_source(raw: bytes) -> dict[int, str]:
    if digest(DECLARATIONS) != CLOSED_DECLARATIONS_SHA256:
        reject("closed declaration vector was changed")
    texts = {page: page_text(raw, page) for page in ASSIGNED_PAGES}
    for section, pages, count, heading in WINDOWS:
        if heading not in texts[pages[0]] or sum(len([item for item in DECLARATIONS if item[1] == page]) for page in pages) != count:
            reject("assigned section window is missing, widened, or reordered")
    for page in ASSIGNED_PAGES:
        expected = [item[3] for item in DECLARATIONS if item[1] == page]
        if SIGNATURE.findall(texts[page]) != expected:
            reject("sealed PDF has missing, extra, reordered, or non-exact formal declarations")
    return texts


def rows(raw: bytes, grammar_rules) -> tuple[list[dict[str, object]], list[dict[str, object]], list[str]]:
    anchored_source(raw)
    if getattr(grammar_rules, "PREFIX", None) != PREFIX:
        reject("normalization grammar has a different C binding prefix")
    extracted, baseline, deferred = [], [], []
    for sequence, (name, page, section, declaration, fragment, state_query, in_baseline) in enumerate(DECLARATIONS, 1):
        try:
            text, _, parsed_name, _ = grammar_rules.parsed(declaration, False)
        except Exception as error:
            reject(f"grammar rejects source declaration {name}: {error}")
        if text != declaration or parsed_name != name:
            reject("grammar changed a literal source declaration")
        locator, fact_id = f"opengl46-core-pdf-v1:page={page};section={section}", f"command:{PREFIX}{name}"
        common = {"fact_id": fact_id, "document_name": name, "declaration": declaration, "physical_page": page,
                  "numeric_section": section, "source_locator": locator, "source_sequence_position": sequence}
        if in_baseline:
            baseline.append(common); continue
        extracted.append({**common, "c_name": PREFIX + name, "fragment": fragment, "source_order": len(extracted) + 1})
        if state_query: deferred.append(fact_id)
    if len(extracted) != 23 or [item["fact_id"] for item in baseline] != ["command:glFenceSync", "command:glCreateQueries"]:
        reject("baseline exclusions or extracted declaration count changed")
    return extracted, baseline, deferred
