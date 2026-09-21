#!/usr/bin/env python3
"""Focused positive and hostile checks for F03.2.3.4.1.3."""

from __future__ import annotations

import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "opengl_closed_anchor_classification.py"
LIVE_ROOT = Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f0323413_aggregate_test", PROBE)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


AGGREGATE = load()


class OpenGlClosedAnchorClassificationTests(unittest.TestCase):
    def live(self) -> Path:
        if not LIVE_ROOT.is_dir(): self.fail("retained external F03.2.2.1 cache is required")
        return LIVE_ROOT

    def copied(self):
        temporary = tempfile.TemporaryDirectory(); root = Path(temporary.name)
        for name in (AGGREGATE.RECEIPT.name, AGGREGATE.BUNDLE, *(item[1] for item in AGGREGATE.LAYOUT)):
            shutil.copyfile(HERE / name, root / name)
        return temporary, root / AGGREGATE.RECEIPT.name, root

    def rehash(self, path: Path, key: str, edit) -> None:
        value = json.loads(path.read_text(encoding="utf-8")); edit(value)
        value[key] = AGGREGATE.SOURCE.sha256({name: item for name, item in value.items() if name != key})
        path.write_text(json.dumps(value), encoding="utf-8")

    def test_exact_closed_source_only_result(self) -> None:
        value = AGGREGATE.validate(self.live()); expected, bundle, parts = AGGREGATE.rendered(self.live())
        self.assertEqual(value, expected)
        self.assertEqual((value["profile"], value["candidate_count"], value["semantic_fact_count"]),
                         ("opengl-4.6-core", 197, 0))
        self.assertEqual(value["route_counts"], {"covered": 0, "eligible-unreviewed": 162,
                         "route-to-state": 28, "shader-unadmitted": 3, "extension-unadmitted": 4,
                         "out-of-domain": 0})
        self.assertEqual(value["destination_counts"], {"F03.2.2.3.2": 28, "F03.2.3.2": 7,
                         "F03.2.3.4.2": 8, "F03.2.3.4.3": 100, "F03.2.3.4.4.1": 29,
                         "F03.2.3.4.5.1": 10, "F03.2.3.4.5.2": 4, "F03.2.3.4.5.3": 8,
                         "F03.2.3.4.5.4": 3})
        self.assertEqual(bundle["final_table_catalog"]["candidate_count"], 159)
        self.assertEqual(bundle["chapter_local_manifest"]["candidate_count"], 38)
        self.assertEqual([part["candidate_count"] for _, part in parts], [53, 53, 53, 38])
        self.assertFalse(any(value["claims"].values()))
        self.assertEqual((value["matrix_row_count"], value["cts_executions"]), (0, 0))

    def test_rehashed_claim_matrix_and_count_promotions_fail(self) -> None:
        edits = (lambda value: value.update(profile="gles-3.2"), lambda value: value.update(candidate_count=196),
                 lambda value: value["claims"].update(api_support=True), lambda value: value.update(matrix_row_count=1),
                 lambda value: value.update(cts_executions=1), lambda value: value.update(status="supported"))
        for edit in edits:
            temporary, receipt, root = self.copied()
            with temporary, self.assertRaises(AGGREGATE.SOURCE.AggregateError):
                self.rehash(receipt, "aggregate_sha256", edit); AGGREGATE.validate(self.live(), receipt, root)

    def test_exact_child_receipt_binding_rejects_a_rehashed_substitute(self) -> None:
        temporary, receipt, root = self.copied(); child = root / AGGREGATE.BUNDLE
        with temporary, self.assertRaises(AGGREGATE.SOURCE.AggregateError):
            self.rehash(child, "child_receipts_sha256", lambda value: value["final_table_catalog"]["claims"].update(api_support=True))
            digest = json.loads(child.read_text(encoding="utf-8"))["child_receipts_sha256"]
            self.rehash(receipt, "aggregate_sha256", lambda value: value["child_receipt_bundle"].update(child_receipts_sha256=digest))
            AGGREGATE.validate(self.live(), receipt, root)

    def test_rehashed_candidate_fragment_substitute_fails(self) -> None:
        temporary, receipt, root = self.copied(); fragment = root / AGGREGATE.LAYOUT[0][1]
        def edit(value) -> None:
            value["candidates"][0]["route_reason"] = "forged-fragment-route-reason"
            value["candidates_sha256"] = AGGREGATE.SOURCE.sha256(value["candidates"])
        with temporary, self.assertRaises(AGGREGATE.SOURCE.AggregateError):
            self.rehash(fragment, "fragment_sha256", edit); AGGREGATE.validate(self.live(), receipt, root)

    def test_duplicate_unanchored_and_promoted_candidate_rows_fail(self) -> None:
        _, _, catalog_rows, local_rows = AGGREGATE.child_data(self.live()); rows = AGGREGATE.rows(catalog_rows, local_rows)
        changes = (lambda value: value.__setitem__("candidate_id", rows[1]["candidate_id"]),
                   lambda value: value.__setitem__("source_locator", "gles32-core-pdf-v1:page=1;section=1"),
                   lambda value: value.__setitem__("owner", "matrix"))
        for change in changes:
            altered = [dict(item) for item in rows]; change(altered[0])
            with self.subTest(change=change), self.assertRaises(AGGREGATE.SOURCE.AggregateError): AGGREGATE.check_rows(altered)
        promoted = [dict(item) for item in catalog_rows]; promoted[0]["owner"] = "matrix"
        with self.assertRaises(AGGREGATE.SOURCE.AggregateError): AGGREGATE.rows(promoted, local_rows)

    def test_fixed_local_policy_and_bounded_artifact_output(self) -> None:
        _, _, catalog_rows, local_rows = AGGREGATE.child_data(self.live())
        policy = dict(AGGREGATE.POLICY.LOCAL_POLICY); policy["opengl46-local-table-8-2"] = ("eligible-unreviewed", "wildcard", "F03.2.3.4.4.1")
        with patch.object(AGGREGATE.POLICY, "LOCAL_POLICY", policy), self.assertRaises(AGGREGATE.SOURCE.AggregateError):
            AGGREGATE.rows(catalog_rows, local_rows)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); AGGREGATE.write_artifacts(self.live(), root)
            self.assertTrue(all(len(path.read_text(encoding="utf-8").splitlines()) <= 180 for path in root.iterdir()))
            self.assertEqual(AGGREGATE.validate(self.live(), root / AGGREGATE.RECEIPT.name, root)["candidate_count"], 197)

    def test_bootstrap_loader_rejects_symlink_and_wrong_resolved_module(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); bad = root / "bad.py"; bad.write_text("__file__ = 'unexpected.py'\n", encoding="utf-8")
            link = root / "link.py"; link.symlink_to(bad)
            with self.assertRaises(RuntimeError): AGGREGATE.fixed(link, "f0323413_bad_link")
            with self.assertRaises(RuntimeError): AGGREGATE.fixed(bad, "f0323413_bad_file")

    def test_fixed_loaders_ignore_and_restore_ambient_aliases(self) -> None:
        code = "\n".join((
            "import importlib.util, sys, types", "from pathlib import Path",
            "source = types.ModuleType('source'); policy = types.ModuleType('policy')",
            "sys.modules['f0323413_source'] = source; sys.modules['f0323413_policy'] = policy",
            f"spec = importlib.util.spec_from_file_location('f0323413_probe', {str(PROBE)!r})",
            "module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)",
            f"assert module.validate(Path({str(LIVE_ROOT)!r}))['candidate_count'] == 197",
            "assert sys.modules['f0323413_source'] is source; assert sys.modules['f0323413_policy'] is policy"))
        result = subprocess.run([sys.executable, "-c", code], cwd=tempfile.gettempdir(), text=True,
                                capture_output=True, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"})
        self.assertEqual(result.returncode, 0, msg=result.stderr)


if __name__ == "__main__": unittest.main()
