# F03.2.3.2 — Ledger unadmitted shader and extension sources

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.2.3.2
Depends: F03.2.1
Evidence: pending

## Outcome

A self-hashed policy ledger preserves the F03.2.1 decisions that shader and extension semantics are
unavailable from the admitted OpenGL PDF. It records missing authority, not shader rules, extensions, or
an implicit F02 admission.

## Starting points

- [source-authority boundary](../../01-source-authority/README.md)
- [sealed source-role contract](../../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)
- [raw-handoff guard](../03-raw-handoff-and-import-guard/README.md)

## Checklist

- [ ] Load the fixed F03.2.1 decision and retain the exact unavailable class, source-policy reason, and
  sealed source identity that caused it.
- [ ] Keep `shader` and `extension` visibly unavailable until a separate F02 admission records their
  source identity, license, attribution, role, locator grammar, and allowed extraction scope.
- [ ] Reject a GLSL document, historical registry alias, extension list, lower version, or compatibility
  behavior offered as a replacement authority.
- [ ] Reject a modified, reordered, profile-mismatched, or silently admitted decision ledger through
  focused hostile checks.
- [ ] Preserve absent semantic facts and zero Matrix v2 rows rather than representing unresolved classes
  as supported, optional, or silently excluded.
- [ ] Hand the exact unresolved decisions to F03.2.3.3 and attach a no-claim receipt.

## Verification

This ledger is an admission boundary, not an inventory or a substitute for a future shader or extension
source. It does not prove shader execution, extension support, CTS execution, or a guest/browser path.
