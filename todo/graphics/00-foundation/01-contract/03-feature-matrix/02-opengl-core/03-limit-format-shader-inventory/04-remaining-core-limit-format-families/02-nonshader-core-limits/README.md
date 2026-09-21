# F03.2.3.4.2 — Extract non-shader core limits

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.2.3.4.2
Depends: F03.2.1, F03.2.2.1, F03.2.3.1, F03.2.3.4.1
Evidence: pending

## Outcome

Eligible non-shader core limits and capabilities are extracted from the closed anchor catalog as bounded,
raw source facts; mixed rows remain classified rather than assumed eligible from their table title.

## Starting points

- [anchor classification catalog](../01-anchor-classification/README.md)
- [reviewed table slice](../../01-limit-format-raw-inventory/README.md)
- [normative PDF cache](../../../02-command-object-state-inventory/01-normative-pdf-cache/README.md)

## Checklist

- [ ] Bind the exact cache, classification catalog, and existing raw artifact identities.
- [ ] Extract only cataloged eligible non-shader rows with page, section, table/row, and source-order anchors.
- [ ] Keep version/context state, extension rows, and shader/program rows routed as the catalog requires.
- [ ] Reject duplicate/reordered/partial facts, wrong profile, registry, compatibility, and inferred eligibility.
- [ ] Preserve `coverage.complete=false` until the aggregate gate; keep Matrix/CTS/claim fields zero.
- [ ] Attach a no-claim receipt with focused hostile checks.

## Verification

Raw values are not implementation limits or proof of a supported feature, conformance, guest/browser
behavior, certification, or performance.
