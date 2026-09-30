"""Extract complete formal declarations from sealed, finite PDF section windows."""

import hashlib
import importlib.util
import json
import re
from pathlib import Path

PDF_PATH = Path(__file__).with_name("pdf.py")
if PDF_PATH.is_symlink() or not PDF_PATH.is_file():
    raise ValueError("fixed PDF extractor must be a regular file")
SPEC = importlib.util.spec_from_file_location("webboxvm_declaration_pdf", PDF_PATH)
if SPEC is None or SPEC.loader is None:
    raise ValueError("cannot load fixed PDF extractor")
PDF = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PDF)
CHECK_PATH = Path(__file__).with_name("crosscheck.py")
if CHECK_PATH.is_symlink() or not CHECK_PATH.is_file():
    raise ValueError("fixed declaration crosscheck must be a regular file")
CHECK_SPEC = importlib.util.spec_from_file_location("webboxvm_declaration_crosscheck", CHECK_PATH)
CHECK = importlib.util.module_from_spec(CHECK_SPEC)
CHECK_SPEC.loader.exec_module(CHECK)

ENTRY_FIELDS = ("raw_id", "unprefixed_name", "c_name", "declaration", "physical_page", "section",
                "source_order", "family_id", "source_family_order")
DEFAULT_FIELDS = frozenset(("derivation_class", "source_locator_format"))
PROTOTYPE = re.compile(r"(?m)^(?:void|uint|ubyte|boolean|enum)\s+\*?(?P<name>[A-Z][^\n(]*)\([^;]*?\);")
HEADING = re.compile(r"(?m)^(?P<section>\d+(?:\.\d+)+) (?P<title>[A-Z][a-z][^\n]*)$")


class CatalogError(ValueError):
    """A formal declaration or source boundary does not match its sealed catalog."""


def reject(message):
    raise CatalogError(message)


def digest(value):
    return hashlib.sha256(json.dumps(value, separators=(",", ":"), ensure_ascii=True).encode()).hexdigest()


class DeclarationCatalog:
    def __init__(self, contract):
        self.contract = contract

    def binding(self):
        c = self.contract
        body = (c.FAMILIES, c.WINDOWS, c.EXCLUSIONS, c.TYPOGRAPHY, c.INDEX_WITNESSES, c.ROUTED_DECLARATIONS)
        if digest(body) != c.BOUNDARY_SHA256:
            reject("formal declaration source boundaries or domain catalog changed")

    def bound_family(self, chunk_paths, document):
        self.binding()
        c = self.contract
        routed = [item for item in document(chunk_paths["execution"]).get("families", []) if item.get("route") == c.ROUTE]
        if routed != list(c.FAMILIES):
            reject("formal declaration route is incomplete, rerouted, or reordered")
        return [{"id": item["id"], "source_order": item["source_order"]} for item in routed]

    def raw_defaults(self, _):
        return {"derivation_class": "formal-declaration", "source_locator_format": "gles32-pdf-v1:page={physical_page};section={section}"}

    def coverage(self, binding):
        self.binding()
        return {"complete": True, "families": binding, "source_windows": [
            {"family_id": w[0], "first_physical_page": w[1], "last_physical_page": w[2],
             "start": w[3], "end": w[4], "formal_declaration_count": w[5], "declarations_sha256": w[6]}
            for w in self.contract.WINDOWS], "excluded_non_commands": [list(item) for item in self.contract.EXCLUSIONS],
            "typographic_spellings": [list(item) for item in self.contract.TYPOGRAPHY],
            "index_crosscheck": {"physical_page_range": [571, 601], "witness_count": len(self.contract.INDEX_WITNESSES),
                                 "witnesses_sha256": digest(self.contract.INDEX_WITNESSES), "role": "omission-crosscheck-only"},
            "routed_declarations": [list(item) for item in self.contract.ROUTED_DECLARATIONS],
            "semantic_routes": ["primitive-and-raster-behavior", "state", "limits", "formats", "essl-unavailable"]}

    def extracted(self, raw):
        self.binding()
        c, result, excluded = self.contract, [], set()
        for family, first, last, start, end, count, checksum in c.WINDOWS:
            try:
                texts = PDF.sealed_pdf_pages(raw, first, last)
            except PDF.PdfError as error:
                reject(str(error))
            begins, ends = texts[first].find(start), texts[last].find(end)
            if begins < 0 or ends < 0 or (first == last and ends <= begins):
                reject("formal declaration window heading or end fence is absent")
            texts[first] = texts[first][begins:]
            if first == last:
                ends -= begins
            texts[last] = texts[last][:ends]
            section, facts = start.split(" ")[0] if not start.startswith("Chapter") else start.split(" ")[1], []
            for page, text in texts.items():
                headings = list(HEADING.finditer(text))
                for match in PROTOTYPE.finditer(text):
                    previous = [h for h in headings if h.start() < match.start()]
                    located = previous[-1].group("section") if previous else section
                    name, declaration = match.group("name"), PDF.compact(match.group())
                    exclusion = next((item for item in c.EXCLUSIONS if item[0] == page and item[1] == name), None)
                    if exclusion:
                        if exclusion[3] not in PDF.compact(texts.get(exclusion[2], "")):
                            reject("excluded pseudo-command lacks its explicit normative reason")
                        excluded.add((page, name)); continue
                    facts.append([page, located, name, declaration])
                if headings:
                    section = headings[-1].group("section")
            if len(facts) != count or digest(facts) != checksum:
                reject("formal declaration window is incomplete, changed, or out of source order")
            result.extend((family, *fact) for fact in facts)
        if excluded != {(item[0], item[1]) for item in c.EXCLUSIONS}:
            reject("non-command exclusion catalog is incomplete")
        if {item[3] for item in result} != {item[1] for item in c.INDEX_WITNESSES}:
            reject("formal declarations differ from the independent sealed index command set")
        return result

    def facts(self, raw, pages, binding, grammar, normalize):
        c = self.contract
        expected = [{"id": item["id"], "source_order": item["source_order"]} for item in c.FAMILIES]
        if pages != c.PAGES or c.PROFILE != "gles-3.2" or binding != expected:
            reject("formal declaration profile, pages, or domain binding differs")
        prefix = grammar.get("grammar", {}).get("c_binding", {}).get("c_command_prefix")
        if prefix != "gl":
            reject("formal declarations require the sealed C binding prefix")
        rows, names, orders = [], set(), {item["id"]: item["source_order"] for item in binding}
        templates = grammar["grammar"]["formal_templates"]
        try:
            CHECK.verify_index(raw, c.INDEX_WITNESSES, PDF)
            CHECK.verify_routes(raw, c.ROUTED_DECLARATIONS, c.RAW_ROOT, PDF)
        except (ValueError, OSError, KeyError, TypeError) as error:
            reject(str(error))
        for family, page, section, name, declaration in self.extracted(raw):
            spelling = next((item[3] for item in c.TYPOGRAPHY if item[:3] == (page, section, name)), None)
            if spelling is not None:
                if not re.fullmatch(r"Get(?:Boolean|Integer|Integer64)i_v", spelling):
                    reject("unsealed typographic command spelling")
                outputs = [prefix + spelling]
            else:
                form = "template" if "{" in name else "literal"
                if form == "template" and not any(t["formal_name"] == name and t["physical_page"] == page
                                                   and t["section"] == section for t in templates):
                    reject("formal template is outside its sealed grammar locator")
                try:
                    outputs = normalize([(form, name)])
                except Exception as error:
                    reject(f"formal declaration grammar failed: {error}")
                if form == "literal" and outputs != [prefix + name]:
                    reject("literal declaration binding is changed or guessed")
            for cname in outputs:
                if not isinstance(cname, str) or not cname.startswith(prefix) or cname in names:
                    reject("formal declarations produce duplicate C names")
                names.add(cname)
                rows.append([f"{c.RAW_PREFIX}-{len(rows) + 1:02d}", cname[2:], cname, declaration,
                             page, section, len(rows) + 1, family, orders[family]])
        if len(rows) != c.EXPANDED_COUNT:
            reject("expanded formal declaration count differs from the finite contract")
        return rows
