#!/usr/bin/env python3
"""Fixed-path provenance and PDF access for F03.2.3.4.1.1."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
AUTHORITY = ROOT / "01-source-authority/opengl_source_authority.py"
CACHE = ROOT / "02-command-object-state-inventory/01-normative-pdf-cache/opengl_normative_pdf_cache.py"
REVIEWED = ROOT / "03-limit-format-shader-inventory/01-limit-format-raw-inventory/opengl_limit_format_raw_inventory.py"
LEDGER = ROOT / "03-limit-format-shader-inventory/02-unadmitted-shader-extension-ledger/opengl_unadmitted_ledger.py"
PROFILE = "opengl-4.6-core"
MAX_SERIALIZED = 1024 * 1024


class CatalogError(ValueError):
    """A final-table classification escapes its sealed source boundary."""


def reject(message: str) -> None:
    raise CatalogError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def sha256(value: object) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def unique(items):
    value = {}
    for key, item in items:
        if key in value:
            reject("catalog has duplicate JSON fields")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    if path.is_symlink() or not path.is_file():
        reject("catalog receipt must be a regular file")
    try:
        raw = path.read_bytes()
        if len(raw) > MAX_SERIALIZED:
            reject("catalog receipt exceeds its serialized-size cap")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=unique)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"catalog receipt cannot be read: {error}")
    if not isinstance(value, dict):
        reject("catalog receipt is not a JSON object")
    return value


def private(path: Path, name: str):
    if path.is_symlink() or not path.is_file():
        reject("fixed private dependency must be a regular file")
    prior = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            reject("cannot load fixed private dependency")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            reject("fixed private dependency resolved from an unexpected path")
        return module
    except CatalogError:
        raise
    except Exception as error:
        reject(f"cannot load fixed private dependency: {error}")
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


def inputs(cache_root: Path) -> dict[str, object]:
    try:
        authority = private(AUTHORITY, "f032341_authority")
        cache = private(CACHE, "f032341_cache")
        reviewed_api = private(REVIEWED, "f032341_reviewed")
        ledger_api = private(LEDGER, "f032341_ledger")
        boundary, manifest = authority.validate(), cache.manifest()
        reviewed, ledger = reviewed_api.validate(cache_root), ledger_api.validate()
        source = boundary["normative_root"]
        decisions = [authority.consume(identifier, "opengl46-core-pdf-v1:page=660;section=22.2")
                     for identifier in ("limit-format", "command-object-state")]
        raw = cache.pdf_bytes(cache.external_root(cache_root), cache.SOURCE)
        pages = cache.physical_pages(raw)
    except Exception as error:
        reject(str(error))
    boundary_keys = ("boundary_sha256", "source_contract_sha256", "inventory_lock_sha256")
    if (boundary.get("profile") != PROFILE or source != cache.SOURCE or manifest.get("source") != source
            or pages != cache.PAGES or any(item.get("source") != source for item in decisions)
            or [item.get("decision", {}).get("id") for item in decisions] != ["limit-format", "command-object-state"]
            or any(item.get("decision", {}).get("availability") != "available" for item in decisions)
            or reviewed.get("profile") != PROFILE or reviewed.get("source") != source
            or ledger.get("profile") != PROFILE
            or any(reviewed.get(key.replace("boundary_sha256", "source_authority_boundary_sha256")) != boundary[key]
                   for key in boundary_keys)
            or ledger.get("source_authority_sha256") != boundary["boundary_sha256"]
            or any(ledger.get(key) != boundary[key] for key in boundary_keys[1:])):
        reject("authority, cache, reviewed inventory, or unadmitted ledger identities differ")
    return {"authority": boundary, "cache": manifest, "reviewed": reviewed, "ledger": ledger,
            "raw": raw, "pages": pages, "source": source}


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"],
                                input=raw, capture_output=True, check=False)
        text = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        reject(f"cannot read sealed PDF page: {error}")
    if result.returncode != 0:
        reject("cannot read sealed PDF page")
    return text


def normalized_page(raw: bytes, page: int) -> str:
    text = page_text(raw, page)
    text = re.sub(r"(?<=\w)-[ \t]*\n[ \t]*(?=\w)", "", text)
    text = re.sub(r"[ \t]+-[ \t]*\n[ \t]*", " ", text)
    return "\n".join(re.sub(r"[ \t]+", " ", line).strip() for line in text.splitlines())
