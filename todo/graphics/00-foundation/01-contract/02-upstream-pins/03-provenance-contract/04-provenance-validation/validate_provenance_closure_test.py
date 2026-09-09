#!/usr/bin/env python3
"""Hermetic tests for the F02.2-cache to F02.3-record closure."""

from __future__ import annotations

import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[6]
FIXTURES = HERE.parents[1] / "02-fetch-verifier" / "02-hermetic-fixtures"
FETCH = HERE.parents[1] / "02-fetch-verifier" / "01-fetch-contract"
sys.path[:0] = [str(FETCH), str(FIXTURES)]

from fixture_transport import PAYLOAD, entry as fixture_entry, source as fixture_source
from inventory_layout import FAMILIES, load_inventory, render_v2_lock
from source_model import ExternalCache, load_manifest
from validate_provenance_closure import (
    CONTRACT,
    MANIFEST,
    REPOSITORY,
    ClosureError,
    Sample,
    SAMPLES,
    fingerprint,
    validate_closure,
    validate_records,
    verify_cached_inputs,
)


class ClosureTests(unittest.TestCase):
    def copied_contract(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory(dir=ROOT)
        contract = Path(temporary.name) / "contract"
        shutil.copytree(CONTRACT, contract)
        return temporary, contract

    def mutate(self, contract: Path, relative: str, change) -> None:
        path = contract / relative
        value = json.loads(path.read_text(encoding="utf-8"))
        change(value)
        path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")

    def fixture_closure(self, temporary: Path):
        inventory, repository, contract = temporary / "inventory", temporary / "repo", temporary / "contract"
        manifest, entries = inventory / "manifest.toml", []
        for number, family in enumerate(sorted(FAMILIES)):
            entry = fixture_entry(id=f"fixture-{number}", source_family=family)
            entry["local_cache"] = f"webboxvm-graphics/f02/{entry['id']}/{entry['sha256']}.source"
            entries.append(entry)
        part = inventory / "inputs/part-0001.toml"
        part.parent.mkdir(parents=True)
        manifest.write_text(
            'schema = 2\ncache_root = "$XDG_CACHE_HOME"\ncache_note = "fixture"\n'
            f"required_families = {json.dumps(sorted(FAMILIES))}\ninput_files = [\"inputs/part-0001.toml\"]\n",
            encoding="utf-8")
        part.write_text("\n".join("[[inputs]]\n" + "\n".join(
            f"{field} = {json.dumps(value) if isinstance(value, str) else value}" for field, value in entry.items())
            for entry in entries) + "\n", encoding="utf-8")
        (inventory / "inventory.lock").write_bytes(render_v2_lock(manifest))
        artifact = repository / "artifact.txt"
        artifact.parent.mkdir()
        artifact.write_bytes(b"fixture output")
        first = entries[0]
        record = {
            "schema": 2, "inventory_sha256": load_inventory(manifest).revision,
            "inputs": [{field: first[field] for field in ("id", "sha256", "license")}],
            "command": "fixture closure", "generator": {"name": "none", "version": "none"},
            "artifact_kind": "handwritten", "artifact_path": "artifact.txt",
            "output_sha256": hashlib.sha256(artifact.read_bytes()).hexdigest(),
        }
        sidecar = contract / "02-abi-fixtures/records/fixture.json"
        sidecar.parent.mkdir(parents=True)
        sidecar.write_text(json.dumps(record), encoding="utf-8")
        sample = Sample("02-abi-fixtures/records/fixture.json", "artifact.txt", "artifact.txt", fingerprint(record))
        return manifest, repository, contract, temporary / "cache", (sample,), artifact

    def test_current_catalog_binds_all_sidecars_and_outputs(self) -> None:
        self.assertEqual(validate_records(MANIFEST, REPOSITORY), 12)

    def test_closure_rehashes_fixture_cache_end_to_end(self) -> None:
        with tempfile.TemporaryDirectory(dir=ROOT) as temporary:
            manifest, repository, contract, cache_root, samples, artifact = self.fixture_closure(Path(temporary))
            cache, sources = ExternalCache.from_path(cache_root, repository), load_manifest(manifest)
            for source in sources:
                target = cache.target(source)
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(PAYLOAD)
            kwargs = {"repository": repository, "contract": contract, "samples": samples}
            self.assertEqual(validate_closure(manifest, cache_root, **kwargs), (len(sources), 1))
            artifact.write_bytes(b"wrong")
            with self.assertRaisesRegex(ClosureError, "output_sha256"):
                validate_closure(manifest, cache_root, **kwargs)
            artifact.write_bytes(b"fixture output")
            target = cache.target(sources[0])
            target.unlink()
            with self.assertRaisesRegex(ClosureError, "unavailable"):
                validate_closure(manifest, cache_root, **kwargs)
            target.write_bytes(PAYLOAD[:-1] + b"!")
            with self.assertRaisesRegex(ClosureError, "sha256 mismatch"):
                validate_closure(manifest, cache_root, **kwargs)

    def test_cache_target_rejects_symlink_escape_and_nonregular_entries(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            cache = ExternalCache.from_path(Path(temporary) / "cache", REPOSITORY)
            source = fixture_source()
            target = cache.target(source)
            target.parent.mkdir(parents=True)
            target.symlink_to(Path(temporary) / "payload")
            with self.assertRaisesRegex(ClosureError, "symlink"):
                verify_cached_inputs(cache, (source,))
            target.unlink()
            target.mkdir()
            with self.assertRaisesRegex(ClosureError, "not a regular file"):
                verify_cached_inputs(cache, (source,))
            escaped = ExternalCache.from_path(Path(temporary) / "escaped", REPOSITORY)
            escaped.root.mkdir()
            (escaped.root / "webboxvm-graphics").symlink_to(Path(temporary) / "outside")
            with self.assertRaisesRegex(ClosureError, "symlink"):
                verify_cached_inputs(escaped, (source,))
            internal = ExternalCache.from_path(Path(temporary) / "internal", REPOSITORY)
            internal.root.mkdir()
            redirected = internal.root / "inside" / Path(*source.local_cache.parts[1:])
            redirected.parent.mkdir(parents=True)
            redirected.write_bytes(PAYLOAD)
            (internal.root / "webboxvm-graphics").symlink_to(internal.root / "inside")
            with self.assertRaisesRegex(ClosureError, "symlink"):
                verify_cached_inputs(internal, (source,))

    def test_abi_and_gl_command_drift_are_rejected(self) -> None:
        for sample in (SAMPLES[0], SAMPLES[7]):
            with self.subTest(sidecar=sample.sidecar):
                temporary, contract = self.copied_contract()
                with temporary:
                    self.mutate(contract, sample.sidecar, lambda record: record.update(command="wrong"))
                    with self.assertRaisesRegex(ClosureError, "fingerprint"):
                        validate_records(MANIFEST, REPOSITORY, contract)

    def test_stale_input_license_is_rejected_before_output_acceptance(self) -> None:
        temporary, contract = self.copied_contract()
        with temporary:
            self.mutate(contract, SAMPLES[0].sidecar,
                        lambda record: record["inputs"][0].update(license="wrong"))
            with self.assertRaisesRegex(ClosureError, "stale license"):
                validate_records(MANIFEST, REPOSITORY, contract)

    def test_missing_catalog_sidecar_is_rejected(self) -> None:
        temporary, contract = self.copied_contract()
        with temporary:
            (contract / SAMPLES[-1].sidecar).unlink()
            with self.assertRaisesRegex(ClosureError, "record set"):
                validate_records(MANIFEST, REPOSITORY, contract)

    def test_unexpected_catalog_sidecar_is_rejected(self) -> None:
        temporary, contract = self.copied_contract()
        with temporary:
            (contract / SAMPLES[0].sidecar).with_name("unexpected.json").write_text('{"schema": 1}\n')
            with self.assertRaisesRegex(ClosureError, "record set"):
                validate_records(MANIFEST, REPOSITORY, contract)


if __name__ == "__main__":
    unittest.main()
