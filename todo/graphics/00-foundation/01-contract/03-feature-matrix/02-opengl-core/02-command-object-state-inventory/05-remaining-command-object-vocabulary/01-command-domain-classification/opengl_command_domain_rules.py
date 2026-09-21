"""Closed source-family rules for F03.2.2.5.1; these are not API facts."""

from __future__ import annotations

import re
import subprocess

PROFILE, PAGES = "opengl-4.6-core", 851
SECTION = re.compile(r"^[1-9][0-9]*(?:\.[1-9][0-9]*)*$")
SCOPE_PART = re.compile(r"^[1-9][0-9]*(?:\.[1-9][0-9]*)*(?:-[1-9][0-9]*(?:\.[1-9][0-9]*)*)?$")
ROUTES = {
    "declaration-grammar": ("F03.2.2.5.2", "grammar"),
    "baseline-direct-creation": ("F03.2.2.2", "baseline"),
    "generic-object-sync": ("F03.2.2.5.3.1", "extract"),
    "buffer-commands": ("F03.2.2.5.3.2", "extract"),
    "program-pipeline-commands": ("F03.2.2.5.3.3", "extract"),
    "texture-sampler-commands": ("F03.2.2.5.3.4", "extract"),
    "framebuffer-renderbuffer-commands": ("F03.2.2.5.3.5", "extract"),
    "vertex-transform-feedback-commands": ("F03.2.2.5.3.6", "extract"),
    "global-execution-sync": ("F03.2.2.5.4.1", "extract"),
    "draw-compute-submission": ("F03.2.2.5.4.2", "extract"),
    "raster-framebuffer-state": ("F03.2.2.5.4.3", "extract"),
    "pixel-read-copy": ("F03.2.2.5.4.4", "extract"),
    "debug-special-context-queries": ("F03.2.2.5.4.5", "extract"),
    "state-lifecycle-route": ("F03.2.2.3", "defer"),
    "limit-format-route": ("F03.2.3", "defer"),
    "shader-source-unavailable": ("F03.2.1:shader", "unavailable"),
    "extension-source-unavailable": ("F03.2.1:extension", "unavailable"),
    "compatibility-profile-blocked": ("F03.2.1", "blocked"),
    "registry-header-blocked": ("F03.2.1", "blocked"),
}
# id, source scope, locator section, physical page, formal declaration, route
COMMAND_FAMILIES = (
    ("declaration-grammar", "2.2", "2.2", 33, "void Uniform{1234}{if}( int location, T value );", "declaration-grammar"),
    ("global-execution", "2.3.1-2.3.3", "2.3.3", 42, "void Flush( void );", "global-execution-sync"),
    ("direct-creation", "2.6.1", "2.6.1", 50, "", "baseline-direct-creation"),
    ("event-query-sync", "4.1-4.3", "4.1.1", 60, "enum ClientWaitSync( sync sync, bitfield flags, uint64 timeout );", "generic-object-sync"),
    ("buffer", "6.1-6.7", "6.1", 82, "void DeleteBuffers( sizei n, const uint *buffers );", "buffer-commands"),
    ("program-pipeline", "7.1-7.12;7.14-7.15", "7.1", 112, "void CompileShader( uint shader );", "program-pipeline-commands"),
    ("shader-memory-sync", "7.13.2", "7.13.2", 183, "void MemoryBarrier( bitfield barriers );", "global-execution-sync"),
    ("texture-sampler", "8.1-8.26", "8.1", 204, "void BindTextureUnit( uint unit, uint texture );", "texture-sampler-commands"),
    ("framebuffer-renderbuffer", "9.2-9.8", "9.2.1", 327, "void NamedFramebufferParameteri( uint framebuffer, enum pname, int param );", "framebuffer-renderbuffer-commands"),
    ("patch-vertex-state", "10.1.15", "10.1.15", 367, "void PatchParameteri( enum pname, int value );", "state-lifecycle-route"),
    ("current-vertex-attribute-state", "10.2.1", "10.2.1", 369, "void VertexAttrib{1234}{sfd}( uint index, T values );", "state-lifecycle-route"),
    ("vertex-array", "10.3.1-10.3.11;10.5", "10.3.1", 373, "void VertexArrayElementBuffer( uint vaobj, uint buffer );", "vertex-transform-feedback-commands"),
    ("draw-submission", "10.4", "10.4", 388, "void DrawArrays( enum mode, int first, sizei count );", "draw-compute-submission"),
    ("conditional-rendering", "10.9", "10.9", 403, "void BeginConditionalRender( uint id, enum mode );", "draw-compute-submission"),
    ("program-query-api", "11.1.1", "11.1.1", 410, "void GetActiveAttrib( uint program, uint index, sizei bufSize, sizei *length, int *size, enum *type, char *name );", "program-pipeline-commands"),
    ("program-validation", "11.1.3.11", "11.1.3.11", 428, "void ValidateProgram( uint program );", "program-pipeline-commands"),
    ("tessellation-default-state", "11.2.2", "11.2.2", 438, "void PatchParameterfv( enum pname, const float *values );", "state-lifecycle-route"),
    ("transform-feedback", "13.3.1-13.3.2", "13.3.2", 467, "void BeginTransformFeedback( enum primitiveMode );", "vertex-transform-feedback-commands"),
    ("transform-feedback-draw", "13.3.3", "13.3.3", 472, "void DrawTransformFeedback( enum mode, uint id );", "draw-compute-submission"),
    ("post-vertex-raster-state", "13.6-13.9", "13.6", 474, "void ProvokingVertex( enum provokeMode );", "raster-framebuffer-state"),
    ("raster-state", "14.1-14.9", "14.6.1", 500, "void CullFace( enum mode );", "raster-framebuffer-state"),
    ("framebuffer-state", "17.1-17.4", "17.3.3", 525, "void StencilFunc( enum func, int ref, uint mask );", "raster-framebuffer-state"),
    ("fragment-output-query", "15.2.3", "15.2.3", 519, "int GetFragDataLocation( uint program, const char *name );", "program-pipeline-commands"),
    ("pixel-read-copy", "18.2-18.4", "18.2.2", 553, "void ReadPixels( int x, int y, sizei width, sizei height, enum format, enum type, void *data );", "pixel-read-copy"),
    ("compute-submission", "19", "19", 568, "void DispatchCompute( uint num groups x, uint num groups y, uint num groups z );", "draw-compute-submission"),
    ("debug", "20.1-20.9", "20.2", 574, "void DebugMessageCallback( DEBUGPROC callback, const void *userParam );", "debug-special-context-queries"),
    ("special-functions", "21.5", "21.5", 584, "void Hint( enum target, enum hint );", "debug-special-context-queries"),
    ("context-queries", "22.1-22.5", "22.1", 586, "void GetBooleanv( enum pname, boolean *data );", "debug-special-context-queries"),
)
# id, bounded non-command source scope, route, reason
NON_COMMAND_FAMILIES = (
    ("state-lifecycle", "2.5;5.1-5.3;23-state-transitions", "state-lifecycle-route", "declaration-returned-state-not-extracted"),
    ("limit-format", "8.5;9.4;22.3;23-limit-format-rows;appendix-D", "limit-format-route", "numeric-and-format-properties-not-command-declarations"),
    ("shader-source", "shader-language-and-compiler-semantics", "shader-source-unavailable", "unadmitted-distinct-source"),
    ("extension-source", "extension-semantics-only", "extension-source-unavailable", "unadmitted-distinct-source"),
    ("compatibility-profile", "2.6.14;12;16;21.1-21.4;21.6;appendix-E", "compatibility-profile-blocked", "core-profile-excludes-compatibility"),
    ("registry-header", "appendix-K-and-header-files", "registry-header-blocked", "not-an-admitted-normative-command-source"),
)
INDEX_ONLY_EXCLUSIONS = ()
INDEX_PHYSICAL_PAGES = tuple(range(801, 852))
BASELINE_OWNER = {
    "command:glCreateBuffers": "buffer",
    "command:glCreateShader": "program-pipeline",
    "command:glCreateProgram": "program-pipeline",
    "command:glCreateProgramPipelines": "program-pipeline",
    "command:glCreateTextures": "texture-sampler",
    "command:glCreateSamplers": "texture-sampler",
    "command:glCreateRenderbuffers": "framebuffer-renderbuffer",
    "command:glCreateFramebuffers": "framebuffer-renderbuffer",
    "command:glCreateVertexArrays": "vertex-array",
    "command:glCreateTransformFeedbacks": "transform-feedback",
    "command:glCreateQueries": "event-query-sync",
    "command:glFenceSync": "event-query-sync",
}


class RuleError(ValueError):
    """A source family has no fixed, admitted formal anchor."""


def reject(message: str) -> None:
    raise RuleError(message)


def normalized(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def exact(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return left.keys() == right.keys() and all(exact(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(exact(a, b) for a, b in zip(left, right))
    return left == right


def scope_parts(value: str) -> tuple[str, ...]:
    parts = tuple(value.split(";"))
    if not parts or any(not SCOPE_PART.fullmatch(part) for part in parts) or len(parts) != len(set(parts)):
        reject("source family has an unstructured or duplicate section scope")
    return parts


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-layout", "-", "-"],
                                input=raw, capture_output=True, check=False)
        text = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        reject(f"cannot read sealed PDF page: {error}")
    if result.returncode:
        reject("cannot read sealed PDF page")
    return normalized(text)


def command_rows(raw: bytes, pages: int, baseline: dict[str, object], assignments: dict[str, list[dict[str, object]]]) -> list[dict[str, object]]:
    result, seen = [], set()
    baseline_ids = baseline.get("direct_creation_command_ids")
    if (not isinstance(baseline_ids, list) or len(baseline_ids) != 12
            or set(baseline_ids) != set(BASELINE_OWNER)):
        reject("direct-creation baseline IDs are not an exact closed set")
    for order, (identifier, scope, section, page, declaration, route) in enumerate(COMMAND_FAMILIES, 1):
        if (identifier in seen or route not in ROUTES or not SECTION.fullmatch(section) or not 1 <= page <= pages
                or not scope_parts(scope)):
            reject("command family rule is duplicate, unbounded, or has an invalid route")
        seen.add(identifier)
        row = {"id": identifier, "source_scope": scope, "route": route, "source_order": order}
        if route == "baseline-direct-creation":
            if declaration or baseline.get("direct_creation_command_count") != 12:
                reject("direct-creation baseline binding is incomplete")
            row["anchor"] = {"kind": "baseline-artifact", "section": section, "physical_page": page}
        else:
            if not declaration or normalized(declaration) not in page_text(raw, page):
                reject(f"formal declaration anchor is absent for {identifier}")
            row["anchor"] = {"kind": "formal-declaration", "section": section, "physical_page": page,
                             "declaration": declaration,
                             "source_locator": f"opengl46-core-pdf-v1:page={page};section={section}"}
        excluded = [item for item in baseline_ids if BASELINE_OWNER[item] == identifier]
        if excluded:
            anchors = assignments.get(identifier)
            if not isinstance(anchors, list) or [item.get("fact_id") for item in anchors if isinstance(item, dict)] != excluded:
                reject("baseline exclusions are not bound to their assigned source anchors")
            row["excluded_baseline_command_ids"] = excluded
            row["excluded_baseline_command_anchors"] = anchors
        elif assignments.get(identifier):
            reject("unowned baseline source anchor was assigned to a remaining family")
        result.append(row)
    if len(seen) != len(COMMAND_FAMILIES):
        reject("command families are not a closed unique list")
    return result


def non_command_rows() -> list[dict[str, object]]:
    rows, seen = [], set()
    for order, (identifier, scope, route, reason) in enumerate(NON_COMMAND_FAMILIES, 1):
        if identifier in seen or route not in ROUTES or "catchall" in route or not reason:
            reject("non-command family is duplicate, catch-all, or unbounded")
        seen.add(identifier)
        rows.append({"id": identifier, "source_scope": scope, "route": route, "reason": reason,
                     "source_order": order})
    return rows
