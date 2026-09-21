# F03.2.2.5.4 — Extract non-object command slices

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.2.2.5.4
Depends: F03.2.1, F03.2.2.1, F03.2.2.5.1, F03.2.2.5.2
Evidence: pending

## Outcome

Remaining non-object OpenGL 4.6 core command declarations are separated by execution purpose so broad
state-setting, draw, and query material cannot be silently omitted or confused with returned state values.

## Starting points

- [command-domain classifier](../01-command-domain-classification/README.md)
- [template declaration grammar](../02-template-declaration-grammar/README.md)
- [raw limit/format route](../../../03-limit-format-shader-inventory/README.md)

## Checklist

- [x] [F03.2.2.5.4.1 — Global execution and synchronization commands](01-global-execution-sync/README.md)
- [ ] [F03.2.2.5.4.2 — Draw and compute submission commands](02-draw-compute-submission/README.md)
- [ ] [F03.2.2.5.4.3 — Rasterization and framebuffer state-setting commands](03-raster-framebuffer-state/README.md)
- [ ] [F03.2.2.5.4.4 — Pixel read and copy commands](04-pixel-read-copy/README.md)
- [ ] [F03.2.2.5.4.5 — Debug, special, and generic context-query commands](05-debug-special-context-queries/README.md)

## Verification

Command declarations do not establish state semantics, numeric limits, formats, Matrix ownership, CTS
execution, API support, guest/browser behavior, conformance, certification, or performance.
