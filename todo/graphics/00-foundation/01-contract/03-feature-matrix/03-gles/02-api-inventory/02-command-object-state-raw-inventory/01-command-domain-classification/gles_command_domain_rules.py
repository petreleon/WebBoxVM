"""Closed source-family rules for F03.3.2.2.1; they are not API facts."""

from __future__ import annotations

import subprocess

PROFILE, PAGES = "gles-3.2", 601
ROUTES = {
    "declaration-grammar": ("F03.3.2.2.2", "grammar"),
    "generic-sync-query": ("F03.3.2.2.3.1", "declaration"),
    "buffer-commands": ("F03.3.2.2.3.2", "declaration"),
    "program-pipeline-commands": ("F03.3.2.2.3.3", "declaration"),
    "texture-sampler-commands": ("F03.3.2.2.3.4", "declaration"),
    "framebuffer-renderbuffer-commands": ("F03.3.2.2.3.5", "declaration"),
    "vertex-transform-feedback-commands": ("F03.3.2.2.3.6", "declaration"),
    "context-state-lifecycle": ("F03.3.2.2.4.1", "state"),
    "draw-raster-commands": ("F03.3.2.2.4.2", "declaration"),
    "pixel-transfer-commands": ("F03.3.2.2.4.3", "declaration"),
    "debug-special-query-commands": ("F03.3.2.2.4.4", "declaration"),
    "limit-format-inventory": ("F03.3.2.3", "separate-inventory"),
    "shader-unavailable": ("F03.3.2.4:shader", "unavailable"),
    "precision-unavailable": ("F03.3.2.4:precision", "unavailable"),
    "extension-unavailable": ("F03.3.2.4:extension", "unavailable"),
    "explicit-exclusion": ("F03.3.1", "excluded"),
}
# chunk, family kind, id, exact source scopes, route, anchor kind, page, section, anchor text, reason
FAMILIES = (
    ("state-lifecycle", "source-route", "shader-language", "1.6.1", "shader-unavailable", "unavailable", 0, "", "shader", "unadmitted-distinct-source"),
    ("state-lifecycle", "source-route", "precision-language", "external:essl-precision", "precision-unavailable", "unavailable", 0, "", "precision", "unadmitted-distinct-source"),
    ("exclusions", "exclusion", "webgl", "1.6.2", "explicit-exclusion", "heading", 22, "1.6.2", "1.6.2 WebGL", "external-web-api-not-admitted-core-source"),
    ("exclusions", "exclusion", "window-system", "1.6.3", "explicit-exclusion", "heading", 22, "1.6.3", "1.6.3 Window System Bindings", "outside-core-command-state-source"),
    ("exclusions", "exclusion", "opencl", "1.6.4", "explicit-exclusion", "heading", 22, "1.6.4", "1.6.4 OpenCL", "external-compute-api-not-admitted-core-source"),
    ("state-lifecycle", "state-route", "fundamentals", "2.1", "context-state-lifecycle", "heading", 24, "2.1", "2.1 OpenGL ES Fundamentals", "foundational-description-not-declaration"),
    ("declarations", "grammar", "command-syntax", "2.2", "declaration-grammar", "grammar", 27, "2.2", "void Uniform{1234}{if}( int location, T value );", "grammar-not-command-fact"),
    ("declarations", "declaration", "generic-context", "2.3", "generic-sync-query", "declaration", 31, "2.3", "enum GetError( void );", "formal-declaration-only"),
    ("state-lifecycle", "state-route", "context-and-lifecycle", "2.4-2.5;5", "context-state-lifecycle", "heading", 42, "2.5", "2.5 Context State", "state-rules-not-declarations"),
    ("state-lifecycle", "state-route", "object-taxonomy", "2.6", "context-state-lifecycle", "heading", 43, "2.6", "2.6 Objects and the Object Model", "object-taxonomy-not-declarations"),
    ("exclusions", "exclusion", "dataflow-description", "3", "explicit-exclusion", "heading", 48, "3", "Chapter 3 Dataflow Model", "descriptive-model-not-source-fact"),
    ("declarations", "declaration", "sync", "4.1", "generic-sync-query", "declaration", 51, "4.1", "sync FenceSync( enum condition, bitfield flags );", "formal-declaration-only"),
    ("declarations", "declaration", "query", "4.2", "generic-sync-query", "declaration", 57, "4.2", "void GenQueries( sizei n, uint *ids );", "formal-declaration-only"),
    ("declarations", "declaration", "buffer", "6", "buffer-commands", "declaration", 68, "6", "void GenBuffers( sizei n, uint *buffers );", "formal-declaration-only"),
    ("object-declarations", "declaration", "program-shader", "7.1-7.11.1;7.11.3-7.13.1;7.13.3-7.15", "program-pipeline-commands", "declaration", 85, "7.1", "uint CreateShader( enum type );", "formal-declaration-only"),
    ("object-declarations", "declaration", "memory-barrier", "7.11.2", "generic-sync-query", "declaration", 141, "7.11.2", "void MemoryBarrier( bitfield barriers );", "formal-declaration-only"),
    ("object-declarations", "declaration", "texture", "8.1;8.5-8.26", "texture-sampler-commands", "declaration", 157, "8.1", "void GenTextures( sizei n, uint *textures );;", "formal-declaration-only"),
    ("object-declarations", "declaration", "sampler", "8.2", "texture-sampler-commands", "declaration", 159, "8.2", "void GenSamplers( sizei count, uint *samplers );", "formal-declaration-only"),
    ("execution", "declaration", "pixel-store-and-transfer", "8.4;16", "pixel-transfer-commands", "declaration", 426, "16.1", "void ReadPixels( int x, int y, sizei width, sizei height, enum format, enum type, void *data );", "formal-declaration-only"),
    ("object-declarations", "declaration", "framebuffer-renderbuffer", "9", "framebuffer-renderbuffer-commands", "declaration", 244, "9.2", "void GenFramebuffers( sizei n, uint *framebuffers );", "formal-declaration-only"),
    ("state-lifecycle", "state-route", "vertex-remaining-state", "10.1;10.6-10.7", "context-state-lifecycle", "heading", 274, "10", "Chapter 10 Vertex Specification and Drawing Commands", "state-and-query-route"),
    ("object-declarations", "declaration", "vertex-array", "10.2;10.3.1-10.3.8;10.4", "vertex-transform-feedback-commands", "declaration", 294, "10.4", "void GenVertexArrays( sizei n, uint *arrays );", "formal-declaration-only"),
    ("execution", "declaration", "draw", "10.5", "draw-raster-commands", "declaration", 296, "10.5", "void DrawArrays( enum mode, int first, sizei count );", "formal-declaration-only"),
    ("execution", "declaration", "indirect-draw", "10.3.9", "draw-raster-commands", "heading", 293, "10.3.9", "10.3.9 Indirect Commands in Buffer Objects", "formal-declaration-only"),
    ("state-lifecycle", "state-route", "programmable-vertex-stage", "11", "context-state-lifecycle", "heading", 306, "11", "Chapter 11 Programmable Vertex Processing", "state-route-not-essl-source"),
    ("object-declarations", "declaration", "transform-feedback", "12.2", "vertex-transform-feedback-commands", "declaration", 357, "12.2", "void BeginTransformFeedback( enum primitiveMode );", "formal-declaration-only"),
    ("state-lifecycle", "state-route", "post-vertex-state", "12.1;12.3-12.6", "context-state-lifecycle", "heading", 353, "12", "Chapter 12 Fixed-Function Vertex Post-Processing", "state-rules-not-declarations"),
    ("execution", "declaration", "raster-and-framebuffer", "13;15", "draw-raster-commands", "declaration", 418, "15.2.3", "void Clear( bitfield buf );", "formal-declaration-only"),
    ("state-lifecycle", "state-route", "programmable-fragment-stage", "14", "context-state-lifecycle", "heading", 389, "14", "Chapter 14 Programmable Fragment Processing", "state-route-not-essl-source"),
    ("execution", "declaration", "compute", "17", "draw-raster-commands", "declaration", 439, "17", "void DispatchCompute( uint num groups x, uint num groups y, uint num groups z );", "formal-declaration-only"),
    ("execution", "declaration", "debug", "18", "debug-special-query-commands", "declaration", 445, "18.2", "void DebugMessageCallback( DEBUGPROC callback, const void *userParam );", "formal-declaration-only"),
    ("execution", "declaration", "special", "19", "debug-special-query-commands", "declaration", 455, "19.1", "void Hint( enum target, enum hint );", "formal-declaration-only"),
    ("execution", "declaration", "context-queries", "20", "debug-special-query-commands", "declaration", 457, "20.1", "void GetBooleanv( enum pname, boolean *data );", "formal-declaration-only"),
    ("state-sources", "state-route", "state-tables", "21.1-21.39", "context-state-lifecycle", "heading", 464, "21", "Chapter 21 State Tables", "table-rules-require-triggered-fact"),
    ("state-sources", "separate-inventory", "limit-format-tables", "21.40-21.42", "limit-format-inventory", "limit", 504, "21.40", "Table 21.40: Implementation Dependent Values", "separate-bounded-limit-format-inventory"),
    ("state-sources", "separate-inventory", "unreviewed-limit-format-tables", "21.43-21.57", "limit-format-inventory", "limit", 507, "21.43", "Table 21.43: Implementation Dependent Version and Extension Support", "unreviewed-outside-bounded-limit-format-inventory"),
    ("state-sources", "separate-inventory", "format-appendix", "appendix-C", "limit-format-inventory", "heading", 531, "C", "Appendix C Compressed Texture Image Formats", "separate-format-source-route"),
    ("state-sources", "source-route", "extension-source", "external:extension-source", "extension-unavailable", "unavailable", 0, "", "extension", "unadmitted-distinct-source"),
    ("exclusions", "exclusion", "invariance-appendix", "appendix-A", "explicit-exclusion", "heading", 522, "A", "Appendix A Invariance", "non-command-appendix-not-source-fact"),
    ("exclusions", "exclusion", "corollaries-appendix", "appendix-B", "explicit-exclusion", "heading", 529, "B", "Appendix B Corollaries", "non-command-appendix-not-source-fact"),
    ("exclusions", "exclusion", "historical-profiles", "appendix-D;appendix-E;appendix-F;appendix-G", "explicit-exclusion", "heading", 534, "D", "Appendix D Version 3.0 and Before", "not-gles-32-core-source-scope"),
    ("exclusions", "exclusion", "desktop-gl", "external:desktop-gl", "explicit-exclusion", "external", 0, "", "desktop-gl", "cross-profile-source-not-admitted"),
    ("exclusions", "exclusion", "registry-header", "external:gl.xml", "explicit-exclusion", "external", 0, "", "gl.xml", "registry-not-admitted-normative-source"),
    ("exclusions", "exclusion", "pdf-index", "index", "explicit-exclusion", "heading", 571, "Index", "Index", "omission-crosscheck-not-fact-source"),
)


class RuleError(ValueError):
    """A source family is not closed, anchored, or safely routed."""


def reject(message: str) -> None:
    raise RuleError(message)


def canonical_text(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw,
                                capture_output=True, check=False)
        text = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        reject(f"cannot read sealed PDF anchor: {error}")
    if result.returncode != 0:
        reject("cannot read sealed PDF anchor")
    return canonical_text(text)


def family_rows(raw: bytes, pages: int, limit: dict[str, object], ledger: dict[str, object], crosschecks) -> dict[str, list[dict[str, object]]]:
    crosschecks.locked(FAMILIES, ROUTES, reject)
    rows, identifiers, routes = {"declarations": [], "object-declarations": [], "execution": [], "state-lifecycle": [], "state-sources": [], "exclusions": []}, set(), set()
    unavailable = {item["id"] for item in ledger["unavailable_classes"]}
    for order, (chunk, kind, identifier, scope, route, anchor_kind, page, section, text, reason) in enumerate(FAMILIES, 1):
        if chunk not in rows or identifier in identifiers or route not in ROUTES or not reason:
            reject("family has an unknown chunk, duplicate identity, or implicit route")
        identifiers.add(identifier); routes.add(route)
        anchor = {"kind": anchor_kind}
        if anchor_kind in {"declaration", "grammar", "heading", "limit"}:
            crosschecks.exact_pdf_anchor(anchor_kind, page, section, text, pages, lambda value: page_text(raw, value), reject)
            anchor.update(physical_page=page, section=section, text=text)
        elif anchor_kind == "unavailable":
            if text not in unavailable:
                reject("unavailable ledger does not expose the required source class")
            anchor.update(class_id=text, ledger_sha256=ledger["ledger_sha256"])
        elif anchor_kind == "external":
            anchor.update(source_id=text)
        else:
            reject("family uses an unrecognized anchor kind")
        if anchor_kind == "limit":
            anchor["inventory_sha256"] = limit["inventory_sha256"]
        rows[chunk].append({"id": identifier, "family_kind": kind, "source_scope": list(crosschecks.scope_parts(scope, reject)),
                            "route": route, "reason": reason, "source_order": order, "anchor": anchor})
    if routes != set(ROUTES) or len(identifiers) != len(FAMILIES):
        reject("closed family map has an unused route or missing identity")
    return rows
