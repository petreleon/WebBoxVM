#!/usr/bin/env python3
"""Finite chapter-local routes for F03.2.3.4.1.3."""

from __future__ import annotations

ROUTES = ("covered", "eligible-unreviewed", "route-to-state", "shader-unadmitted",
          "extension-unadmitted", "out-of-domain")
DESTINATIONS = {
    "covered": frozenset(), "out-of-domain": frozenset(),
    "route-to-state": frozenset(("F03.2.2.3.2",)),
    "shader-unadmitted": frozenset(("F03.2.3.2",)),
    "extension-unadmitted": frozenset(("F03.2.3.2",)),
    "eligible-unreviewed": frozenset(("F03.2.3.4.2", "F03.2.3.4.3", "F03.2.3.4.4.1",
                                       "F03.2.3.4.4.2", "F03.2.3.4.5.1", "F03.2.3.4.5.2",
                                       "F03.2.3.4.5.3", "F03.2.3.4.5.4")),
}


def eligible(reason: str, destination: str) -> tuple[str, str, str]:
    return ("eligible-unreviewed", reason, destination)


LOCAL_POLICY = {
    "opengl46-local-table-6-5": eligible("indexed-buffer-anchor-needs-nonshader-core-limit-review", "F03.2.3.4.2"),
    "opengl46-local-table-8-2": eligible("table-8-2-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-3": eligible("table-8-3-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-4": eligible("table-8-4-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-5": eligible("table-8-5-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-6": eligible("table-8-6-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-7": eligible("table-8-7-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-8": eligible("table-8-8-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-9": eligible("table-8-9-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-10": eligible("table-8-10-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-11": eligible("table-8-11-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-12": eligible("table-8-12-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-13": eligible("table-8-13-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-14": eligible("table-8-14-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-15": eligible("table-8-15-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-16": eligible("table-8-16-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-17": eligible("table-8-17-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-18": eligible("table-8-18-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-19": eligible("table-8-19-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-20": eligible("table-8-20-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-21": eligible("table-8-21-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-22": eligible("table-8-22-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-23": eligible("table-8-23-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-24": eligible("table-8-24-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-25": eligible("table-8-25-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-26": eligible("table-8-26-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-8-27": eligible("table-8-27-needs-storage-image-review", "F03.2.3.4.4.1"),
    "opengl46-local-table-9-1": eligible("table-9-1-needs-framebuffer-review", "F03.2.3.4.5.1"),
    "opengl46-local-table-9-2": eligible("table-9-2-needs-framebuffer-review", "F03.2.3.4.5.1"),
    "opengl46-local-table-9-3": eligible("table-9-3-needs-framebuffer-review", "F03.2.3.4.5.1"),
    "opengl46-local-table-10-3": eligible("table-10-3-needs-vertex-transform-review", "F03.2.3.4.5.3"),
    "opengl46-local-table-10-4": eligible("table-10-4-needs-vertex-transform-review", "F03.2.3.4.5.3"),
    "opengl46-local-table-10-5": eligible("table-10-5-needs-vertex-transform-review", "F03.2.3.4.5.3"),
    "opengl46-local-table-10-6": eligible("table-10-6-needs-vertex-transform-review", "F03.2.3.4.5.3"),
    "opengl46-local-table-18-2": eligible("table-18-2-needs-pixel-transfer-review", "F03.2.3.4.5.2"),
    "opengl46-local-table-18-4": eligible("table-18-4-needs-pixel-transfer-review", "F03.2.3.4.5.2"),
    "opengl46-local-section-22-3": eligible("section-22-3-needs-query-constraint-review", "F03.2.3.4.5.4"),
    "opengl46-local-table-22-2": eligible("table-22-2-needs-query-constraint-review", "F03.2.3.4.5.4"),
}
