# F03.2.2.5.3 — Extract object and resource command slices

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.2.2.5.3
Depends: F03.2.1, F03.2.2.1, F03.2.2.5.1, F03.2.2.5.2
Evidence: pending

## Outcome

Remaining object and resource command declarations are divided into six independently source-bounded
slices. State transitions and shader-language semantics remain outside these declaration inventories.

## Starting points

- [command-domain classifier](../01-command-domain-classification/README.md)
- [template declaration grammar](../02-template-declaration-grammar/README.md)
- [raw state route](../../03-state-and-lifecycle-raw-inventory/README.md)

## Checklist

- [x] [F03.2.2.5.3.1 — Generic object and synchronization command declarations](01-generic-object-sync/README.md)
- [x] [F03.2.2.5.3.2 — Buffer command declarations](02-buffer-commands/README.md)
- [ ] [F03.2.2.5.3.3 — Program and pipeline command declarations](03-program-pipeline-commands/README.md)
- [ ] [F03.2.2.5.3.4 — Texture and sampler command declarations](04-texture-sampler-commands/README.md)
- [ ] [F03.2.2.5.3.5 — Framebuffer and renderbuffer command declarations](05-framebuffer-renderbuffer-commands/README.md)
- [ ] [F03.2.2.5.3.6 — Vertex-array and transform-feedback command declarations](06-vertex-transform-feedback-commands/README.md)

## Verification

Every child emits only raw, source-located declarations and must leave state/lifecycle completion, Matrix
rows, owners, CTS execution, guest/browser behavior, certification, and performance unresolved.
