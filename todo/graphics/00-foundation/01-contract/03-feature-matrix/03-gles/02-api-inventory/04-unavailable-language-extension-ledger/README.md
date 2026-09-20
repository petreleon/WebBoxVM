# F03.3.2.4 — Record unavailable language and extension decisions

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.3.2.4
Depends: F03.3.1
Evidence: pending

## Outcome

A self-consistent decision ledger preserves the F03.3.1 blockers for GLES shader, precision, and extension
semantics. It prevents an unadmitted ESSL source or a registry from silently becoming a core API input.

## Starting points

- [source-authority boundary](../../01-source-authority/README.md)
- [sealed role-aware source contract](../../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)
- [shader and CTS obligations](../../03-shader-and-cts-obligations/README.md)

## Checklist

- [ ] Load the exact F03.3.1 decision and retain `unadmitted-distinct-source` for shader, precision, and
  extension classes.
- [ ] Record each unavailable class, its absent locator syntax, source-role absence, required separate
  admission, and exact blocker without making a substitute derivation.
- [ ] Reject ESSL aliases, `gl.xml`, desktop GLSL, lower-version sources, stale decisions, mixed profiles,
  reordered entries, or an auxiliary source presented as an admitted class.
- [ ] Keep unavailable semantics visible to F03.3.3 instead of dropping them or treating them as excluded
  GLES 3.2 requirements.
- [ ] Refuse a Matrix v2 row or supported state derived from an unavailable class.
- [ ] Add focused ledger and hostile-input checks, then attach a no-claim receipt.

## Verification

The ledger establishes only a source-admission boundary. It neither admits ESSL nor makes shader,
extension, conformance, guest, browser, certification, or performance claims.
