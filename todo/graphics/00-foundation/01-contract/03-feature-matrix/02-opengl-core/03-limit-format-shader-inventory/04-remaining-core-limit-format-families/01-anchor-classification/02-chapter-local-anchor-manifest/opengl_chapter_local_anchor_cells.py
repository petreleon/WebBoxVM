#!/usr/bin/env python3
"""Closed, source-order anchor cells for F03.2.3.4.1.2."""

from __future__ import annotations

# (identifier, kind, table-or-none, physical-caption-or-heading-page, page-header section)
FRAGMENTS = (
    ("buffer", (
        ("table-6-5", "table", "6.5", 107, "6.8"),
        ("table-8-2", "table", "8.2", 215, "8.4"),
        ("table-8-3", "table", "8.3", 216, "8.4"),
        ("table-8-4", "table", "8.4", 216, "8.4"),
        ("table-8-5", "table", "8.5", 219, "8.4"),
    )),
    ("pixel", (
        ("table-8-6", "table", "8.6", 220, "8.4"),
        ("table-8-7", "table", "8.7", 221, "8.4"),
        ("table-8-8", "table", "8.8", 222, "8.4"),
        ("table-8-9", "table", "8.9", 223, "8.4"),
        ("table-8-10", "table", "8.10", 224, "8.4"),
    )),
    ("texture-image", (
        ("table-8-11", "table", "8.11", 227, "8.5"),
        ("table-8-12", "table", "8.12", 232, "8.5"),
        ("table-8-13", "table", "8.13", 233, "8.5"),
        ("table-8-14", "table", "8.14", 234, "8.5"),
        ("table-8-15", "table", "8.15", 243, "8.6"),
    )),
    ("texture-state", (
        ("table-8-16", "table", "8.16", 260, "8.10"),
        ("table-8-17", "table", "8.17", 262, "8.10"),
        ("table-8-18", "table", "8.18", 271, "8.11"),
        ("table-8-19", "table", "8.19", 275, "8.13"),
        ("table-8-20", "table", "8.20", 282, "8.14"),
    )),
    ("texture-views", (
        ("table-8-21", "table", "8.21", 294, "8.18"),
        ("table-8-22", "table", "8.22", 295, "8.18"),
        ("table-8-23", "table", "8.23", 310, "8.24"),
        ("table-8-24", "table", "8.24", 311, "8.25"),
        ("table-8-25", "table", "8.25", 315, "8.26"),
    )),
    ("texture-framebuffer", (
        ("table-8-26", "table", "8.26", 318, "8.26"),
        ("table-8-27", "table", "8.27", 320, "8.26"),
        ("table-9-1", "table", "9.1", 331, "9.2"),
        ("table-9-2", "table", "9.2", 341, "9.2"),
        ("table-9-3", "table", "9.3", 358, "9.8"),
    )),
    ("vertex-pixel-query", (
        ("table-10-3", "table", "10.3", 376, "10.3"),
        ("table-10-4", "table", "10.4", 384, "10.3"),
        ("table-10-5", "table", "10.5", 384, "10.3"),
        ("table-10-6", "table", "10.6", 385, "10.3"),
        ("table-18-2", "table", "18.2", 559, "18.3"),
        ("table-18-4", "table", "18.4", 566, "18.4"),
        ("section-22-3", "section", None, 591, "22.3"),
        ("table-22-2", "table", "22.2", 592, "22.3"),
    )),
)
