"""Versioned graphics-chunk identities and source-selection rules."""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path


DIGEST = re.compile(r"^[0-9a-f]{64}$")


class ChunkSchemaError(ValueError):
    """Raised when a chunk schema cannot safely select its authoritative source."""


@dataclass(frozen=True)
class ChunkSchema:
    version: int
    identity_field: str
    header_field: str
    generator: str
    source_flag: str
    source_attribute: str
    source_label: str


V1 = ChunkSchema(
    1,
    "manifest_revision",
    "manifest-revision",
    "webboxvm.graphics-chunker.v1",
    "--source-manifest",
    "source_manifest",
    "source manifest revision",
)
V2 = ChunkSchema(
    2,
    "inventory_sha256",
    "inventory-sha256",
    "webboxvm.graphics-chunker.v2",
    "--inventory-lock",
    "inventory_lock",
    "inventory lock identity",
)
SCHEMAS = {V1.version: V1, V2.version: V2}


def profile_for(version: object) -> ChunkSchema:
    """Return a supported schema profile, rejecting bool and unknown versions."""
    if type(version) is not int or version not in SCHEMAS:
        raise ChunkSchemaError("chunk specification has an unsupported schema")
    return SCHEMAS[version]


def spec_fields(document: object) -> tuple[ChunkSchema, str, object]:
    """Read and validate the versioned identity while leaving record grammar to the chunker."""
    if not isinstance(document, dict):
        raise ChunkSchemaError("chunk specification has an unsupported schema")
    profile = profile_for(document.get("schema"))
    if set(document) != {"schema", profile.identity_field, "records"}:
        raise ChunkSchemaError("chunk specification fields do not match its schema")
    revision = document[profile.identity_field]
    if not isinstance(revision, str) or not DIGEST.fullmatch(revision):
        raise ChunkSchemaError("chunk specification has an invalid source identity")
    return profile, revision, document["records"]


def add_source_arguments(parser: object) -> None:
    """Add mutually exclusive v1/v2 source options to an argparse parser."""
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--source-manifest", type=Path)
    group.add_argument("--inventory-lock", type=Path)


def source_for(arguments: object, profile: ChunkSchema) -> Path:
    """Select the raw source byte stream required by this profile."""
    path = getattr(arguments, profile.source_attribute)
    if path is None:
        raise ChunkSchemaError(
            f"chunk specification schema v{profile.version} requires {profile.source_flag}"
        )
    return path
