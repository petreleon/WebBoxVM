"""Bounded formal GLES program/pipeline literals for F03.3.2.2.3.3.1."""

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
SECTION_WITNESSES = {
    "7.1": (85, "7.1 Shader Objects"), "7.2": (88, "7.2 Shader Binaries"),
    "7.3": (89, "7.3 Program Objects"), "7.3.1": (97, "7.3.1 Program Interfaces"),
    "7.4": (112, "7.4 Program Pipeline Objects"), "7.5": (118, "7.5 Program Binaries"),
}
SECTION_PAGES = {
    "7.1": (85, 86, 87, 88), "7.2": (88,), "7.3": (89, 90, 91, 93, 94, 95),
    "7.3.1": (101, 102, 103, 104, 111), "7.4": (112, 113, 114, 115), "7.5": (118,),
}
SECTION_BOUNDARIES = {
    (88, "7.1"): ("before", "7.2 Shader Binaries"),
    (88, "7.2"): ("after", "7.2 Shader Binaries"),
}
# physical page, semantic section, unprefixed command, exact normalized PDF declaration
DECLARATIONS = (
    (85, "7.1", "CreateShader", "uint CreateShader( enum type );"),
    (86, "7.1", "ShaderSource", "void ShaderSource( uint shader, sizei count, const char * const *string, const int *length );"),
    (87, "7.1", "CompileShader", "void CompileShader( uint shader );"),
    (87, "7.1", "ReleaseShaderCompiler", "void ReleaseShaderCompiler( void );"),
    (88, "7.1", "DeleteShader", "void DeleteShader( uint shader );"),
    (88, "7.1", "IsShader", "boolean IsShader( uint shader );"),
    (88, "7.2", "ShaderBinary", "void ShaderBinary( sizei count, const uint *shaders, enum binaryformat, const void *binary, sizei length );"),
    (89, "7.3", "CreateProgram", "uint CreateProgram( void );"),
    (89, "7.3", "AttachShader", "void AttachShader( uint program, uint shader );"),
    (90, "7.3", "DetachShader", "void DetachShader( uint program, uint shader );"),
    (91, "7.3", "LinkProgram", "void LinkProgram( uint program );"),
    (93, "7.3", "UseProgram", "void UseProgram( uint program );"),
    (94, "7.3", "ProgramParameteri", "void ProgramParameteri( uint program, enum pname, int value );"),
    (95, "7.3", "DeleteProgram", "void DeleteProgram( uint program );"),
    (95, "7.3", "IsProgram", "boolean IsProgram( uint program );"),
    (95, "7.3", "CreateShaderProgramv", "uint CreateShaderProgramv( enum type, sizei count, const char * const *strings );"),
    (101, "7.3.1", "GetProgramInterfaceiv", "void GetProgramInterfaceiv( uint program, enum programInterface, enum pname, int *params );"),
    (102, "7.3.1", "GetProgramResourceIndex", "uint GetProgramResourceIndex( uint program, enum programInterface, const char *name );"),
    (103, "7.3.1", "GetProgramResourceName", "void GetProgramResourceName( uint program, enum programInterface, uint index, sizei bufSize, sizei *length, char *name );"),
    (104, "7.3.1", "GetProgramResourceiv", "void GetProgramResourceiv( uint program, enum programInterface, uint index, sizei propCount, const enum *props, sizei count, sizei *length, int *params );"),
    (111, "7.3.1", "GetProgramResourceLocation", "int GetProgramResourceLocation( uint program, enum programInterface, const char *name );"),
    (112, "7.4", "GenProgramPipelines", "void GenProgramPipelines( sizei n, uint *pipelines );"),
    (112, "7.4", "DeleteProgramPipelines", "void DeleteProgramPipelines( sizei n, const uint *pipelines );"),
    (113, "7.4", "IsProgramPipeline", "boolean IsProgramPipeline( uint pipeline );"),
    (113, "7.4", "BindProgramPipeline", "void BindProgramPipeline( uint pipeline );"),
    (114, "7.4", "UseProgramStages", "void UseProgramStages( uint pipeline, bitfield stages, uint program );"),
    (115, "7.4", "ActiveShaderProgram", "void ActiveShaderProgram( uint pipeline, uint program );"),
    (118, "7.5", "GetProgramBinary", "void GetProgramBinary( uint program, sizei bufSize, sizei *length, enum *binaryFormat, void *binary );"),
    (118, "7.5", "ProgramBinary", "void ProgramBinary( uint program, enum binaryFormat, const void *binary, sizei length );"),
)
SEALED_DECLARATIONS_SHA256 = "0bbef764a93e1edd05ea6cf526b56e283961b1b7e8fabd96e93d5ca655eca49d"
PROTOTYPE = re.compile(r"(?:void |void \*|boolean |uint |int )([A-Z][A-Za-z0-9]*)\(.*\);\Z")
SOURCE_PROTOTYPE = re.compile(r"(?:void \*|void |boolean |uint |int )[A-Z][A-Za-z0-9]*\(.*?\);")


class CatalogError(ValueError):
    """A proposed declaration lies outside the sealed program/pipeline source slice."""


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
        reject(f"cannot read sealed program/pipeline page: {error}")
    if result.returncode != 0:
        reject("cannot read sealed program/pipeline page")
    return compact(value)


def in_section_window(page: int, section: str, declaration: str, value: str) -> bool:
    boundary = SECTION_BOUNDARIES.get((page, section))
    if boundary is None:
        return True
    relation, marker = boundary
    start, split = value.find(declaration), value.find(marker)
    return start >= 0 and split >= 0 and ((relation == "before" and start < split) or (relation == "after" and start > split))


def source_slice(text) -> tuple[tuple[int, str], ...]:
    return tuple((page, item) for page in range(85, 119) for item in SOURCE_PROTOTYPE.findall(text(page)))


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
            reject("program/pipeline inventory rows cannot be rendered")
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
        reject("program/pipeline command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def facts(raw: bytes, pages: int, family_order: int, normalize) -> list[list[object]]:
    if (pages != PAGES or family_order != FAMILY[3]
            or declaration_sha256(DECLARATIONS) != SEALED_DECLARATIONS_SHA256):
        reject("GLES program/pipeline source boundary, literal set, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str:
        texts.setdefault(page, page_text(raw, page))
        return texts[page]
    expected_slice = tuple((page, declaration) for page, _, _, declaration in DECLARATIONS)
    if source_slice(text) != expected_slice:
        reject("formal program/pipeline source window is incomplete, rerouted, or out of source order")
    result, names = [], set()
    for order, (page, section, name, declaration) in enumerate(DECLARATIONS, 1):
        witness, match, page_value = SECTION_WITNESSES.get(section), PROTOTYPE.fullmatch(declaration), text(page)
        if (witness is None or page not in SECTION_PAGES.get(section, ()) or witness[1] not in text(witness[0])
                or declaration not in page_value or not in_section_window(page, section, declaration, page_value)
                or match is None or match.group(1) != name or "{" in declaration):
            reject("formal program/pipeline declaration or section witness is missing, malformed, or outside this source slice")
        try:
            c_names = normalize([("literal", name)])
        except Exception as error:
            reject(f"declaration grammar rejected a literal program/pipeline name: {error}")
        if c_names != [f"gl{name}"] or name in names:
            reject("literal program/pipeline declaration is duplicate, prefixed, or outside the sealed grammar")
        names.add(name)
        result.append([f"gles32-program-pipeline-literal-{order:02d}", name, c_names[0], declaration, page, section, order])
    if len(result) != len(DECLARATIONS) or [row[-1] for row in result] != list(range(1, len(result) + 1)):
        reject("raw program/pipeline declaration ordering is incomplete or unstable")
    return result
