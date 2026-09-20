# F03.3.1 — Set the GLES source-authority boundary

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.3.1
Depends: F03.1, F02.5.4.2
Evidence: pending

## Outcome

Every proposed GLES 3.2 API and ESSL locator class has an explicit role-aware authority decision before
an inventory reader exists. The boundary consumes the sealed F02 contract, cannot resolve a historical
manifest by alias, and fails closed when a required locator needs an unadmitted distinct source.

## Starting points

- [sealed F02 source contract](../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)
- [F03 role-aware bindings](../../01-profile-scope/role_aware_bindings.py)
- [GLES parent task](../README.md)

## Checklist

- [ ] Load the exact GLES normative-root and full-suite-root records only through the F03 role-aware
  binding; reject a legacy manifest alias, ambient module, wrong profile, or auxiliary record.
- [ ] Classify GLES command/state/limit/format, shader, precision, and extension locator classes as
  derivable from the admitted normative root or as requiring a separately recorded source decision.
- [ ] Record the exact revision, digest, source-locator syntax, and permitted extraction scope for each
  accepted class; do not infer ESSL authority from an unrelated record.
- [ ] Make an unresolved class unavailable to F03.3.2 and F03.3.3 rather than replacing it with a lower
  GLES version, desktop OpenGL behavior, or an unpinned document.
- [ ] Add positive and hostile boundary checks for stale, mixed, reordered, legacy, and auxiliary input.
- [ ] Attach a no-claim boundary receipt and retain `matrix-incomplete`.

## Verification

A source ambiguity stops downstream extraction. This task neither imports an API row nor makes a support,
guest, browser, CTS-execution, certification, or performance claim.
