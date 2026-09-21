# F03.3.2.2.3 — Extract object and resource command slices

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.3.2.2.3
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: pending

## Outcome

Separate bounded inventories extract formal object/resource command declarations. Each slice preserves source
anchors while routing state, formats, limits, and unavailable ESSL semantics out of the command facts.

## Starting points

- [domain classification](../01-command-domain-classification/README.md)
- [declaration grammar](../02-template-declaration-grammar/README.md)
- [verified normative-PDF cache](../../01-normative-pdf-cache/README.md)

## Checklist

- [x] [F03.3.2.2.3.1 — Extract generic, sync, and query commands](01-generic-sync-query/README.md)
- [ ] [F03.3.2.2.3.2 — Extract buffer commands](02-buffer-commands/README.md)
- [ ] [F03.3.2.2.3.3 — Extract program and pipeline commands](03-program-pipeline-commands/README.md)
- [ ] [F03.3.2.2.3.4 — Extract texture and sampler commands](04-texture-sampler-commands/README.md)
- [ ] [F03.3.2.2.3.5 — Extract framebuffer and renderbuffer commands](05-framebuffer-renderbuffer-commands/README.md)
- [ ] [F03.3.2.2.3.6 — Extract vertex and transform-feedback commands](06-vertex-transform-feedback-commands/README.md)

## Verification

Every child remains raw-only and `matrix-incomplete`; its declarations do not prove state behavior, format
support, guest/browser behavior, conformance, certification, or performance.
