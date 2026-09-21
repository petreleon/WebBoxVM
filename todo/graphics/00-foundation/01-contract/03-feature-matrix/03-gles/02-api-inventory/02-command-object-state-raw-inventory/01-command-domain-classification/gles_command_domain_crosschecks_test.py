#!/usr/bin/env python3
"""Adversarial closure, anchor, and index checks for F03.3.2.2.1."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import subprocess
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
PROBE = HERE / "gles_command_domain_classification.py"
LIVE_ROOT = Path("/private/tmp/webboxvm-f0341.cqT6ZX")


def load():
    spec = importlib.util.spec_from_file_location("f033221_crosscheck_test", PROBE)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {PROBE}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


MAP = load()


class GlesCommandDomainCrosscheckTests(unittest.TestCase):
    def live(self):
        if not MAP.CACHE.cache_file(LIVE_ROOT, MAP.CACHE.SOURCE).is_file():
            self.fail("retained external F03.3.2.1 cache is required")
        return LIVE_ROOT

    def changed(self, identifier, field, value):
        rows = list(MAP.RULES.FAMILIES)
        index = next(index for index, row in enumerate(rows) if row[2] == identifier)
        row = list(rows[index]); row[field] = value; rows[index] = tuple(row)
        return tuple(rows)

    def rendered_rejects(self, rows):
        with patch.object(MAP.RULES, "FAMILIES", rows), patch.object(MAP.CROSSCHECKS, "CLOSED_FAMILIES", rows):
            MAP.rendered(self.live())

    def test_closed_manifest_rejects_missing_scope_and_reroute_before_rehash(self):
        rows = tuple(row for row in MAP.RULES.FAMILIES if row[2] != "indirect-draw")
        with patch.object(MAP.RULES, "FAMILIES", rows), self.assertRaises(MAP.ClassificationError):
            MAP.rendered(self.live())
        for field, value in ((3, "10.3.8"), (4, "buffer-commands")):
            with self.subTest(field=field), self.assertRaises(MAP.ClassificationError):
                self.rendered_rejects(self.changed("indirect-draw", field, value))

    def test_empty_prefix_and_wrong_section_anchors_reject_on_rerender(self):
        for field, value in ((8, ""), (8, "GetError"), (7, "999.999")):
            with self.subTest(field=field), self.assertRaises(MAP.ClassificationError):
                self.rendered_rejects(self.changed("generic-context", field, value))

    def test_scope_forms_and_index_witnesses_fail_closed(self):
        for scope in ("CATCH-ALL", "all-sources", "3-2"):
            with self.subTest(scope=scope), self.assertRaises(MAP.ClassificationError):
                MAP.CROSSCHECKS.scope_parts(scope, MAP.reject)
        raw = MAP.source_input(self.live())[-1]
        command = ["pdftotext", "-f", "571", "-l", "601", "-layout", "-", "-"]
        actual = subprocess.run(command, input=raw, capture_output=True, check=False)
        short = types.SimpleNamespace(returncode=0, stdout=b"x\f" * 30)
        injected = types.SimpleNamespace(returncode=0, stdout=actual.stdout.replace(b"\f", b" FenceSync\f", 1))
        for result in (short, injected):
            with patch.object(MAP.CROSSCHECKS.subprocess, "run", return_value=result), self.assertRaises(MAP.ClassificationError):
                MAP.CROSSCHECKS.index_crosscheck(raw, MAP.reject)
        wrong = list(MAP.CROSSCHECKS.INDEX_WITNESSES)
        wrong[1] = (*wrong[1][:3], "buffer-commands")
        with patch.object(MAP.CROSSCHECKS, "INDEX_WITNESSES", tuple(wrong)), self.assertRaises(MAP.ClassificationError):
            MAP.CROSSCHECKS.index_crosscheck(raw, MAP.reject)

    def test_rehashed_index_receipt_cannot_be_substituted(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / MAP.INDEX.name
            value = json.loads(MAP.INDEX.read_text(encoding="utf-8"))
            value["index_omission_crosscheck"]["role"] = "api-support"
            body = {key: item for key, item in value.items() if key != "index_sha256"}
            value["index_sha256"] = hashlib.sha256(MAP.canonical(body)).hexdigest()
            path.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(MAP.ClassificationError):
                MAP.validate(self.live(), index_path=path)


if __name__ == "__main__":
    unittest.main()
