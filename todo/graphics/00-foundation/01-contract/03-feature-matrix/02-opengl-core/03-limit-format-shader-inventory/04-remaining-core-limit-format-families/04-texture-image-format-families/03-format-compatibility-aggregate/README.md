# F03.2.3.4.4.3 — Aggregate format compatibility coverage

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F03.2.3.4.4.3
Depends: F03.2.3.4.1, F03.2.3.4.4.1, F03.2.3.4.4.2
Evidence: pending

## Outcome

One raw-source aggregate reconciles every texture/image-format candidate with its fact, route, unavailable
decision, or blocker before F03.2.3.4.6 consumes this domain.

## Starting points

- [storage/image review](../01-storage-image-format-anchors/README.md)
- [sampler/view review](../02-sampler-view-format-constraints/README.md)
- [anchor classification catalog](../../01-anchor-classification/README.md)

## Checklist

- [ ] Bind all child artifact identities and every cataloged texture/image candidate.
- [ ] Require one fact, route, unavailable decision, or blocker for each anchor exactly once.
- [ ] Reject gaps, duplicate/reordered coverage, false completeness, and owner/test/Matrix fields.
- [ ] Retain zero CTS executions and all support/conformance/guest/browser/certification/performance claims false.
- [ ] Attach a no-claim aggregate receipt for F03.2.3.4.6.

## Verification

Coverage reconciliation is not evidence of format compatibility or an implemented texture/image path.
