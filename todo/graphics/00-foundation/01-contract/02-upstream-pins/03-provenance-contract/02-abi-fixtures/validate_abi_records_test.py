"""Hermetic negative tests for the selected ABI provenance sidecars."""

from __future__ import annotations

import json
import shutil
import tempfile
import unittest
from pathlib import Path

import validate_abi_records as checker
from provenance_record import ProvenanceError

FAILURE = (checker.AbiRecordError, ProvenanceError)


class AbiRecordTests(unittest.TestCase):
    def copied_records(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory(prefix="webboxvm-abi-provenance-")
        records = Path(temporary.name) / "records"
        shutil.copytree(checker.RECORDS, records)
        return temporary, records

    def mutate(self, records: Path, name: str, change) -> None:
        path = records / name
        record = json.loads(path.read_text(encoding="utf-8"))
        change(record)
        path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")

    def test_selected_records_bind_current_artifacts(self) -> None:
        self.assertEqual(checker.validate_all(), 6)

    def test_unknown_input_is_rejected_before_artifact_acceptance(self) -> None:
        temporary, records = self.copied_records()
        with temporary:
            self.mutate(records, "guest-virgl-uapi.json", lambda record: record["inputs"][0].update(id="unknown"))
            with self.assertRaisesRegex(FAILURE, "unknown"):
                checker.validate_all(records)

    def test_known_but_wrong_input_is_rejected_by_selected_scope(self) -> None:
        temporary, records = self.copied_records()
        with temporary:
            self.mutate(records, "emulator-virgl-capset.json", self.replace_with_venus)
            with self.assertRaisesRegex(checker.AbiRecordError, "selected ABI scope"):
                checker.validate_all(records)

    def test_stale_manifest_digest_is_rejected_before_artifact_acceptance(self) -> None:
        temporary, records = self.copied_records()
        with temporary:
            self.mutate(records, "guest-webgpu-uapi.json", lambda record: record.update(manifest_sha256="f" * 64))
            with self.assertRaisesRegex(ProvenanceError, "stale manifest"):
                checker.validate_all(records)

    def test_dishonest_copied_origin_is_rejected(self) -> None:
        temporary, records = self.copied_records()
        with temporary:
            self.mutate(records, "guest-virgl-uapi.json", lambda record: record.update(artifact_kind="copied-upstream"))
            with self.assertRaisesRegex(ProvenanceError, "copied-upstream"):
                checker.validate_all(records)

    def test_changed_output_hash_is_rejected(self) -> None:
        temporary, records = self.copied_records()
        with temporary:
            self.mutate(records, "guest-virgl-wire.json", lambda record: record.update(output_sha256="f" * 64))
            with self.assertRaisesRegex(checker.AbiRecordError, "output_sha256"):
                checker.validate_all(records)

    def test_missing_selected_record_is_rejected(self) -> None:
        temporary, records = self.copied_records()
        with temporary:
            (records / "guest-virgl-kms.json").unlink()
            with self.assertRaisesRegex(checker.AbiRecordError, "record set"):
                checker.validate_all(records)

    @staticmethod
    def replace_with_venus(record: dict[str, object]) -> None:
        record["inputs"] = [{
            "id": "venus-protocol-registry",
            "sha256": "d92839bc728fa9ad9a7decdc6b91df6fa1a0fb26cffae4009865f18a789e0535",
            "license": "Apache-2.0 OR MIT (SPDX file notice)",
        }]


if __name__ == "__main__":
    unittest.main()
