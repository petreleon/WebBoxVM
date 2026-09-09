#!/usr/bin/env python3
"""Validate F02.1's offline immutable graphics-input inventory."""

from __future__ import annotations

import argparse
import copy
import hashlib
import re
import sys
import tomllib
import unittest
from pathlib import Path, PurePosixPath
from urllib.parse import urlsplit

HERE = Path(__file__).resolve().parent
MANIFEST = HERE / "manifest.toml"
FIELDS = frozenset(("id", "source_family", "immutable_url", "revision", "sha256", "bytes", "license", "local_cache", "generated_code_role", "provenance"))
FAMILIES = frozenset(("linux-uapi", "mesa-virgl", "mesa-venus", "virglrenderer", "venus-protocol", "gl-gles-registry", "glsl", "essl", "vulkan", "spirv", "webgpu", "wgsl", "vk-gl-cts", "webgpu-cts", "piglit"))
COMMIT = re.compile(r"^[0-9a-f]{40}$")
DIGEST = re.compile(r"^[0-9a-f]{64}$")
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9-]*$")
MUTABLE = frozenset(("head", "latest", "main", "master", "stable", "trunk"))


def fail(message: str) -> None:
    raise ValueError(message)


def string(value: object, field: str) -> str:
    if not isinstance(value, str) or not value:
        fail(f"{field} must be a nonempty string")
    return value


def validate(document: object) -> int:
    if not isinstance(document, dict):
        fail("manifest must be a TOML table")
    expected = {"schema", "cache_root", "cache_note", "required_families", "inputs"}
    if set(document) != expected or document["schema"] != 1:
        fail("manifest schema is not version 1")
    if document["cache_root"] != "$XDG_CACHE_HOME" or not string(document["cache_note"], "cache_note"):
        fail("cache must resolve below external $XDG_CACHE_HOME")
    declared = document["required_families"]
    if not isinstance(declared, list) or set(declared) != FAMILIES:
        fail("required source-family inventory is incomplete")
    entries = document["inputs"]
    if not isinstance(entries, list) or not entries:
        fail("inputs must be a nonempty array")
    ids, families = set(), set()
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict) or set(entry) != FIELDS:
            fail(f"input {index} does not match the maintained schema")
        identifier = string(entry["id"], "id")
        family = string(entry["source_family"], "source_family")
        revision = string(entry["revision"], "revision")
        digest = string(entry["sha256"], "sha256")
        if not IDENTIFIER.fullmatch(identifier) or identifier in ids:
            fail(f"input {index} has a duplicate or invalid id")
        if not COMMIT.fullmatch(revision):
            fail(f"input {identifier} has a non-commit revision")
        if not DIGEST.fullmatch(digest) or digest == "0" * 64:
            fail(f"input {identifier} has an invalid sha256")
        if not isinstance(entry["bytes"], int) or entry["bytes"] <= 0:
            fail(f"input {identifier} has invalid byte count")
        for field in ("license", "generated_code_role"):
            string(entry[field], field)
        provenance = urlsplit(string(entry["provenance"], "provenance"))
        if provenance.scheme != "https" or not provenance.netloc:
            fail(f"input {identifier} has invalid provenance URL")
        url = urlsplit(string(entry["immutable_url"], "immutable_url"))
        parts = tuple(part.lower() for part in url.path.split("/") if part)
        if url.scheme != "https" or not url.netloc or revision not in url.path:
            fail(f"input {identifier} has no immutable HTTPS source URL")
        if MUTABLE.intersection(parts):
            fail(f"input {identifier} has a mutable URL reference")
        cache = PurePosixPath(string(entry["local_cache"], "local_cache"))
        if cache.is_absolute() or ".." in cache.parts or cache.parts[:2] != ("webboxvm-graphics", "f02"):
            fail(f"input {identifier} has a repository-relative or unsafe cache")
        if identifier not in cache.parts or not str(cache).endswith(f"/{digest}.source"):
            fail(f"input {identifier} cache does not bind its byte identity")
        ids.add(identifier)
        families.add(family)
    if families != FAMILIES or len(ids) != len(entries):
        fail("source-family coverage is incomplete or duplicated")
    return len(entries)


def load(path: Path = MANIFEST) -> tuple[dict[str, object], str]:
    raw = path.read_bytes()
    try:
        document = tomllib.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        fail(f"manifest cannot be parsed: {error}")
    if not isinstance(document, dict):
        fail("manifest must be a TOML table")
    return document, hashlib.sha256(raw).hexdigest()


class InventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.document, self.revision = load()

    def reject(self, change, message: str) -> None:
        document = copy.deepcopy(self.document)
        change(document)
        with self.assertRaisesRegex(ValueError, message):
            validate(document)

    def test_reviewed_inventory_is_valid(self) -> None:
        self.assertEqual(validate(self.document), len(FAMILIES))
        self.assertRegex(self.revision, r"^[0-9a-f]{64}$")

    def test_missing_family_is_rejected(self) -> None:
        self.reject(lambda data: data["inputs"].pop(), "source-family coverage")

    def test_bad_digest_is_rejected(self) -> None:
        self.reject(lambda data: data["inputs"][0].update(sha256="0" * 64), "invalid sha256")

    def test_mutable_source_reference_is_rejected(self) -> None:
        def make_mutable(data):
            entry = data["inputs"][0]
            entry["immutable_url"] = entry["immutable_url"].replace(entry["revision"], "main")
        self.reject(make_mutable, "immutable HTTPS source URL")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true", help="run hermetic structural tests")
    if parser.parse_args().self_test:
        unittest.main(argv=[sys.argv[0]])
        return
    document, revision = load()
    print(f"PASS: {validate(document)} immutable inputs; manifest sha256={revision}")


if __name__ == "__main__":
    main()
