# F03.2.3.4 — Cover remaining core limit and format families

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.2.3.4
Depends: F03.2.1, F03.2.2.1, F03.2.3.1, F03.2.3.2
Evidence: pending

## Outcome

The reviewed OpenGL table slice does not become a proxy for all OpenGL 4.6 core limits and format
properties. This task must split remaining tables and chapter-local families into bounded reviews before
the raw handoff can complete.

## Starting points

- [reviewed limit/format raw inventory](../01-limit-format-raw-inventory/README.md)
- [normative PDF cache](../../02-command-object-state-inventory/01-normative-pdf-cache/README.md)
- [source-authority boundary](../../01-source-authority/README.md)

## Checklist

- [x] [F03.2.3.4.1 — Classify remaining anchors](01-anchor-classification/README.md)
- [ ] [F03.2.3.4.2 — Extract non-shader core limits](02-nonshader-core-limits/README.md)
- [ ] [F03.2.3.4.3 — Classify shader and program limit rows](03-shader-program-limit-classification/README.md)
- [ ] [F03.2.3.4.4 — Review texture and image format families](04-texture-image-format-families/README.md)
- [ ] [F03.2.3.4.5 — Review framebuffer, pixel, and vertex families](05-framebuffer-pixel-vertex-families/README.md)
- [ ] [F03.2.3.4.6 — Aggregate gaps and guard the raw handoff](06-aggregate-gap-handoff/README.md)

## Verification

Only F03.2.3.4.6 can close the classified raw limit/format domain, after every anchor is covered or visibly
routed. Raw facts remain blocked and `matrix-incomplete`; they create no Matrix v2 rows, owner/test fields,
CTS execution, guest/browser behavior, certification claim, or performance measurement.
