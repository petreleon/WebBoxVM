#!/usr/bin/env python3
"""Fixed-path JSON and module helpers for F03.2.3.4.1.3."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path

MAX_SERIALIZED = 1024 * 1024


class AggregateError(ValueError):
    """The closed anchor aggregate lost its bounded source-only boundary."""


def reject(message: str) -> None:
    raise AggregateError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def sha256(value: object) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def pairs(items):
    value = {}
    for key, item in items:
        if key in value:
            reject("aggregate JSON has duplicate fields")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    if path.is_symlink() or not path.is_file():
        reject("aggregate artifact must be a regular file")
    try:
        raw = path.read_bytes()
        if len(raw) > MAX_SERIALIZED:
            reject("aggregate artifact exceeds its serialized-size cap")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"aggregate artifact cannot be read: {error}")
    if not isinstance(value, dict):
        reject("aggregate artifact is not a JSON object")
    return value


def private(path: Path, name: str):
    if path.is_symlink() or not path.is_file():
        reject("fixed private dependency must be a regular file")
    previous = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            reject("cannot load fixed private dependency")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            reject("fixed private dependency resolved from an unexpected path")
        return module
    except AggregateError:
        raise
    except Exception as error:
        reject(f"cannot load fixed private dependency: {error}")
    finally:
        if previous is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = previous


def write(path: Path, text: str) -> None:
    if path.exists() and (path.is_symlink() or not path.is_file()):
        reject("aggregate artifact output must be a regular file")
    if len(text.splitlines()) > 180:
        reject("aggregate artifact exceeds the source-file line limit")
    try:
        path.write_text(text, encoding="utf-8")
    except OSError as error:
        reject(f"aggregate artifact cannot be written: {error}")


def candidate_fragment_text(value: dict[str, object]) -> str:
    lines = ["{"]
    for key in ("schema", "kind", "profile", "fragment_id", "child"):
        lines.append(f"  {json.dumps(key)}: {json.dumps(value[key], sort_keys=True)},")
    lines.append('  "candidates": [')
    for index, item in enumerate(value["candidates"]):
        first = json.dumps({key: item[key] for key in ("candidate_id", "source_locator")}, sort_keys=True)[1:-1]
        second = json.dumps({key: item[key] for key in ("aggregate_order", "child", "child_source_order")}, sort_keys=True)[1:-1]
        third = json.dumps({key: item[key] for key in ("destination", "route", "route_reason")}, sort_keys=True)[1:-1]
        lines.extend((f"    {{{first},", f"     {second},", f"     {third}}}" + ("," if index + 1 < len(value["candidates"]) else "")))
    lines.append("  ],")
    for key in ("candidate_count", "candidates_sha256", "fragment_sha256"):
        lines.append(f"  {json.dumps(key)}: {json.dumps(value[key], sort_keys=True)}" + ("," if key != "fragment_sha256" else ""))
    return "\n".join((*lines, "}", ""))
