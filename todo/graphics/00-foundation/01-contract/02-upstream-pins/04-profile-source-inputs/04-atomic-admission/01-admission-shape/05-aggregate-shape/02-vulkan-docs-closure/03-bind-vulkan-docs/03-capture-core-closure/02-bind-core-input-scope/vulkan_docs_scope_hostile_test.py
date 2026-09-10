#!/usr/bin/env python3
"""Hostile regressions for unsafe, expanded, or partial core input scopes."""

from __future__ import annotations

from dataclasses import replace
import tempfile
import unittest
from pathlib import Path

from vulkan_docs_scope_bind import _bind_fixture, artifact_file
from vulkan_docs_scope_contract import compact, receipt_value
from vulkan_docs_scope_fixture import expectation, refresh, row, valid
from vulkan_docs_scope_model import DERIVED, EXTENSION_CONTROLS, PROMOTIONS, RAW, ScopeError
from vulkan_docs_scope_parse import canonical, document


def drop(value: dict[str, object], selector: str) -> None:
    value["records"] = [item for item in value["records"] if item["selector"] != selector]
    value["includes"] = [item for item in value["includes"] if item["selector"] != selector]


class ScopeHostileTest(unittest.TestCase):
    def rejected(self, mutate, refresh_capture: bool = False, message: str | None = None) -> None:
        value, capture = valid()
        mutate(value)
        if refresh_capture:
            refresh(value)
            capture = expectation(value)
        with self.assertRaises(ScopeError) as error:
            _bind_fixture(value, capture)
        if message:
            self.assertIn(message, str(error.exception))

    def test_root_only_or_required_scope_omissions_are_rejected(self):
        self.rejected(lambda value: (value.__setitem__("records", value["records"][:1]), value.__setitem__("includes", [])), True)
        for _, selector in (*EXTENSION_CONTROLS, *PROMOTIONS):
            with self.subTest(selector=selector):
                self.rejected(lambda value, name=selector: drop(value, name), True)
        self.rejected(lambda value: drop(value, "images/fixture-00.svg"), True)

    def test_duplicates_malformed_or_oversize_records_are_rejected(self):
        self.rejected(lambda value: value["records"].append(dict(value["records"][0])), True)
        self.rejected(lambda value: value["records"][0].__setitem__("selector", "../escape.adoc"))
        self.rejected(lambda value: value["records"][0].__setitem__("phase_roles", ["unknown"]))
        self.rejected(lambda value: value["records"][0].__setitem__("bytes", 8 * 1024 * 1024 + 1))
        self.rejected(lambda value: value.__setitem__("records", list(reversed(value["records"]))))

    def test_namespace_output_and_expanded_extension_inputs_are_rejected(self):
        for kind, selector in (
                (RAW, "generated/not-a-raw-source.adoc"), (DERIVED, "not-generated.adoc"),
                (DERIVED, "generated/out/html/injected.html"), (RAW, "chapters/VK_FAKE/extension.adoc"),
                (RAW, "appendices/VK_FAKE.adoc"), (RAW, "chapters/descriptorheaps.adoc"),
                (RAW, "chapters/looks-safe.adoc?mutable=1"), (RAW, "chapters/looks-safe.adoc#fragment")):
            with self.subTest(selector=selector):
                self.rejected(lambda value, item=row(kind, selector): value["records"].append(item), True)
        self.rejected(lambda value: value["records"].append(
            row(DERIVED, "generated/api/protos/vkGetPhysicalDeviceExternalImageFormatPropertiesNV.adoc")), True,
            "individual extension semantics")

    def test_wsi_video_and_stale_root_or_producer_are_rejected(self):
        for selector in ("chapters/VK_KHR_surface/wsi.adoc", "chapters/videocoding.adoc"):
            with self.subTest(selector=selector):
                self.rejected(lambda value, item=row(RAW, selector): value["records"].append(item), True)
        self.rejected(lambda value: value["records"].__setitem__(next(i for i, item in enumerate(value["records"])
                                                                          if item["selector"] == "vkspec.adoc"),
                                                                       row(RAW, "vkspec.adoc")), True)
        value, capture = valid()
        with self.assertRaises(ScopeError):
            _bind_fixture(value, replace(capture, producer_digest="f" * 64))

    def test_resealed_receipt_cannot_change_configuration_or_state(self):
        value, capture = valid()
        scope = _bind_fixture(value, capture)
        receipt = compact(scope)
        receipt["configuration_sha256"] = "f" * 64
        payload = dict(receipt)
        payload.pop("scope_receipt_sha256", None)
        receipt["scope_receipt_sha256"] = canonical(payload, "webboxvm-graphics-vulkan-docs-core-scope-receipt-v1")
        with self.assertRaises(ScopeError):
            receipt_value(receipt, scope)

    def test_duplicate_json_and_unsafe_artifact_paths_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "duplicate.json"
            path.write_text('{"records":[],"records":[]}', encoding="utf-8")
            with self.assertRaises(ScopeError):
                document(path)
            with self.assertRaises(ScopeError):
                artifact_file(Path(directory), "runs/../escape", "run.json")


if __name__ == "__main__":
    unittest.main()
