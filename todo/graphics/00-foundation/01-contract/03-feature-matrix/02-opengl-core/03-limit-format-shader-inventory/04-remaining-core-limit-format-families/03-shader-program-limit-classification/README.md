# F03.2.3.4.3 — Classify shader and program limit rows

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.2.3.4.3
Depends: F03.2.1, F03.2.2.1, F03.2.3.2, F03.2.3.4.1
Evidence: pending

## Outcome

Mixed shader/program and stage-limit rows receive an explicit per-row source-policy decision; a `Shader`
heading neither admits nor excludes a core API limit automatically.

## Starting points

- [anchor classification catalog](../01-anchor-classification/README.md)
- [unadmitted shader ledger](../../02-unadmitted-shader-extension-ledger/README.md)
- [normative PDF cache](../../../02-command-object-state-inventory/01-normative-pdf-cache/README.md)

## Checklist

- [ ] Bind the exact unavailable shader/extension ledger and the closed anchor catalog.
- [ ] Decide each table 23.56–23.66 candidate at row/column granularity, with source-policy reason.
- [ ] Preserve eligible API limits as eligible-unreviewed until a separate raw fact extraction verifies them.
- [ ] Keep compiler, shader binary, shader-language, and extension semantics unavailable where the ledger requires.
- [ ] Reject table-title inference, lower profiles, registry/GLSL substitutes, and Matrix/claim promotion.
- [ ] Self-hash the decision ledger and attach a no-claim receipt.

## Verification

This task resolves source-policy classification only. It does not execute shaders or prove shader support,
limit behavior, conformance, guest/browser behavior, certification, or performance.
