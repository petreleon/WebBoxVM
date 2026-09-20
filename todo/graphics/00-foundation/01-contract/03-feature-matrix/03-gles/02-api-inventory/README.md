# F03.3.2 — Extract the GLES API inventory

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.3.2
Depends: F03.3.1
Evidence: pending

## Outcome

The shared matrix receives the complete admitted GLES 3.2 core API vocabulary: commands, objects, state,
limits, formats, features, and explicit extension decisions, all with stable source locators. Extraction
records facts and blockers, not an implementation verdict.

## Starting points

- [source-authority boundary](../01-source-authority/README.md)
- [F03 v2 matrix contract](../../01-profile-scope/matrix_contract_v2.py)
- [GLES parent task](../README.md)

## Checklist

- [ ] Consume only locator classes accepted by F03.3.1 and preserve the exact source identity on every
  API row.
- [ ] Enumerate mandatory GLES 3.2 commands, objects, state variables, state transitions, limits, format
  properties, and feature requirements without folding desktop OpenGL or lower-version behavior into the
  target.
- [ ] Record each extension as adopted, excluded, or unresolved with a reason; do not turn optional
  capabilities into implicit GLES 3.2 core requirements.
- [ ] Give every row a stable identity, exact source locator, requirement kind, and explicit provisional
  state suitable for the shared schema.
- [ ] Reject missing, duplicate, reordered, cross-profile, stale, and source-identity-mismatched rows;
  retain unavailable implementation evidence as blocked.
- [ ] Add focused extraction and hostile-input checks, then attach a no-claim API inventory receipt.

## Verification

The emitted rows are a bounded source inventory only. They leave shader execution, ownership, independent
test obligations, CTS execution, guest behavior, browser behavior, certification, and performance
unresolved.
