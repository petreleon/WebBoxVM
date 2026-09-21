"""Bounded formal GLES uniform/query literals for F03.3.2.2.3.3.2."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "program-pipeline-commands"
FAMILY = (
    "object-declarations", "program-shader",
    ("7.1-7.11.1", "7.11.3-7.13.1", "7.13.3-7.15"), 15, 85, "7.1",
    "uint CreateShader( enum type );",
)
SOURCE_PAGES = (123, 124, 125, 133, 145, 146, 148, 149, 150, 151, 152)
SECTION_WITNESSES = {
    "7.6": (120, "7.6 Uniform Variables"),
    "7.6.3": (133, "7.6.3 Uniform Buffer Object Bindings"),
    "7.12": (145, "7.12 Shader, Program, and Program Pipeline Queries"),
}
SECTION_PAGES = {
    "7.6": (123, 124, 125), "7.6.3": (133,),
    "7.12": (145, 146, 148, 149, 150, 151, 152),
}
SECTION_BOUNDARIES = {
    (133, "7.6.3"): ("after", "7.6.3 Uniform Buffer Object Bindings"),
    (145, "7.12"): ("after", "7.12 Shader, Program, and Program Pipeline Queries"),
}
# physical page, semantic section, unprefixed command, exact normalized PDF declaration
DECLARATIONS = (
    (123, "7.6", "GetUniformLocation", "int GetUniformLocation( uint program, const char *name );"),
    (123, "7.6", "GetUniformIndices", "void GetUniformIndices( uint program, sizei uniformCount, const char * const *uniformNames, uint *uniformIndices );"),
    (124, "7.6", "GetActiveUniform", "void GetActiveUniform( uint program, uint index, sizei bufSize, sizei *length, int *size, enum *type, char *name );"),
    (124, "7.6", "GetActiveUniformsiv", "void GetActiveUniformsiv( uint program, sizei uniformCount, const uint *uniformIndices, enum pname, int *params );"),
    (124, "7.6", "GetUniformBlockIndex", "uint GetUniformBlockIndex( uint program, const char *uniformBlockName );"),
    (125, "7.6", "GetActiveUniformBlockName", "void GetActiveUniformBlockName( uint program, uint uniformBlockIndex, sizei bufSize, sizei length, char *uniformBlockName );"),
    (125, "7.6", "GetActiveUniformBlockiv", "void GetActiveUniformBlockiv( uint program, uint uniformBlockIndex, enum pname, int *params );"),
    (133, "7.6.3", "UniformBlockBinding", "void UniformBlockBinding( uint program, uint uniformBlockIndex, uint uniformBlockBinding );"),
    (145, "7.12", "GetShaderiv", "void GetShaderiv( uint shader, enum pname, int *params );"),
    (146, "7.12", "GetProgramiv", "void GetProgramiv( uint program, enum pname, int *params );"),
    (148, "7.12", "GetProgramPipelineiv", "void GetProgramPipelineiv( uint pipeline, enum pname, int *params );"),
    (149, "7.12", "GetAttachedShaders", "void GetAttachedShaders( uint program, sizei maxCount, sizei *count, uint *shaders );"),
    (150, "7.12", "GetShaderInfoLog", "void GetShaderInfoLog( uint shader, sizei bufSize, sizei *length, char *infoLog );"),
    (150, "7.12", "GetProgramInfoLog", "void GetProgramInfoLog( uint program, sizei bufSize, sizei *length, char *infoLog );"),
    (150, "7.12", "GetProgramPipelineInfoLog", "void GetProgramPipelineInfoLog( uint pipeline, sizei bufSize, sizei *length, char *infoLog );"),
    (151, "7.12", "GetShaderSource", "void GetShaderSource( uint shader, sizei bufSize, sizei *length, char *source );"),
    (151, "7.12", "GetShaderPrecisionFormat", "void GetShaderPrecisionFormat( enum shadertype, enum precisiontype, int *range, int *precision );"),
    (152, "7.12", "GetUniformfv", "void GetUniformfv( uint program, int location, float *params );"),
    (152, "7.12", "GetUniformiv", "void GetUniformiv( uint program, int location, int *params );"),
    (152, "7.12", "GetUniformuiv", "void GetUniformuiv( uint program, int location, uint *params );"),
    (152, "7.12", "GetnUniformfv", "void GetnUniformfv( uint program, int location, sizei bufSize, float *params );"),
    (152, "7.12", "GetnUniformiv", "void GetnUniformiv( uint program, int location, sizei bufSize, int *params );"),
    (152, "7.12", "GetnUniformuiv", "void GetnUniformuiv( uint program, int location, sizei bufSize, uint *params );"),
)
SEALED_DECLARATIONS_SHA256 = "0bd847e8f6fba180bec3b2670ee501580207870714ce1a6dc0fd9f7e0ac5a47e"
PROTOTYPE = re.compile(r"(?:void |void \*|boolean |uint |int )([A-Z][A-Za-z0-9]*)\(.*\);\Z")
SOURCE_PROTOTYPE = re.compile(r"(?:void \*|void |boolean |uint |int )[A-Z][A-Za-z0-9]*\(.*?\);")


class CatalogError(ValueError):
    """A proposed declaration lies outside the sealed uniform/query source slice."""


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
        reject(f"cannot read sealed uniform/query page: {error}")
    if result.returncode != 0:
        reject("cannot read sealed uniform/query page")
    return compact(value)


def in_section_window(page: int, section: str, declaration: str, value: str) -> bool:
    boundary = SECTION_BOUNDARIES.get((page, section))
    if boundary is None:
        return True
    relation, marker = boundary
    start, split = value.find(declaration), value.find(marker)
    return start >= 0 and split >= 0 and ((relation == "before" and start < split) or (relation == "after" and start > split))


def source_slice(text) -> tuple[tuple[int, str], ...]:
    return tuple((page, item) for page in SOURCE_PAGES for item in SOURCE_PROTOTYPE.findall(text(page)))


def pretty_json(value: dict[str, object]) -> str:
    """Keep a large literal inventory readable without inflating its line count."""
    lines, pairs = ["{"], sorted(value.items())
    for position, (key, item) in enumerate(pairs):
        comma = "," if position + 1 < len(pairs) else ""
        if key != "raw_entries":
            field = f"{json.dumps(key)}: {json.dumps(item, indent=2, sort_keys=True)}"
            chunk = [f"  {line}" for line in field.splitlines()]
            chunk[-1] += comma
            lines.extend(chunk)
            continue
        if not isinstance(item, list) or not all(isinstance(row, list) and len(row) == 7 for row in item):
            reject("uniform/query inventory rows cannot be rendered")
        lines.append('  "raw_entries": [')
        for row_number, row in enumerate(item):
            row_comma = "," if row_number + 1 < len(item) else ""
            lines.extend((f"    [{json.dumps(row[0])}, {json.dumps(row[1])}, {json.dumps(row[2])},",
                          f"     {json.dumps(row[3])},",
                          f"     {json.dumps(row[4])}, {json.dumps(row[5])}, {json.dumps(row[6])}]{row_comma}"))
        lines.append(f"  ]{comma}")
    return "\n".join((*lines, "}"))


def bound_family(chunk_paths: dict[str, object], document) -> int:
    chunk, identifier, scopes, order, page, section, anchor = FAMILY
    value = document(chunk_paths[chunk])
    rows = [row for row in value.get("families", []) if row.get("id") == identifier]
    expected = {"id": identifier, "family_kind": "declaration", "source_scope": list(scopes), "route": ROUTE,
                "reason": "formal-declaration-only", "source_order": order,
                "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": anchor}}
    if len(rows) != 1 or rows[0] != expected:
        reject("uniform/query command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def facts(raw: bytes, pages: int, family_order: int, normalize) -> list[list[object]]:
    if (pages != PAGES or family_order != FAMILY[3]
            or SOURCE_PAGES != (123, 124, 125, 133, 145, 146, 148, 149, 150, 151, 152)
            or declaration_sha256(DECLARATIONS) != SEALED_DECLARATIONS_SHA256):
        reject("GLES uniform/query source boundary, literal set, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str:
        texts.setdefault(page, page_text(raw, page))
        return texts[page]
    expected_slice = tuple((page, declaration) for page, _, _, declaration in DECLARATIONS)
    if source_slice(text) != expected_slice:
        reject("formal uniform/query source window is incomplete, rerouted, or out of source order")
    result, names = [], set()
    for order, (page, section, name, declaration) in enumerate(DECLARATIONS, 1):
        witness, match, page_value = SECTION_WITNESSES.get(section), PROTOTYPE.fullmatch(declaration), text(page)
        if (witness is None or page not in SECTION_PAGES.get(section, ()) or witness[1] not in text(witness[0])
                or declaration not in page_value or not in_section_window(page, section, declaration, page_value)
                or match is None or match.group(1) != name or "{" in declaration):
            reject("formal uniform/query declaration or section witness is missing, malformed, or outside this source slice")
        try:
            c_names = normalize([("literal", name)])
        except Exception as error:
            reject(f"declaration grammar rejected a literal uniform/query name: {error}")
        if c_names != [f"gl{name}"] or name in names:
            reject("literal uniform/query declaration is duplicate, prefixed, or outside the sealed grammar")
        names.add(name)
        result.append([f"gles32-uniform-query-literal-{order:02d}", name, c_names[0], declaration, page, section, order])
    if len(result) != len(DECLARATIONS) or [row[-1] for row in result] != list(range(1, len(result) + 1)):
        reject("raw uniform/query declaration ordering is incomplete or unstable")
    return result
