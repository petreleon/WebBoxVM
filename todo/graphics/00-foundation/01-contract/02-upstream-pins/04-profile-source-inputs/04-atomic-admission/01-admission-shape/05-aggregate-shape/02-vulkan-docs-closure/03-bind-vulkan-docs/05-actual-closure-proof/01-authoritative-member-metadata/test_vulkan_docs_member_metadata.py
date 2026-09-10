#!/usr/bin/env python3
"""Hermetic checks for Docs metadata and its explicit derived blocker."""

from __future__ import annotations

import copy
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path: sys.path.insert(0, str(HERE))

from vulkan_docs_member_metadata import (
    DERIVED, DOCS_COMMIT, F02, GENERATION_ID, MetadataBlocked, MetadataError, NAMESPACE, _annotations,
    cache_name, derived_requirements, license_for, raw_member, require_derived_authority,
)
import vulkan_docs_member_snapshot as snapshot


def raw_record(selector: str, payload: bytes) -> dict[str, object]:
    return {"kind": "raw-observed-input", "selector": selector, "revision": DOCS_COMMIT,
            "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{DOCS_COMMIT}/{selector}",
            "sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload), "phase_roles": ["producer"]}


def derived_record() -> dict[str, object]:
    return {"kind": DERIVED, "selector": "generated/spec.adoc", "generation_id": GENERATION_ID,
            "sha256": "a" * 64, "bytes": 7, "phase_roles": ["generator"]}


class MemberMetadataTest(unittest.TestCase):
    def test_reifies_hermetic_raw_spdx_and_reuse_metadata(self) -> None:
        payload = b"// SPDX-License-Identifier: MIT\ncontent\n"
        values = {"REUSE.toml": b"version = 1\n", "member.adoc": payload}
        result = raw_member(raw_record("member.adoc", payload), values, _annotations(values))
        self.assertEqual((result["id"], result["license"]), ("raw-6d656d6265722e61646f63", "MIT"))
        reuse = {"REUSE.toml": b'version = 1\n[[annotations]]\npath = "plain.adoc"\nprecedence = "aggregate"\nSPDX-License-Identifier = "Apache-2.0"\n', "plain.adoc": b"content"}
        self.assertEqual(license_for(reuse, "plain.adoc", _annotations(reuse)), "Apache-2.0")

    def test_rejects_mutated_or_source_output_swapped_raw_record(self) -> None:
        payloads = {"REUSE.toml": b"version = 1\n", "member.adoc": b"one", "other.adoc": b"two"}
        record = raw_record("member.adoc", payloads["member.adoc"])
        changed = copy.deepcopy(record); changed["sha256"] = "1" * 64
        with self.assertRaises(MetadataError): raw_member(changed, payloads, _annotations(payloads))
        swapped = copy.deepcopy(record); swapped["selector"] = "other.adoc"
        swapped["immutable_url"] = f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{DOCS_COMMIT}/other.adoc"
        with self.assertRaises(MetadataError): raw_member(swapped, payloads, _annotations(payloads))

    def test_rejects_ambiguous_snapshot_license_evidence(self) -> None:
        values = {"REUSE.toml": b'version = 1\n[[annotations]]\npath = "member.adoc"\nprecedence = "aggregate"\nSPDX-License-Identifier = "Apache-2.0"\n', "member.adoc": b"// SPDX-License-Identifier: MIT\n"}
        with self.assertRaises(MetadataError): license_for(values, "member.adoc", _annotations(values))

    def test_license_uses_one_snapshot_without_a_later_source_reread(self) -> None:
        original = {"REUSE.toml": b"version = 1\n", "member.adoc": b"// SPDX-License-Identifier: MIT\n"}
        annotations = _annotations(original)
        changed_source = dict(original); changed_source["member.adoc"] = b"// SPDX-License-Identifier: Apache-2.0\n"
        self.assertEqual(license_for(original, "member.adoc", annotations), "MIT")
        self.assertEqual(license_for(changed_source, "member.adoc", _annotations(changed_source)), "Apache-2.0")

    def test_snapshot_uses_immutable_git_blobs_after_a_root_swap(self) -> None:
        payload, object_id = b"// SPDX-License-Identifier: MIT\n", "a" * 40
        rows = [{"selector": "member.adoc", "sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload), "mode": "100644"}]
        expected = (1, len(payload), hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(",", ":")).encode()).hexdigest())

        def capture(blob: bytes) -> tuple[dict[str, bytes], list[tuple[str, ...]]]:
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary); source, moved, replacement = root / "source", root / "moved", root / "replacement"
                source.mkdir(); replacement.mkdir(); calls: list[tuple[str, ...]] = []
                resolved_source = str(source.resolve())

                def runner(command: tuple[str, ...]) -> bytes:
                    calls.append(command)
                    if command[:3] == ("git", "-C", resolved_source):
                        return {("status", "--porcelain", "--ignored"): b"", ("rev-parse", "HEAD"): f"{DOCS_COMMIT}\n".encode(),
                                ("rev-parse", "--absolute-git-dir"): b"/sealed/repository\n"}[command[3:]]
                    if command[:2] != ("git", "--git-dir=/sealed/repository"): raise AssertionError(command)
                    if command[2:] == ("ls-tree", "-r", "-z", DOCS_COMMIT):
                        source.rename(moved); source.symlink_to(replacement, target_is_directory=True)
                        return f"100644 blob {object_id}\tmember.adoc\0".encode()
                    if command[2:] == ("cat-file", "blob", object_id): return blob
                    raise AssertionError(command)

                with patch.object(snapshot, "SOURCE_TREE", expected), patch.object(Path, "read_bytes", side_effect=AssertionError("worktree reread")):
                    return snapshot.source_snapshot(source, runner), calls

        values, calls = capture(payload)
        self.assertEqual(values, {"member.adoc": payload})
        self.assertEqual(sum(call[1] == "-C" for call in calls), 3)
        self.assertTrue(all(call[:2] == ("git", "--git-dir=/sealed/repository") for call in calls[3:]))
        with self.assertRaises(snapshot.SnapshotError): capture(b"swapped component")

    def test_derived_static_keys_do_not_fill_missing_authority(self) -> None:
        record = derived_record(); requirements = derived_requirements([record]); identifier = "derived-67656e6572617465642f737065632e61646f63"
        self.assertEqual(requirements[0], {"kind": "derived-source-input", "id": identifier,
                         "generation_id": GENERATION_ID, "selector": "generated/spec.adoc", "sha256": "a" * 64,
                         "bytes": 7, "local_cache": cache_name("derived-source-input", identifier, "a" * 64)})
        self.assertEqual(requirements[0]["local_cache"], f"{NAMESPACE}/derived/{identifier}/{'a' * 64}.derived")
        with self.assertRaisesRegex(MetadataBlocked, "license, role, provenance, and producer"):
            require_derived_authority(requirements)
        stale = copy.deepcopy(record); stale["generation_id"] = "stale"
        with self.assertRaises(MetadataError): derived_requirements([stale])
        oversized = copy.deepcopy(record); oversized["bytes"] = F02.MAX_INPUT_BYTES + 1
        with self.assertRaises(MetadataError): derived_requirements([oversized])
        invalid = copy.deepcopy(record); invalid["bytes"] = True
        with self.assertRaises(MetadataError): derived_requirements([invalid])


if __name__ == "__main__": unittest.main(verbosity=2)
