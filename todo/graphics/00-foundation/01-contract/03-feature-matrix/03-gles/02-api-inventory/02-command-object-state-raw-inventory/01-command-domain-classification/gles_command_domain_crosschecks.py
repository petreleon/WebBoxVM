#!/usr/bin/env python3
"""Independent closure, scope, and index checks for F03.3.2.2.1."""

from __future__ import annotations

import re
import subprocess

SECTION = re.compile(r"^[1-9][0-9]*(?:\.[1-9][0-9]*)*(?:-[1-9][0-9]*(?:\.[1-9][0-9]*)*)?$")
EXTERNAL = re.compile(r"^external:[a-z0-9][a-z0-9.-]*$")
DECLARATION = re.compile(r"^(?:void|enum|sync|uint)\s+[A-Za-z_][A-Za-z0-9_]*\([^;]*\);{1,2}$")

# This is intentionally independent of RULES.FAMILIES; it locks source work, not API facts.
CLOSED_FAMILIES = (
    ('state-lifecycle', 'source-route', 'shader-language', '1.6.1', 'shader-unavailable', 'unavailable', 0, '', 'shader', 'unadmitted-distinct-source'),
    ('state-lifecycle', 'source-route', 'precision-language', 'external:essl-precision', 'precision-unavailable', 'unavailable', 0, '', 'precision', 'unadmitted-distinct-source'),
    ('exclusions', 'exclusion', 'webgl', '1.6.2', 'explicit-exclusion', 'heading', 22, '1.6.2', '1.6.2 WebGL', 'external-web-api-not-admitted-core-source'),
    ('exclusions', 'exclusion', 'window-system', '1.6.3', 'explicit-exclusion', 'heading', 22, '1.6.3', '1.6.3 Window System Bindings', 'outside-core-command-state-source'),
    ('exclusions', 'exclusion', 'opencl', '1.6.4', 'explicit-exclusion', 'heading', 22, '1.6.4', '1.6.4 OpenCL', 'external-compute-api-not-admitted-core-source'),
    ('state-lifecycle', 'state-route', 'fundamentals', '2.1', 'context-state-lifecycle', 'heading', 24, '2.1', '2.1 OpenGL ES Fundamentals', 'foundational-description-not-declaration'),
    ('declarations', 'grammar', 'command-syntax', '2.2', 'declaration-grammar', 'grammar', 27, '2.2', 'void Uniform{1234}{if}( int location, T value );', 'grammar-not-command-fact'),
    ('declarations', 'declaration', 'generic-context', '2.3', 'generic-sync-query', 'declaration', 31, '2.3', 'enum GetError( void );', 'formal-declaration-only'),
    ('state-lifecycle', 'state-route', 'context-and-lifecycle', '2.4-2.5;5', 'context-state-lifecycle', 'heading', 42, '2.5', '2.5 Context State', 'state-rules-not-declarations'),
    ('state-lifecycle', 'state-route', 'object-taxonomy', '2.6', 'context-state-lifecycle', 'heading', 43, '2.6', '2.6 Objects and the Object Model', 'object-taxonomy-not-declarations'),
    ('exclusions', 'exclusion', 'dataflow-description', '3', 'explicit-exclusion', 'heading', 48, '3', 'Chapter 3 Dataflow Model', 'descriptive-model-not-source-fact'),
    ('declarations', 'declaration', 'sync', '4.1', 'generic-sync-query', 'declaration', 51, '4.1', 'sync FenceSync( enum condition, bitfield flags );', 'formal-declaration-only'),
    ('declarations', 'declaration', 'query', '4.2', 'generic-sync-query', 'declaration', 57, '4.2', 'void GenQueries( sizei n, uint *ids );', 'formal-declaration-only'),
    ('declarations', 'declaration', 'buffer', '6', 'buffer-commands', 'declaration', 68, '6', 'void GenBuffers( sizei n, uint *buffers );', 'formal-declaration-only'),
    ('object-declarations', 'declaration', 'program-shader', '7.1-7.11.1;7.11.3-7.13.1;7.13.3-7.15', 'program-pipeline-commands', 'declaration', 85, '7.1', 'uint CreateShader( enum type );', 'formal-declaration-only'),
    ('object-declarations', 'declaration', 'memory-barrier', '7.11.2', 'generic-sync-query', 'declaration', 141, '7.11.2', 'void MemoryBarrier( bitfield barriers );', 'formal-declaration-only'),
    ('object-declarations', 'declaration', 'texture', '8.1;8.5-8.26', 'texture-sampler-commands', 'declaration', 157, '8.1', 'void GenTextures( sizei n, uint *textures );;', 'formal-declaration-only'),
    ('object-declarations', 'declaration', 'sampler', '8.2', 'texture-sampler-commands', 'declaration', 159, '8.2', 'void GenSamplers( sizei count, uint *samplers );', 'formal-declaration-only'),
    ('execution', 'declaration', 'pixel-store-and-transfer', '8.4;16', 'pixel-transfer-commands', 'declaration', 426, '16.1', 'void ReadPixels( int x, int y, sizei width, sizei height, enum format, enum type, void *data );', 'formal-declaration-only'),
    ('object-declarations', 'declaration', 'framebuffer-renderbuffer', '9', 'framebuffer-renderbuffer-commands', 'declaration', 244, '9.2', 'void GenFramebuffers( sizei n, uint *framebuffers );', 'formal-declaration-only'),
    ('state-lifecycle', 'state-route', 'vertex-remaining-state', '10.1;10.6-10.7', 'context-state-lifecycle', 'heading', 274, '10', 'Chapter 10 Vertex Specification and Drawing Commands', 'state-and-query-route'),
    ('object-declarations', 'declaration', 'vertex-array', '10.2;10.3.1-10.3.8;10.4', 'vertex-transform-feedback-commands', 'declaration', 294, '10.4', 'void GenVertexArrays( sizei n, uint *arrays );', 'formal-declaration-only'),
    ('execution', 'declaration', 'draw', '10.5', 'draw-raster-commands', 'declaration', 296, '10.5', 'void DrawArrays( enum mode, int first, sizei count );', 'formal-declaration-only'),
    ('execution', 'declaration', 'indirect-draw', '10.3.9', 'draw-raster-commands', 'heading', 293, '10.3.9', '10.3.9 Indirect Commands in Buffer Objects', 'formal-declaration-only'),
    ('state-lifecycle', 'state-route', 'programmable-vertex-stage', '11', 'context-state-lifecycle', 'heading', 306, '11', 'Chapter 11 Programmable Vertex Processing', 'state-route-not-essl-source'),
    ('object-declarations', 'declaration', 'transform-feedback', '12.2', 'vertex-transform-feedback-commands', 'declaration', 357, '12.2', 'void BeginTransformFeedback( enum primitiveMode );', 'formal-declaration-only'),
    ('state-lifecycle', 'state-route', 'post-vertex-state', '12.1;12.3-12.6', 'context-state-lifecycle', 'heading', 353, '12', 'Chapter 12 Fixed-Function Vertex Post-Processing', 'state-rules-not-declarations'),
    ('execution', 'declaration', 'raster-and-framebuffer', '13;15', 'draw-raster-commands', 'declaration', 418, '15.2.3', 'void Clear( bitfield buf );', 'formal-declaration-only'),
    ('state-lifecycle', 'state-route', 'programmable-fragment-stage', '14', 'context-state-lifecycle', 'heading', 389, '14', 'Chapter 14 Programmable Fragment Processing', 'state-route-not-essl-source'),
    ('execution', 'declaration', 'compute', '17', 'draw-raster-commands', 'declaration', 439, '17', 'void DispatchCompute( uint num groups x, uint num groups y, uint num groups z );', 'formal-declaration-only'),
    ('execution', 'declaration', 'debug', '18', 'debug-special-query-commands', 'declaration', 445, '18.2', 'void DebugMessageCallback( DEBUGPROC callback, const void *userParam );', 'formal-declaration-only'),
    ('execution', 'declaration', 'special', '19', 'debug-special-query-commands', 'declaration', 455, '19.1', 'void Hint( enum target, enum hint );', 'formal-declaration-only'),
    ('execution', 'declaration', 'context-queries', '20', 'debug-special-query-commands', 'declaration', 457, '20.1', 'void GetBooleanv( enum pname, boolean *data );', 'formal-declaration-only'),
    ('state-sources', 'state-route', 'state-tables', '21.1-21.39', 'context-state-lifecycle', 'heading', 464, '21', 'Chapter 21 State Tables', 'table-rules-require-triggered-fact'),
    ('state-sources', 'separate-inventory', 'limit-format-tables', '21.40-21.42', 'limit-format-inventory', 'limit', 504, '21.40', 'Table 21.40: Implementation Dependent Values', 'separate-bounded-limit-format-inventory'),
    ('state-sources', 'separate-inventory', 'unreviewed-limit-format-tables', '21.43-21.57', 'limit-format-inventory', 'limit', 507, '21.43', 'Table 21.43: Implementation Dependent Version and Extension Support', 'unreviewed-outside-bounded-limit-format-inventory'),
    ('state-sources', 'separate-inventory', 'format-appendix', 'appendix-C', 'limit-format-inventory', 'heading', 531, 'C', 'Appendix C Compressed Texture Image Formats', 'separate-format-source-route'),
    ('state-sources', 'source-route', 'extension-source', 'external:extension-source', 'extension-unavailable', 'unavailable', 0, '', 'extension', 'unadmitted-distinct-source'),
    ('exclusions', 'exclusion', 'invariance-appendix', 'appendix-A', 'explicit-exclusion', 'heading', 522, 'A', 'Appendix A Invariance', 'non-command-appendix-not-source-fact'),
    ('exclusions', 'exclusion', 'corollaries-appendix', 'appendix-B', 'explicit-exclusion', 'heading', 529, 'B', 'Appendix B Corollaries', 'non-command-appendix-not-source-fact'),
    ('exclusions', 'exclusion', 'historical-profiles', 'appendix-D;appendix-E;appendix-F;appendix-G', 'explicit-exclusion', 'heading', 534, 'D', 'Appendix D Version 3.0 and Before', 'not-gles-32-core-source-scope'),
    ('exclusions', 'exclusion', 'desktop-gl', 'external:desktop-gl', 'explicit-exclusion', 'external', 0, '', 'desktop-gl', 'cross-profile-source-not-admitted'),
    ('exclusions', 'exclusion', 'registry-header', 'external:gl.xml', 'explicit-exclusion', 'external', 0, '', 'gl.xml', 'registry-not-admitted-normative-source'),
    ('exclusions', 'exclusion', 'pdf-index', 'index', 'explicit-exclusion', 'heading', 571, 'Index', 'Index', 'omission-crosscheck-not-fact-source'),
)
INDEX_WITNESSES = (
    ('Uniform4f', 598, 'command-syntax', 'declaration-grammar'), ('FenceSync', 577, 'sync', 'generic-sync-query'),
    ('GenBuffers', 579, 'buffer', 'buffer-commands'), ('CreateShader', 575, 'program-shader', 'program-pipeline-commands'),
    ('GenTextures', 579, 'texture', 'texture-sampler-commands'), ('GenFramebuffers', 579, 'framebuffer-renderbuffer', 'framebuffer-renderbuffer-commands'),
    ('GenVertexArrays', 579, 'vertex-array', 'vertex-transform-feedback-commands'), ('DrawArraysIndirect', 577, 'indirect-draw', 'draw-raster-commands'),
    ('ReadPixels', 591, 'pixel-store-and-transfer', 'pixel-transfer-commands'), ('DispatchCompute', 576, 'compute', 'draw-raster-commands'),
    ('DebugMessageCallback', 575, 'debug', 'debug-special-query-commands'), ('GetBooleanv', 579, 'context-queries', 'debug-special-query-commands'),
    ('MAX TEXTURE SIZE', 588, 'limit-format-tables', 'limit-format-inventory'),
)


def scope_parts(value: object, reject):
    if not isinstance(value, str) or not value:
        reject("source scope is not a nonempty string")
    parts = tuple(value.split(";"))
    if len(parts) != len(set(parts)) or any(not part for part in parts):
        reject("source scope is duplicate or empty")
    for part in parts:
        lower = part.lower()
        if "catch" in lower or lower in {"*", "all", "all-sources"}:
            reject("source scope is wildcard or catch-all")
        if SECTION.fullmatch(part):
            low, _, high = part.partition("-")
            number = lambda item: tuple(int(piece) for piece in item.split("."))
            if number(low) > number(high or low):
                reject("source scope has an inverted numeric interval")
        elif not (EXTERNAL.fullmatch(part) or re.fullmatch(r"appendix-[A-Z]", part) or part == "index"):
            reject("source scope is not an admitted finite form")
    return parts


def section_in_scope(scope: str, section: str) -> bool:
    if "-" not in scope:
        return section == scope or section.startswith(scope + ".")
    low, _, high = scope.partition("-")
    number = lambda item: tuple(int(piece) for piece in item.split("."))
    return number(low) <= number(section) <= number(high)


def locked(families, routes, reject) -> None:
    if tuple(families) != CLOSED_FAMILIES:
        reject("closed family manifest is missing, reordered, rerouted, or mutated")
    ids, used, scopes, intervals = set(), set(), set(), []
    for chunk, kind, identifier, scope, route, anchor_kind, page, section, text, reason in families:
        if identifier in ids or route not in routes or not isinstance(reason, str) or not reason:
            reject("closed family manifest has a duplicate identity or invalid route")
        ids.add(identifier); used.add(route)
        parts = scope_parts(scope, reject)
        for part in parts:
            if part in scopes:
                reject("two families own the same source scope")
            scopes.add(part)
            if SECTION.fullmatch(part):
                low, _, high = part.partition("-")
                number = lambda item: tuple(int(piece) for piece in item.split("."))
                intervals.append((number(low), number(high or low)))
        if anchor_kind in {"declaration", "grammar", "heading", "limit"} and (type(page) is not int or page < 1 or not isinstance(text, str) or not text or not isinstance(section, str) or not section):
            reject("closed family has an incomplete PDF anchor")
        if SECTION.fullmatch(section):
            numeric = [part for part in parts if SECTION.fullmatch(part)]
            parent = anchor_kind == "heading" and numeric and all(part.startswith(section + ".") for part in numeric)
            if numeric and not parent and not any(section_in_scope(part, section) for part in numeric):
                reject("closed family scope lacks its numeric-section witness")
    for index, (low, high) in enumerate(intervals):
        if any(low <= other_high and other_low <= high for other_low, other_high in intervals[index + 1:]):
            reject("two families have overlapping numeric source scopes")
    if used != set(routes):
        reject("closed family manifest leaves a route unused")


def exact_pdf_anchor(anchor_kind, page, section, text, pages, page_text, reject) -> None:
    if not 1 <= page <= pages:
        reject("sealed PDF anchor page is outside the source")
    source = page_text(page)
    if not isinstance(text, str) or not text or not re.search(rf"(?<![A-Za-z0-9_]){re.escape(text)}(?![A-Za-z0-9_])", source):
        reject("sealed PDF lacks an exact anchor")
    if SECTION.fullmatch(section):
        section_ok = re.search(rf"(?<![0-9.]){re.escape(section)}(?![0-9]|\.[0-9])", source)
    else:
        section_ok = re.search(rf"(?<![A-Za-z0-9_]){re.escape(section)}(?![A-Za-z0-9_])", source)
    if not section_ok:
        reject("sealed PDF anchor lacks its exact section witness")
    if anchor_kind == "declaration" and not DECLARATION.fullmatch(text):
        reject("sealed PDF declaration anchor is incomplete")
    if anchor_kind == "grammar" and not (text.startswith("void ") and text.endswith(";")):
        reject("sealed PDF grammar anchor is incomplete")


def index_crosscheck(raw: bytes, reject):
    pages = tuple(range(571, 602))
    try:
        result = subprocess.run(["pdftotext", "-f", "571", "-l", "601", "-layout", "-", "-"], input=raw, capture_output=True, check=False)
        chunks = [part for part in result.stdout.decode("utf-8").split("\f") if part.strip()]
    except (OSError, UnicodeDecodeError) as error:
        reject(f"physical PDF index crosscheck cannot be read: {error}")
    if result.returncode or len(chunks) != len(pages):
        reject("physical PDF index crosscheck is incomplete or unreadable")
    routes = {identifier: route for _chunk, _kind, identifier, _scope, route, *_ in CLOSED_FAMILIES}
    witnesses = []
    for term, page, family, route in INDEX_WITNESSES:
        hits = [(pages[index], len(re.findall(rf"(?<![A-Za-z0-9_]){re.escape(term)}(?![A-Za-z0-9_])", chunk))) for index, chunk in enumerate(chunks)]
        if sum(count for _page, count in hits) != 1 or dict(hits).get(page) != 1 or routes.get(family) != route:
            reject("index witness is missing, duplicated, misplaced, or unaccounted")
        witnesses.append({"term": term, "physical_page": page, "expected_family_id": family, "expected_route": route})
    return {"role": "omission-crosscheck-only", "physical_page_range": [571, 601], "checked_page_count": 31,
            "normalized_index_witnesses": witnesses}
