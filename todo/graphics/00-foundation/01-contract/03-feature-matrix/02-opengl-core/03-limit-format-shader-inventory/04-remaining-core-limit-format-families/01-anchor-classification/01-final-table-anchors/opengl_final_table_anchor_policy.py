#!/usr/bin/env python3
"""Finite cell policy for F03.2.3.4.1.1; no table-wide fallbacks."""

from __future__ import annotations

import importlib.util
from pathlib import Path

HERE = Path(__file__).resolve().parent
PARTS = ("opengl_final_table_anchor_policy_23_56_23_66.py",
         "opengl_final_table_anchor_policy_23_67_23_74.py")
OUTCOMES = {
    "k1": ("route-to-state", "version-context-or-identity-query-state", "F03.2.2.3.2"),
    "k2": ("extension-unadmitted", "extension-name-or-count-needs-unadmitted-extension-source", "F03.2.3.2"),
    "k3": ("shader-unadmitted", "shader-language-version-needs-unadmitted-shader-source", "F03.2.3.2"),
    "k4": ("eligible-unreviewed", "stage-or-program-limit-needs-per-row-source-policy-review", "F03.2.3.4.3"),
    "k5": ("route-to-state", "debug-returned-state-is-not-a-raw-limit-or-format-fact", "F03.2.2.3.2"),
    "k6": ("eligible-unreviewed", "numeric-debug-limit-needs-nonshader-core-limit-review", "F03.2.3.4.2"),
    "k7": ("eligible-unreviewed", "framebuffer-or-multisample-limit-needs-framebuffer-review", "F03.2.3.4.5.1"),
    "k8": ("eligible-unreviewed", "format-scoped-sample-limit-needs-storage-image-review", "F03.2.3.4.4.1"),
    "k9": ("eligible-unreviewed", "query-counter-limit-needs-chapter-local-query-review", "F03.2.3.4.5.4"),
    "k10": ("eligible-unreviewed", "nonshader-sync-limit-needs-core-limit-review", "F03.2.3.4.2"),
    "k11": ("eligible-unreviewed", "fragment-stage-limit-needs-per-row-source-policy-review", "F03.2.3.4.3"),
    "k12": ("eligible-unreviewed", "draw-buffer-limit-needs-nonshader-core-limit-review", "F03.2.3.4.2"),
    "k13": ("eligible-unreviewed", "framebuffer-attachment-limit-needs-framebuffer-review", "F03.2.3.4.5.1"),
    "k14": ("eligible-unreviewed", "transform-feedback-limit-needs-vertex-transform-review", "F03.2.3.4.5.3"),
    "k15": ("route-to-state", "framebuffer-dependent-returned-state-is-not-a-raw-limit-fact", "F03.2.2.3.2"),
    "k16": ("eligible-unreviewed", "pixel-read-format-property-needs-pixel-transfer-review", "F03.2.3.4.5.2"),
    "k17": ("route-to-state", "error-binding-or-query-returned-state-needs-state-lifecycle-review", "F03.2.2.3.2"),
}


def load(path: Path, name: str):
    if path.is_symlink() or not path.is_file():
        raise ValueError("finite policy fragment is unavailable")
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise ValueError("cannot load finite policy fragment")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
        raise ValueError("finite policy fragment resolved from an unexpected path")
    return getattr(module, "ROWS", None)


POLICY = {}
for part in PARTS:
    rows = load(HERE / part, f"f032341_policy_{part[:8]}")
    if not isinstance(rows, tuple):
        raise ValueError("finite policy fragment has an invalid shape")
    for table, cell, outcome in rows:
        key = (table, cell)
        if (not all(isinstance(value, str) and value for value in key) or outcome not in OUTCOMES
                or key in POLICY):
            raise ValueError("finite policy mapping is invalid or duplicated")
        POLICY[key] = OUTCOMES[outcome]
if len(POLICY) != 158:
    raise ValueError("finite policy mapping has an unexpected key count")


def policy(table: str, cell: str) -> tuple[str, str, str]:
    try:
        return POLICY[(table, cell)]
    except (KeyError, TypeError) as error:
        raise ValueError("catalog has no explicit policy for a source cell") from error
