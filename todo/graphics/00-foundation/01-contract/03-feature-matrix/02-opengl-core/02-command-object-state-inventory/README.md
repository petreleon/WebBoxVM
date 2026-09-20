# F03.2.2 — Extract command, object, and state rows

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.2.2
Depends: F03.2.1
Evidence: pending

## Outcome

The shared matrix receives the complete admitted OpenGL 4.6 core command, object, and state vocabulary
with stable source locators. The extraction records facts and blockers, not an implementation verdict.

## Starting points

- [source-authority boundary](../01-source-authority/README.md)
- [F03 v2 matrix contract](../../01-profile-scope/matrix_contract_v2.py)
- [OpenGL parent task](../README.md)

## Checklist

- [ ] Consume only locator classes accepted by F03.2.1 and preserve the exact source identity on every
  command, object, and state row.
- [ ] Enumerate mandatory core commands, objects, state variables, and state transitions without folding
  compatibility-profile behavior or unrecorded extensions into the target.
- [ ] Give every row a stable identity, source locator, requirement kind, and explicit provisional state
  suitable for the shared schema.
- [ ] Reject missing, duplicate, reordered, cross-profile, or source-identity-mismatched rows.
- [ ] Keep unavailable implementation evidence and unknown behavior visibly blocked; no row becomes
  supported merely because it was extracted.
- [ ] Add focused extraction and hostile-input checks, then attach a no-claim inventory receipt.

## Verification

The emitted rows are a bounded source inventory only. They leave ownership, independent test obligations,
CTS execution, guest behavior, browser behavior, certification, and performance unresolved.
