# F03.3.2.2.3.6.5 — Transform-feedback capture-control declarations

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F03.3.2.2.3.6.5
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: pending

## Outcome

A bounded raw inventory covers only `BeginTransformFeedback`, `EndTransformFeedback`,
`PauseTransformFeedback`, and `ResumeTransformFeedback` on p.357. Capture behavior stays out.

## Starting points

- [vertex/transform-feedback parent](../README.md)
- [declaration grammar](../../../02-template-declaration-grammar/README.md)
- [verified normative-PDF cache](../../../../01-normative-pdf-cache/README.md)

## Checklist

- [ ] Bind exact authority, cache, transform-feedback family, route, and grammar identities.
- [ ] Extract only the four assigned §12.2.2 capture-control literals.
- [ ] Preserve p.357, section heading, return forms, and source order.
- [ ] Route capture, primitive processing, buffers, state, output, and ESSL semantics out.
- [ ] Reject transform-feedback objects, vertex arrays, program varying, draw, extensions, and registry data.
- [ ] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declarations do not prove capture behavior, primitive processing, captured output,
guest/browser rendering, support, conformance, certification, or performance.
