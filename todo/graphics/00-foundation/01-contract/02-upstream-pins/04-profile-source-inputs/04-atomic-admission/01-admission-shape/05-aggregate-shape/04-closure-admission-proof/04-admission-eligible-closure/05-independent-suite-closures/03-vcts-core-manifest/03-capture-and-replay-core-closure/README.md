# F02.4.4.1.5.4.4.5.3.3 — Capture and replay the VCTS core closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.3.3
Depends: F02.4.4.1.5.4.4.5.3.2
Evidence: pending

## Outcome

After authoritative manifest validation, capture its complete core closure through F02.2, verify each
member's cap, license, and cache identity, then prove offline replay. The broader V2 handoff remains
diagnostic and unadmitted.

## Starting points

- [manifest validation](../02-validate-authoritative-core-manifest/README.md)
- [F02 fetch verifier](../../../../../../../../../02-fetch-verifier/README.md)

## Checklist

- [ ] Fetch every manifest-selected raw member through F02.2 and verify its immutable identity.
- [ ] Reject over-cap, missing-license, partial, reordered, duplicate, symlinked, or stale members.
- [ ] Publish a complete marker last and replay it offline without a network fetch.
- [ ] Preserve active F02/F03 and V2 handoff false state.

## Verification

- No closure is valid before the explicit authoritative manifest is valid.
- Capture/replay does not prove CTS execution, a guest API, browser behavior, or conformance.
