# F03.3.2.2.3.6.3 — Vertex-array object lifecycle commands

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F03.3.2.2.3.6.3
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: [receipt](evidence.md)

## Outcome

A bounded raw inventory covers only `GenVertexArrays`, `DeleteVertexArrays`,
`BindVertexArray`, and `IsVertexArray` on pp.294–295. Object effects and draw behavior stay out.

## Starting points

- [vertex/transform-feedback parent](../README.md)
- [declaration grammar](../../../02-template-declaration-grammar/README.md)
- [verified normative-PDF cache](../../../../01-normative-pdf-cache/README.md)

## Checklist

- [x] Bind exact authority, cache, vertex-array family, route, and grammar identities.
- [x] Extract only the four assigned §10.4 lifecycle literals and preserve the `boolean` return form.
- [x] Preserve pp.294–295, section headings, source order, and the p.295 pre-§10.5 fence.
- [x] Route object existence, binding, array state, drawing, and ESSL semantics out.
- [x] Reject configuration, transform-feedback, draw, extensions, and registry data.
- [x] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declarations do not prove vertex-array object behavior, draw behavior,
guest/browser rendering, support, conformance, certification, or performance.
