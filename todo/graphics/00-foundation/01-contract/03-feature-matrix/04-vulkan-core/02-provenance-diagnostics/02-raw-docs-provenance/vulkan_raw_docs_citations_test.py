#!/usr/bin/env python3
"""Focused hostile checks for F03.4.2.2 raw Docs citation candidates."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "vulkan_raw_docs_citations.py"


def load():
    spec = importlib.util.spec_from_file_location("f03422_test_map", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


AUTH, CACHE = load(), None
CACHE = AUTH.CACHE


class RawDocsCitationTests(unittest.TestCase):
    def inputs(self):
        return ({"source_contract_sha256": "a" * 64, "inventory_lock_sha256": "b" * 64}, AUTH.EXPECTED_BINDING,
                {"boundary_sha256": AUTH.BOUNDARY_SHA256}, {"mode": "sealed-external-reviewed-raw-fragments",
                "file_count": 4, "manifest_sha256": "c" * 64})

    def built(self) -> dict[str, object]:
        with patch.object(AUTH, "source_inputs", return_value=self.inputs()):
            return AUTH.build(Path("/private/tmp/f03422-unit"))

    def copied(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory(); path = Path(temporary.name) / "map.json"
        path.write_text(json.dumps(self.built()), encoding="utf-8")
        return temporary, path

    def checked(self, path: Path) -> dict[str, object]:
        with patch.object(AUTH, "source_inputs", return_value=self.inputs()):
            return AUTH.validate(path, Path("/private/tmp/f03422-unit"))

    def mutate(self, path: Path, edit, rehash: bool = False) -> None:
        value = json.loads(path.read_text(encoding="utf-8")); edit(value)
        if rehash:
            body = {key: item for key, item in value.items() if key != "map_sha256"}
            value["map_sha256"] = hashlib.sha256(AUTH.canonical(body)).hexdigest()
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_candidate_map_has_no_core_coverage_claim_or_matrix_rows(self) -> None:
        value = self.built()
        self.assertEqual((value["citation_candidate_count"], value["scope"]["include_closure"]), (2, "unadmitted"))
        self.assertTrue(all(item["semantic_status"] == "citation-candidate-only" and item["core_coverage"] == "unassigned"
                            and item["relation"].endswith("inclusion-unproven")
                            and item["raw_fragment"]["immutable_url"].endswith(item["raw_fragment"]["raw_path"])
                            for item in value["citation_candidates"]))
        self.assertEqual((value["matrix_rows"], value["cts_executions"], value["scope"]["matrix_row_count"]), ([], 0, 0))
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual(value["scope"]["registry_structural_fact"]["source_locator"], None)

    def test_self_hash_root_anchor_license_registry_and_promotions_fail(self) -> None:
        edits = (
            (lambda value: value["citation_candidates"][0]["normative_root"].update(revision="0" * 40), True),
            (lambda value: value["citation_candidates"][0]["raw_fragment"].update(semantic_anchor="[[guessed-symbol]]"), True),
            (lambda value: value["citation_candidates"][0]["raw_fragment"].update(license="Apache-2.0"), True),
            (lambda value: value["scope"]["registry_structural_fact"].update(source_locator="xml/vk.xml#guessed"), True),
            (lambda value: value["scope"].update(matrix_row_count=1), True),
            (lambda value: value["claims"].update(api_support=True), True),
            (lambda value: value["citation_candidates"][0].update(core_coverage="covered"), True),
            (lambda value: value.update(map_sha256="0" * 64), False),
        )
        for edit, rehash in edits:
            temporary, path = self.copied()
            with temporary, self.subTest(edit=edit), self.assertRaises(AUTH.CitationError):
                self.mutate(path, edit, rehash); self.checked(path)

    def test_fixed_boundary_is_checked_before_the_cache(self) -> None:
        with patch.object(AUTH.CACHE, "cache", return_value=self.inputs()[3]):
            contract, binding, boundary, _cache = AUTH.source_inputs(Path("/private/tmp/f03422-unit"))
        self.assertEqual((contract["source_contract_sha256"], binding, boundary["boundary_sha256"]),
                         ("d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3", AUTH.EXPECTED_BINDING,
                          AUTH.BOUNDARY_SHA256))

    def test_cache_rejects_mixed_manifest_generated_path_and_symlinked_payload(self) -> None:
        root_data, fragment_data = b"// SPDX-License-Identifier: CC-BY-4.0\nroot\n", b"// SPDX-License-Identifier: CC-BY-4.0\n[[exact]]\n"
        root = {**CACHE.ROOT, "revision": "a" * 40, "sha256": hashlib.sha256(root_data).hexdigest(), "bytes": len(root_data)}
        fragment = {"id": "fragment", "root_id": "vulkan-14-spec", "revision": "a" * 40, "raw_path": "chapters/exact.adoc",
                    "sha256": hashlib.sha256(fragment_data).hexdigest(), "bytes": len(fragment_data),
                    "license": "CC-BY-4.0 (raw fragment SPDX-License-Identifier)", "attribution": "Copyright test; CC-BY-4.0",
                    "semantic_anchor": "[[exact]]"}
        with tempfile.TemporaryDirectory(dir="/private/tmp") as temporary, patch.object(CACHE, "ROOT", root), patch.object(CACHE, "FRAGMENTS", (fragment,)):
            cache_root = Path(temporary); target = cache_root / fragment["raw_path"]
            target.parent.mkdir(); (cache_root / "vkspec.adoc").write_bytes(root_data); target.write_bytes(fragment_data)
            manifest = cache_root / "citation-cache.json"; manifest.write_bytes(CACHE.canonical(CACHE.manifest()))
            self.assertEqual(CACHE.cache(cache_root)["file_count"], 3)
            mixed = json.loads(manifest.read_text(encoding="utf-8")); mixed["root"]["revision"] = "b" * 40
            manifest.write_text(json.dumps(mixed), encoding="utf-8")
            with self.assertRaises(CACHE.CitationSourceError): CACHE.cache(cache_root)
            with self.assertRaises(CACHE.CitationSourceError): CACHE.raw_path("generated/api.adoc")
            for bytes_value in (b"// SPDX-License-Identifier: CC-BY-4.0\n[[other]]\n", b"// SPDX-License-Identifier: CC-BY-4.0\n[[exact]]\n[[exact]]\n"):
                altered = {**fragment, "sha256": hashlib.sha256(bytes_value).hexdigest(), "bytes": len(bytes_value)}
                with patch.object(CACHE, "FRAGMENTS", (altered,)):
                    target.write_bytes(bytes_value); manifest.write_bytes(CACHE.canonical(CACHE.manifest()))
                    with self.assertRaises(CACHE.CitationSourceError): CACHE.cache(cache_root)
            target.write_bytes(fragment_data)
            manifest.write_bytes(CACHE.canonical(CACHE.manifest()))
            target.unlink(); outside = cache_root / "outside.adoc"; outside.write_bytes(fragment_data); os.symlink(outside, target)
            with self.assertRaises(CACHE.CitationSourceError): CACHE.secure_bytes(target, "symlinked payload")
            with self.assertRaises(CACHE.CitationSourceError): CACHE.cache(cache_root)
            fifo = cache_root / "nonregular"; os.mkfifo(fifo)
            with self.assertRaises(CACHE.CitationSourceError): CACHE.secure_bytes(fifo, "FIFO payload")

    def test_fixed_loaders_ignore_ambient_sibling_decoys(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            for name in ("vulkan_raw_docs_citation_sources.py", "vulkan_raw_docs_citation_cache.py"):
                (Path(temporary) / name).write_text("raise RuntimeError('ambient decoy loaded')\n", encoding="utf-8")
            code = "\n".join(("import importlib.util,os,sys", f"sys.path.insert(0,{temporary!r}); os.chdir({temporary!r})",
                                f"spec=importlib.util.spec_from_file_location('map',{str(PROBE)!r})",
                                "module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
                                f"assert module.SOURCE.__file__ == {str(HERE / 'vulkan_raw_docs_citation_sources.py')!r}",
                                f"assert module.CACHE.__file__ == {str(HERE / 'vulkan_raw_docs_citation_cache.py')!r}"))
            result = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True,
                                    env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_validation_requires_an_external_cache(self) -> None:
        temporary, path = self.copied()
        with temporary, self.assertRaisesRegex(AUTH.CitationError, "external reviewed raw Docs cache"):
            AUTH.validate(path)


if __name__ == "__main__":
    unittest.main()
