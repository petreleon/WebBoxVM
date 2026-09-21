# F03.2.3.4.6 — Aggregate gaps and guard the raw handoff

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.2.3.4.6
Depends: F03.2.3.1, F03.2.3.2, F03.2.3.4.1, F03.2.3.4.2, F03.2.3.4.3, F03.2.3.4.4, F03.2.3.4.5
Evidence: pending

## Outcome

One aggregate guard reconciles the prior table slice, all classified residual anchors, and unavailable-source
decisions before F03.2.3.3 can consume raw facts; incomplete or routed anchors remain visibly blocked.

## Starting points

- [anchor classification catalog](../01-anchor-classification/README.md)
- [unadmitted source ledger](../../02-unadmitted-shader-extension-ledger/README.md)
- [raw handoff guard](../../03-raw-handoff-and-import-guard/README.md)

## Checklist

- [ ] Bind every child artifact and the F03.2.3.1/2 baseline identities exactly.
- [ ] Require each catalog anchor to be covered by a raw fact, explicit unavailable decision, route, or blocker.
- [ ] Reject unclassified, duplicated, reordered, or falsely complete coverage and raw facts with owner/test fields.
- [ ] Conclude only raw-source coverage; leave Matrix, CTS, support, guest/browser, certification, and performance false.
- [ ] Attach a no-claim receipt that F03.2.3.3 can consume.

## Verification

This aggregate is a provenance and gap gate. It is not evidence that an OpenGL limit or format is implemented
or behaves correctly in a guest or browser.
