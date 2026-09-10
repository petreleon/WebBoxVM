"""Verify distinct recorded locations without reading source or output payloads."""

from __future__ import annotations

import sys
from pathlib import Path

from vulkan_docs_compare_model import reject

HERE = Path(__file__).resolve().parent
SCOPE = HERE.parent / "02-bind-core-input-scope"
if str(SCOPE) not in sys.path:
    sys.path.insert(0, str(SCOPE))

from vulkan_docs_scope_bind import artifact_file  # noqa: E402
from vulkan_docs_scope_parse import require_safe_child  # noqa: E402


def distinct(left: Path, right: Path, label: str) -> None:
    try:
        if left.samefile(right):
            reject(f"comparison reuses a {label} location")
    except OSError as error:
        reject(f"comparison cannot inspect {label} locations: {error}")


def directory(root: Path, relative: str, label: str) -> Path:
    try:
        supplied = root.absolute()
        if root.is_symlink() or root.resolve(strict=True) != supplied:
            reject("comparison artifact root is not a canonical non-symlink path")
        base = supplied
        if not base.is_dir():
            reject("comparison artifact root is not a directory")
        parts = relative.split("/")
        if not all(part and part not in (".", "..") for part in parts):
            reject(f"comparison {label} location is unsafe")
        value = root
        for part in parts:
            value /= part
            if value.is_symlink():
                reject(f"comparison {label} location contains a symlink")
        resolved = value.resolve(strict=True)
        resolved.relative_to(base)
        if not resolved.is_dir():
            reject(f"comparison {label} location is not a directory")
        return resolved
    except (OSError, ValueError) as error:
        reject(f"comparison {label} location is unsafe: {error}")


def artifact(root: Path, run: str, name: str) -> Path:
    try:
        return artifact_file(root, run, name)
    except Exception as error:
        reject(f"comparison {name} location is unsafe: {error}")


def required(root: Path, name: str, label: str) -> Path:
    try:
        return require_safe_child(root, name, label)
    except Exception as error:
        reject(f"comparison {label} is unsafe: {error}")


def checkout(root: Path, identifier: str) -> Path:
    value = directory(root, f"sources/{identifier}", "source")
    required(value, "vkspec.adoc", "source root")
    return value


def output(root: Path, scope) -> Path:
    artifact(root, scope.capture.artifact, "run.json")
    value = directory(root, f"{scope.capture.artifact}/generated", "output")
    required(value, "out/html/vkspec.html", "primary output")
    return value


def independent_locations(root: Path, scopes) -> None:
    scopes = tuple(scopes)
    sources = tuple(checkout(root, scope.capture.identifier) for scope in scopes)
    distinct(sources[0], sources[1], "source checkout")
    for name in ("run.json", "normalized-inputs.json", "io-events.jsonl", "resolved-includes.jsonl"):
        paths = tuple(artifact(root, scope.capture.artifact, name) for scope in scopes)
        distinct(paths[0], paths[1], name)
    outputs = tuple(output(root, scope) for scope in scopes)
    distinct(outputs[0], outputs[1], "generated output")
    for source in sources:
        for generated in outputs:
            distinct(source, generated, "source/output")
