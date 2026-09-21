#!/usr/bin/env python3
"""Closed source declarations for F03.2.2.5.4.2; no execution semantics."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess

PROFILE, PAGES, PREFIX = "opengl-4.6-core", 851, "gl"
ASSIGNED_PAGES = tuple(range(386, 399)) + (472, 568, 569)
WINDOWS = (
    ("10.4", tuple(range(386, 399)), 22, 20, "10.4 Drawing Commands Using Vertex Arrays"),
    ("13.3.3", (472,), 4, 4, "13.3.3 Transform Feedback Draw Operations"),
    ("19", (568, 569), 2, 2, "Chapter 19"),
)
# name, physical page, section, literal declaration, artifact fragment, actual GL command
DECLARATIONS = (
    ("DrawArraysOneInstance", 386, "10.4", "void DrawArraysOneInstance( enum mode, int first, sizei count, int instance, uint baseinstance );", "", False),
    ("DrawArrays", 388, "10.4", "void DrawArrays( enum mode, int first, sizei count );", "array-basic", True),
    ("DrawArraysInstancedBaseInstance", 388, "10.4", "void DrawArraysInstancedBaseInstance( enum mode, int first, sizei count, sizei instancecount, uint baseinstance );", "array-basic", True),
    ("DrawArraysInstanced", 388, "10.4", "void DrawArraysInstanced( enum mode, int first, sizei count, sizei instancecount );", "array-basic", True),
    ("DrawArraysIndirect", 389, "10.4", "void DrawArraysIndirect( enum mode, const void *indirect );", "array-basic", True),
    ("MultiDrawArrays", 389, "10.4", "void MultiDrawArrays( enum mode, const int *first, const sizei *count, sizei drawcount );", "array-basic", True),
    ("MultiDrawArraysIndirect", 390, "10.4", "void MultiDrawArraysIndirect( enum mode, const void *indirect, sizei drawcount, sizei stride );", "array-basic", True),
    ("MultiDrawArraysIndirectCount", 391, "10.4", "void MultiDrawArraysIndirectCount( enum mode, const void *indirect, intptr drawcount, intptr maxdrawcount, sizei stride );", "array-basic", True),
    ("DrawElementsOneInstance", 391, "10.4", "void DrawElementsOneInstance( enum mode, sizei count, enum type, const void *indices, int instance, int basevertex, uint baseinstance );", "", False),
    ("DrawElements", 393, "10.4", "void DrawElements( enum mode, sizei count, enum type, const void *indices );", "element-basic", True),
    ("DrawElementsInstancedBaseInstance", 393, "10.4", "void DrawElementsInstancedBaseInstance( enum mode, sizei count, enum type, const void *indices, sizei instancecount, uint baseinstance );", "element-basic", True),
    ("DrawElementsInstanced", 394, "10.4", "void DrawElementsInstanced( enum mode, sizei count, enum type, const void *indices, sizei instancecount );", "element-basic", True),
    ("MultiDrawElements", 394, "10.4", "void MultiDrawElements( enum mode, const sizei *count, enum type, const void * const *indices, sizei drawcount );", "element-basic", True),
    ("DrawRangeElements", 394, "10.4", "void DrawRangeElements( enum mode, uint start, uint end, sizei count, enum type, const void *indices );", "element-basic", True),
    ("DrawElementsBaseVertex", 395, "10.4", "void DrawElementsBaseVertex( enum mode, sizei count, enum type, const void *indices, int basevertex );", "element-basic", True),
    ("DrawRangeElementsBaseVertex", 395, "10.4", "void DrawRangeElementsBaseVertex( enum mode, uint start, uint end, sizei count, enum type, const void *indices, int basevertex );", "element-advanced", True),
    ("DrawElementsInstancedBaseVertex", 395, "10.4", "void DrawElementsInstancedBaseVertex( enum mode, sizei count, enum type, const void *indices, sizei instancecount, int basevertex );", "element-advanced", True),
    ("DrawElementsInstancedBaseVertexBaseInstance", 395, "10.4", "void DrawElementsInstancedBaseVertexBaseInstance( enum mode, sizei count, enum type, const void *indices, sizei instancecount, int basevertex, uint baseinstance );", "element-advanced", True),
    ("DrawElementsIndirect", 396, "10.4", "void DrawElementsIndirect( enum mode, enum type, const void *indirect );", "element-advanced", True),
    ("MultiDrawElementsIndirect", 396, "10.4", "void MultiDrawElementsIndirect( enum mode, enum type, const void *indirect, sizei drawcount, sizei stride );", "element-advanced", True),
    ("MultiDrawElementsIndirectCount", 397, "10.4", "void MultiDrawElementsIndirectCount( enum mode, enum type, const void *indirect, intptr drawcount, sizei maxdrawcount, sizei stride );", "element-advanced", True),
    ("MultiDrawElementsBaseVertex", 398, "10.4", "void MultiDrawElementsBaseVertex( enum mode, const sizei *count, enum type, const void * const *indices, sizei drawcount, const int *basevertex );", "element-advanced", True),
    ("DrawTransformFeedback", 472, "13.3.3", "void DrawTransformFeedback( enum mode, uint id );", "transform-feedback", True),
    ("DrawTransformFeedbackInstanced", 472, "13.3.3", "void DrawTransformFeedbackInstanced( enum mode, uint id, sizei instancecount );", "transform-feedback", True),
    ("DrawTransformFeedbackStream", 472, "13.3.3", "void DrawTransformFeedbackStream( enum mode, uint id, uint stream );", "transform-feedback", True),
    ("DrawTransformFeedbackStreamInstanced", 472, "13.3.3", "void DrawTransformFeedbackStreamInstanced( enum mode, uint id, uint stream, sizei instancecount );", "transform-feedback", True),
    ("DispatchCompute", 568, "19", "void DispatchCompute( uint num groups x, uint num groups y, uint num groups z );", "compute", True),
    ("DispatchComputeIndirect", 569, "19", "void DispatchComputeIndirect( intptr indirect );", "compute", True),
)
CLOSED_DECLARATIONS_SHA256 = "463227564514d39dfc7fcfc37792373f034c2fe11b120137ce55625121ed2036"
REJECTIONS = ("registry-header-lower-profile-extension-compatibility", "guessed-template", "unanchored-or-ambiguous-declaration", "non-gl-pseudocommand", "primitive-assembly-compute-state-output-semantics")
SIGNATURE = re.compile(r"(?<![A-Za-z0-9_])void (?:MultiDraw|Draw|Dispatch)[A-Za-z0-9_]*\(.*?\);")


class RuleError(ValueError):
    """An assigned declaration window is missing, widened, or untrusted."""


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
    for section, pages, formal, actual, heading in WINDOWS:
        rows = [item for item in DECLARATIONS if item[1] in pages]
        if heading not in texts[pages[0]] or len(rows) != formal or sum(item[5] for item in rows) != actual:
            reject("assigned section window is missing, widened, or reordered")
    for page in ASSIGNED_PAGES:
        expected = [item[3] for item in DECLARATIONS if item[1] == page]
        if SIGNATURE.findall(texts[page]) != expected:
            reject("sealed PDF has missing, extra, reordered, or non-exact formal declarations")
    return texts


def rows(raw: bytes, grammar_rules) -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    anchored_source(raw)
    if getattr(grammar_rules, "PREFIX", None) != PREFIX:
        reject("normalization grammar has a different C binding prefix")
    extracted, exclusions = [], []
    for sequence, (name, page, section, declaration, fragment, actual) in enumerate(DECLARATIONS, 1):
        try:
            text, _, parsed_name, _ = grammar_rules.parsed(declaration, False)
        except Exception as error:
            reject(f"grammar rejects source declaration {name}: {error}")
        if text != declaration or parsed_name != name:
            reject("grammar changed a literal source declaration")
        common = {"document_name": name, "declaration": declaration, "physical_page": page, "numeric_section": section,
                  "source_locator": f"opengl46-core-pdf-v1:page={page};section={section}", "source_sequence_position": sequence}
        if not actual:
            exclusions.append({**common, "exclusion": "document-not-a-gl-command"})
        else:
            extracted.append({**common, "fact_id": f"command:{PREFIX}{name}", "c_name": PREFIX + name,
                              "fragment": fragment, "source_order": len(extracted) + 1})
    if (len(extracted), [item["document_name"] for item in exclusions]) != (26, ["DrawArraysOneInstance", "DrawElementsOneInstance"]):
        reject("actual declaration count or non-GL formal exclusions changed")
    return extracted, exclusions
