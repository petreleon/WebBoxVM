"""Strict profile-independent schema for named local checks."""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Sequence

NAME = re.compile(r"^[a-z0-9][a-z0-9-]*$")
ENVIRONMENT = re.compile(r"^[A-Z][A-Z0-9_]*$")
KINDS = frozenset(("executable", "asset", "browser", "hardware", "permission"))
ACCESS = frozenset(("read", "write", "execute"))


class CatalogError(ValueError):
    """Raised when a local-check catalog is ambiguous or malformed."""


@dataclass(frozen=True)
class Prerequisite:
    kind: str
    value: str
    access: str | None = None


@dataclass(frozen=True)
class Check:
    name: str
    command: tuple[str, ...]
    expected_count: int
    artifacts: tuple[str, ...]
    tools: tuple[tuple[str, ...], ...]
    prerequisites: tuple[Prerequisite, ...]


def reject(message: str) -> None:
    raise CatalogError(message)


def text(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value.strip() or "\x00" in value:
        reject(f"{label} must be a nonempty string")
    return value


def local_path(value: Any, label: str) -> str:
    value = text(value, label)
    path = Path(value)
    if path.is_absolute() or path == Path(".") or ".." in path.parts:
        reject(f"{label} must be a relative local path")
    return value


def argv(value: Any, label: str) -> tuple[str, ...]:
    if not isinstance(value, list) or not value:
        reject(f"{label} must be a nonempty argument array")
    return tuple(text(part, f"{label} argument") for part in value)


def prerequisite(value: Any) -> Prerequisite:
    if not isinstance(value, dict):
        reject("prerequisite must be an object")
    kind = value.get("kind")
    if kind not in KINDS:
        reject("prerequisite kind is unsupported")
    fields = {"kind", "value"} | ({"access"} if kind == "permission" else set())
    if set(value) != fields:
        reject(f"{kind} prerequisite fields do not match")
    raw = value["value"]
    if kind in ("asset", "permission"):
        raw = local_path(raw, f"{kind} prerequisite value")
    else:
        raw = text(raw, f"{kind} prerequisite value")
    if kind == "hardware" and not ENVIRONMENT.fullmatch(raw):
        reject("hardware prerequisite value must name an environment variable")
    access = value.get("access")
    if kind == "permission" and access not in ACCESS:
        reject("permission prerequisite access is unsupported")
    return Prerequisite(kind, raw, access)


def string_list(value: Any, label: str) -> tuple[str, ...]:
    if not isinstance(value, list):
        reject(f"{label} must be an array")
    return tuple(local_path(item, label) for item in value)


def tool_list(value: Any) -> tuple[tuple[str, ...], ...]:
    if not isinstance(value, list) or not value:
        reject("tools must be a nonempty argument-array list")
    return tuple(argv(item, "tool") for item in value)


def parse_check(value: Any) -> Check:
    fields = {"name", "command", "expected_count", "artifacts", "tools", "prerequisites"}
    if not isinstance(value, dict) or set(value) != fields:
        reject("check fields do not match the schema")
    name = text(value["name"], "check name")
    if not NAME.fullmatch(name):
        reject("check name is invalid")
    expected = value["expected_count"]
    if type(expected) is not int or expected <= 0:
        reject("expected_count must be a positive integer")
    prerequisites = value["prerequisites"]
    if not isinstance(prerequisites, list):
        reject("prerequisites must be an array")
    return Check(
        name, argv(value["command"], "command"), expected,
        string_list(value["artifacts"], "artifact"), tool_list(value["tools"]),
        tuple(prerequisite(item) for item in prerequisites),
    )


def load_catalog(path: Path) -> tuple[Check, ...]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        reject(f"cannot load catalog: {error}")
    if not isinstance(value, dict) or set(value) != {"schema", "checks"}:
        reject("catalog fields do not match the schema")
    if value["schema"] != 1 or not isinstance(value["checks"], list) or not value["checks"]:
        reject("catalog must use schema 1 with at least one check")
    checks = tuple(parse_check(item) for item in value["checks"])
    if len({check.name for check in checks}) != len(checks):
        reject("catalog check names must be unique")
    return checks


def select_checks(checks: Sequence[Check], names: Sequence[str]) -> tuple[Check, ...]:
    if not names or any(not isinstance(name, str) or not name.strip() for name in names):
        reject("selection must name at least one check")
    if len(set(names)) != len(names):
        reject("selection must not repeat a check")
    indexed = {check.name: check for check in checks}
    unknown = [name for name in names if name not in indexed]
    if unknown:
        reject("unknown check selection: " + ", ".join(unknown))
    return tuple(indexed[name] for name in names)
