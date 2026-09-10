"""One reversible raw/derived member-ID convention for Docs proof children."""

from __future__ import annotations

from pathlib import PurePosixPath


def selector(value: object) -> str:
    if not isinstance(value, str) or not value or not value.isascii() or "\\" in value:
        raise ValueError("member selector is invalid")
    parsed = PurePosixPath(value)
    if parsed.is_absolute() or str(parsed) != value or any(part in (".", "..") for part in parsed.parts):
        raise ValueError("member selector escapes its root")
    return value


def member_id(kind: str, value: object) -> str:
    selected = selector(value)
    if kind == "raw-source-input" and selected == "vkspec.adoc": return "vulkan-14-spec"
    prefix = {"raw-source-input": "raw", "derived-source-input": "derived"}.get(kind)
    if prefix is None: raise ValueError("member kind is invalid")
    return f"{prefix}-{selected.encode('ascii').hex()}"


def raw_member_id(value: object) -> str:
    return member_id("raw-source-input", value)
