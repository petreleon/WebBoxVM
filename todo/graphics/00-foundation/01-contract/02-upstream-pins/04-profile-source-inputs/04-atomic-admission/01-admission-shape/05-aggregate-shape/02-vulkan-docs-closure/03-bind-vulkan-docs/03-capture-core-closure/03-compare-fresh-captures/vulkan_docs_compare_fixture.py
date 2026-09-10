"""Hermetic two-capture scopes for independent-comparison tests."""

from __future__ import annotations

from dataclasses import replace
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCOPE = HERE.parent / "02-bind-core-input-scope"
if str(SCOPE) not in sys.path:
    sys.path.insert(0, str(SCOPE))

from vulkan_docs_scope_bind import _bind_fixture  # noqa: E402
from vulkan_docs_scope_fixture import valid  # noqa: E402


def scope(identifier: str, artifact: str, run: str, io_trace: str):
    value, capture = valid()
    selected = replace(capture, identifier=identifier, artifact=artifact, run_digest=run * 64, io_trace_digest=io_trace * 64)
    return _bind_fixture(value, selected)


def pair():
    return (scope("observer-a", "runs/observer-a", "a", "c"), scope("observer-b", "runs/observer-b", "b", "d"))
