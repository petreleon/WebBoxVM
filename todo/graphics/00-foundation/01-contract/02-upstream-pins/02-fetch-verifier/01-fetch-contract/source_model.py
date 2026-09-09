"""Offline manifest and external-cache checks for the F02 fetch contract."""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from urllib.parse import urlsplit

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "01-input-inventory"))
from inventory_layout import FAMILIES as REQUIRED_FAMILIES, InventoryLayoutError, load_inventory

COMMIT = re.compile(r"^[0-9a-f]{40}$")
DIGEST = re.compile(r"^[0-9a-f]{64}$")
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9-]*$")
MUTABLE_REFS = frozenset(("head", "latest", "main", "master", "stable", "trunk"))
MAX_INPUT_BYTES = 8 * 1024 * 1024
INPUT_FIELDS = frozenset(("id", "source_family", "immutable_url", "revision", "sha256", "bytes", "license", "local_cache", "generated_code_role", "provenance"))


class ContractError(ValueError):
    """A source cannot be fetched or accepted under the F02 contract."""


def reject(message: str) -> None:
    raise ContractError(message)


def text(value: object, name: str) -> str:
    if not isinstance(value, str) or not value:
        reject(f"{name} must be a nonempty string")
    return value


@dataclass(frozen=True)
class SourceInput:
    identifier: str
    url: str
    revision: str
    sha256: str
    byte_count: int
    local_cache: PurePosixPath

    @classmethod
    def from_manifest(cls, entry: object) -> "SourceInput":
        if not isinstance(entry, dict) or set(entry) != INPUT_FIELDS:
            reject("input does not match the maintained manifest schema")
        identifier = text(entry["id"], "id")
        revision = text(entry["revision"], "revision")
        digest = text(entry["sha256"], "sha256")
        if not IDENTIFIER.fullmatch(identifier):
            reject(f"input {identifier!r} has an invalid id")
        if not COMMIT.fullmatch(revision):
            reject(f"input {identifier} has a non-commit revision")
        if not DIGEST.fullmatch(digest) or digest == "0" * 64:
            reject(f"input {identifier} has an invalid sha256")
        if (not isinstance(entry["bytes"], int) or isinstance(entry["bytes"], bool)
                or not 0 < entry["bytes"] <= MAX_INPUT_BYTES):
            reject(f"input {identifier} has an invalid byte count")
        for field in ("source_family", "license", "generated_code_role", "provenance"):
            text(entry[field], field)
        url = immutable_url(text(entry["immutable_url"], "immutable_url"), revision, identifier)
        cache = cache_name(text(entry["local_cache"], "local_cache"), identifier, digest)
        return cls(identifier, url, revision, digest, entry["bytes"], cache)


def immutable_url(value: str, revision: str, identifier: str) -> str:
    try:
        parsed = urlsplit(value)
        port = parsed.port
    except ValueError as error:
        reject(f"input {identifier} has an invalid URL: {error}")
    if parsed.scheme != "https" or not parsed.hostname or parsed.username or parsed.password:
        reject(f"input {identifier} must use an immutable HTTPS URL")
    if port not in (None, 443) or parsed.query or parsed.fragment or "%" in parsed.path:
        reject(f"input {identifier} URL has unsafe mutable components")
    path_parts = parsed.path.split("/")
    parts = tuple(path_parts[1:]) if path_parts and not path_parts[0] else ()
    if not parts or any(not part or part in (".", "..") for part in parts):
        reject(f"input {identifier} URL has an unsafe path")
    if parsed.hostname == "raw.githubusercontent.com":
        reference = parts[2] if len(parts) >= 3 else ""
        valid = len(parts) >= 4 and reference == revision
    elif parsed.hostname == "gitlab.freedesktop.org":
        try:
            marker = parts.index("-")
        except ValueError:
            marker = -1
        reference = parts[marker + 2] if marker >= 0 and len(parts) > marker + 2 else ""
        valid = marker >= 2 and tuple(parts[marker:marker + 3]) == ("-", "raw", revision) and marker + 3 < len(parts)
    else:
        reference = ""
        valid = False
    if reference.lower() in MUTABLE_REFS:
        reject(f"input {identifier} URL has a mutable branch reference")
    if not valid:
        reject(f"input {identifier} URL does not use a pinned raw source path")
    return value


def cache_name(value: str, identifier: str, digest: str) -> PurePosixPath:
    path = PurePosixPath(value)
    expected = PurePosixPath("webboxvm-graphics") / "f02" / identifier / f"{digest}.source"
    if path.is_absolute() or str(path) != value or path != expected:
        reject(f"input {identifier} has an unsafe local_cache path")
    return path


def load_manifest(path: Path) -> tuple[SourceInput, ...]:
    try:
        inventory = load_inventory(path)
    except InventoryLayoutError as error:
        reject(f"manifest cannot be read: {error}")
    entries = inventory.inputs
    inputs = tuple(SourceInput.from_manifest(entry) for entry in entries)
    if len({item.identifier for item in inputs}) != len(inputs):
        reject("manifest has duplicate input ids")
    if len({item.local_cache for item in inputs}) != len(inputs):
        reject("manifest has colliding cache paths")
    families = [entry["source_family"] for entry in entries]
    if set(families) != REQUIRED_FAMILIES or len(set(families)) != len(families):
        reject("manifest has incomplete source-family coverage")
    return inputs


@dataclass(frozen=True)
class ExternalCache:
    root: Path

    @classmethod
    def from_path(cls, value: Path, repository: Path) -> "ExternalCache":
        if not isinstance(value, Path) or not isinstance(repository, Path) or not value.is_absolute():
            reject("cache root must be an explicit absolute external path")
        root = value.expanduser().resolve(strict=False)
        repository = repository.resolve(strict=False)
        if root == repository or repository in root.parents:
            reject("cache root must stay outside the repository")
        return cls(root)

    def target(self, source: SourceInput) -> Path:
        target = (self.root / source.local_cache).resolve(strict=False)
        try:
            target.relative_to(self.root)
        except ValueError:
            reject(f"input {source.identifier} escapes the external cache root")
        return target


def repository_root(anchor: Path) -> Path:
    for candidate in (anchor.resolve().parent, *anchor.resolve().parents):
        if (candidate / ".git").exists():
            return candidate
    reject("repository root cannot be located")
