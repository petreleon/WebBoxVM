#!/usr/bin/env python3
"""Anchored, bounded source rules for F03.2.2.2 raw facts."""

from __future__ import annotations

import json
import re
import subprocess

PROFILE, PAGES = "opengl-4.6-core", 851
LOCATOR_SYNTAX = "opengl46-core-pdf-v1:page=<positive-decimal>;section=<section-path>"
LOCATOR = re.compile(r"^opengl46-core-pdf-v1:page=[1-9][0-9]*;section=[1-9][0-9]*(?:\.[1-9][0-9]*)*$")
PREFIX_LOCATOR = "opengl46-core-pdf-v1:page=32;section=2.2"
OBJECTS = (
    ("buffer", "buffer-object", 51, "2.6.2", "Buffer Objects"),
    ("shader", "shader-object", 51, "2.6.3", "Shader Objects"),
    ("program", "program-object", 51, "2.6.4", "Program Objects"),
    ("program-pipeline", "program-pipeline-object", 52, "2.6.5", "Program Pipeline Objects"),
    ("texture", "texture-object", 52, "2.6.6", "Texture Objects"),
    ("sampler", "sampler-object", 52, "2.6.7", "Sampler Objects"),
    ("renderbuffer", "renderbuffer-object", 53, "2.6.8", "Renderbuffer Objects"),
    ("framebuffer", "framebuffer-object", 53, "2.6.9", "Framebuffer Objects"),
    ("vertex-array", "vertex-array-object", 53, "2.6.10", "Vertex Array Objects"),
    ("transform-feedback", "transform-feedback-object", 53, "2.6.11", "Transform Feedback Objects"),
    ("query", "query-object", 54, "2.6.12", "Query Objects"),
    ("sync", "sync-object", 54, "2.6.13", "Sync Objects"),
)
COMMANDS = (
    ("sync", "FenceSync", 58, "4.1", "sync FenceSync( enum condition, bitfield flags );"),
    ("query", "CreateQueries", 66, "4.2.2", "void CreateQueries( enum target, sizei n, uint *ids );"),
    ("buffer", "CreateBuffers", 81, "6.1", "void CreateBuffers( sizei n, uint *buffers );"),
    ("shader", "CreateShader", 110, "7.1", "uint CreateShader( enum type );"),
    ("program", "CreateProgram", 116, "7.3", "uint CreateProgram( void );"),
    ("program-pipeline", "CreateProgramPipelines", 145, "7.4", "void CreateProgramPipelines( sizei n, uint *pipelines );"),
    ("texture", "CreateTextures", 204, "8.1", "void CreateTextures( enum target, sizei n, uint *textures );"),
    ("sampler", "CreateSamplers", 206, "8.2", "void CreateSamplers( sizei n, uint *samplers );"),
    ("framebuffer", "CreateFramebuffers", 324, "9.2", "void CreateFramebuffers( sizei n, uint *framebuffers );"),
    ("renderbuffer", "CreateRenderbuffers", 335, "9.2.4", "void CreateRenderbuffers( sizei n, uint *renderbuffers );"),
    ("vertex-array", "CreateVertexArrays", 373, "10.3.1", "void CreateVertexArrays( sizei n, uint *arrays );"),
    ("transform-feedback", "CreateTransformFeedbacks", 466, "13.3.1", "void CreateTransformFeedbacks( sizei n, uint *ids );"),
)
EXCLUDED = ("all-openGL-command-universe-not-inferred", "state-and-lifecycle-deferred-to-F03.2.2.3",
            "limits-and-formats-deferred-to-F03.2.3", "compatibility-profile-excluded",
            "extensions-excluded", "shader-language-semantics-unadmitted")


class RuleError(ValueError):
    """A PDF anchor is not the bounded source rule this leaf admits."""


def reject(message: str) -> None:
    raise RuleError(message)


def locator(page: int, section: str) -> str:
    value = f"opengl46-core-pdf-v1:page={page};section={section}"
    if not LOCATOR.fullmatch(value):
        reject("rule has an invalid F03.2.1 PDF locator")
    return value


def condition(anchor: str, section: str, row: str, declaration: str | None = None) -> dict[str, object]:
    value: dict[str, object] = {"anchor": anchor, "section": section, "table": "none", "row": row}
    if declaration is not None:
        value.update(declaration=declaration, c_binding_prefix="gl")
    return value


def facts() -> list[dict[str, object]]:
    rows = []
    for family, name, page, section, row in OBJECTS:
        rows.append({"fact_id": f"object:{family}", "fact_kind": "object-taxonomy", "family": family,
                     "name": name, "source_locator": locator(page, section), "physical_page": page,
                     "source_order": len(rows) + 1, "condition": condition("object-taxonomy-subsection", section, row)})
    for family, bare, page, section, declaration in COMMANDS:
        rows.append({"fact_id": f"command:gl{bare}", "fact_kind": "command-declaration", "family": family,
                     "name": f"gl{bare}", "source_locator": locator(page, section), "physical_page": page,
                     "source_order": len(rows) + 1,
                     "condition": condition("formal-command-declaration", section, bare, declaration)})
    return rows


def coverage() -> dict[str, object]:
    names = {family: (f"object:{family}", f"command:gl{bare}") for family, bare, *_ in COMMANDS}
    return {"manifest_id": "opengl46-core-bounded-command-object-v1", "source_locator_syntax": LOCATOR_SYNTAX,
            "coverage_decision": "complete-for-listed-object-families-only", "excluded": list(EXCLUDED),
            "prefix_rule": {"source_locator": PREFIX_LOCATOR, "physical_page": 32,
                            "condition": condition("c-binding-prefix-prose", "2.2", "command names", None)},
            "families": [{"id": family, "object_fact_id": f"object:{family}", "command_fact_id": names[family][1]}
                         for family, *_ in OBJECTS]}


def normalized(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-layout", "-", "-"],
                                input=raw, capture_output=True, check=False)
    except OSError as error:
        reject(f"pdftotext is required for anchored PDF extraction: {error}")
    if result.returncode != 0:
        reject("anchored PDF page extraction failed")
    try:
        return normalized(result.stdout.decode("utf-8"))
    except UnicodeDecodeError as error:
        reject(f"anchored PDF text is not UTF-8: {error}")


def verify_anchors(raw: bytes) -> tuple[list[dict[str, object]], dict[str, object]]:
    pages: dict[int, str] = {}
    def text(page: int) -> str:
        pages.setdefault(page, page_text(raw, page))
        return pages[page]
    if "command names, constants, and types are prefixed in the C language binding" not in text(32):
        reject("C-binding gl prefix rule is not anchored at physical page 32")
    rows = facts()
    for row in rows:
        details, page = row["condition"], row["physical_page"]
        needle = details.get("declaration", f"{details['section']} {details['row']}")
        if not isinstance(needle, str) or needle not in text(page):
            reject(f"source anchor is absent for {row['fact_id']}")
    return rows, coverage()


def packed(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")
