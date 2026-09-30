"""Independent PDF-index witnesses and exact receipts for declarations routed elsewhere."""

import hashlib
import json
from pathlib import Path


def verify_index(raw, witnesses, pdf):
    pages = pdf.sealed_pdf_pages(raw, 571, 601)
    for page, name in witnesses:
        if page not in pages or not any(line.startswith(name + ",") for line in pages[page].splitlines()):
            raise ValueError("independent command index witness is missing or changed")
    if len(witnesses) != len(set(witnesses)):
        raise ValueError("independent command index witnesses are duplicate")


def verify_routes(raw, routes, source_root, pdf):
    for path, file_hash, task, inventory_hash, cname, declaration, page, section in routes:
        target = source_root / path
        if target.is_symlink() or not target.is_file() or ".." in Path(path).parts or Path(path).is_absolute():
            raise ValueError("routed declaration receipt must use its fixed regular artifact")
        contents = target.read_bytes()
        if hashlib.sha256(contents).hexdigest() != file_hash:
            raise ValueError("routed declaration artifact differs from its admitted receipt")
        value = json.loads(contents)
        if value["source"]["sha256"] != hashlib.sha256(raw).hexdigest() or value["inventory_sha256"] != inventory_hash:
            raise ValueError("routed declaration does not use the same sealed source")
        rows = [row for row in value["raw_entries"] if row["c_name"] == cname]
        if len(rows) != 1 or any(rows[0][key] != expected for key, expected in (
                ("declaration", declaration), ("physical_page", page), ("section", section))):
            raise ValueError("routed declaration receipt is incomplete or rerouted")
        if declaration not in pdf.sealed_pdf_page(raw, page):
            raise ValueError("routed declaration exact normative quote is absent")
