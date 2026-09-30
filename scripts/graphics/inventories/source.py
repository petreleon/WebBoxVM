"""Exact source admission shared by bounded GLES declaration inventories."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


class InventoryError(ValueError):
    """A source inventory cannot be reproduced without changing its contract."""


def reject(message: str) -> None:
    raise InventoryError(message)


def private(item_file: Path, name: str):
    if item_file.is_symlink() or not item_file.is_file():
        reject("fixed private dependency must be a regular file")
    prior = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, item_file)
        if spec is None or spec.loader is None:
            reject("cannot load fixed private dependency")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != item_file.resolve():
            reject("fixed private dependency resolved from an unexpected path")
        return module
    except InventoryError:
        raise
    except Exception as error:
        reject(f"cannot load fixed private dependency: {error}")
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


class SourceContext:
    def __init__(self, here: Path, catalog_filename: str, task_key: str, raw_root: Path | None = None):
        raw = raw_root or here.parents[2]
        dependencies = {
            "CATALOG": here / catalog_filename,
            "ARTIFACT": raw / "01-command-domain-classification/gles_command_domain_artifact.py",
            "DOMAIN_API": raw / "01-command-domain-classification/gles_command_domain_classification.py",
            "GRAMMAR": raw / "02-template-declaration-grammar/gles_declaration_grammar.py",
            "CACHE": raw.parent / "01-normative-pdf-cache/gles_normative_pdf_cache.py",
            "LEDGER": raw.parent / "04-unavailable-language-extension-ledger/gles_unavailable_ledger.py",
        }
        for label, path in dependencies.items():
            setattr(self, label, private(path, task_key + "_" + label.lower()))
        self.canonical = self.ARTIFACT.canonical
        self.exact = self.ARTIFACT.exact
        self.batch_snapshot = None

    def artifact(self, callable, *args):
        try:
            return callable(*args)
        except (self.ARTIFACT.ArtifactError, ValueError) as error:
            reject(str(error))

    def reject_unadmitted(self, identifier: str, candidate: str) -> None:
        try:
            self.LEDGER.reject_substitute(identifier, candidate)
        except Exception as error:
            reject(str(error))

    def source_input(self, cache_root: Path):
        catalog, cache = self.CATALOG, self.CACHE
        try:
            page = catalog.PRIMARY_PAGE if hasattr(catalog, "PRIMARY_PAGE") else catalog.SOURCE_PAGE
            section = catalog.PRIMARY_SECTION if hasattr(catalog, "PRIMARY_SECTION") else catalog.SECTION
            locator = f"gles32-pdf-v1:page={page};section={section}"
            authority = cache.SOURCE_API.authority().validate()
            admission = cache.SOURCE_API.authority().consume("command-state", locator)
            decision = admission["decision"]
            receipt, manifest = cache.inspect(cache_root, locator, "command-state"), cache.manifest()
            source = receipt["source"]
            raw = cache.pdf_bytes(cache.external_root(cache_root), source)
            if self.batch_snapshot is not None and self.batch_snapshot.proof is not None:
                domain, grammar, ledger = self.batch_snapshot.reuse(self, source, raw)
            else:
                domain = self.DOMAIN_API.validate(cache_root)
                grammar, ledger = self.GRAMMAR.validate(cache_root), self.LEDGER.validate()
                if self.batch_snapshot is not None:
                    self.batch_snapshot.remember(source, raw, domain, grammar, ledger)
            family_order = catalog.bound_family(self.DOMAIN_API.CHUNKS, self.ARTIFACT.document)
        except Exception as error:
            reject(str(error))
        exact = self.exact
        if (not exact(authority.get("normative_root"), source)
                or not exact(manifest.get("source"), source)
                or admission.get("locator") != locator or receipt.get("locator") != locator
                or decision.get("id") != "command-state" or source.get("profile") != catalog.PROFILE
                or receipt.get("physical_pdf_pages") != catalog.PAGES
                or not exact(domain.get("source"), source) or domain.get("source_class") != "command-state"
                or not exact(grammar.get("source"), source)
                or grammar.get("domain_classification_sha256") != domain.get("classification_sha256")
                or not exact(ledger.get("normative_root"), source)
                or ledger.get("ledger_sha256") != domain.get("unavailable_ledger_sha256")):
            reject("authority, cache, domain, grammar, or unavailable-ledger identity is not the exact GLES source root")
        return source, authority, decision, manifest, domain, grammar, ledger, family_order, raw
