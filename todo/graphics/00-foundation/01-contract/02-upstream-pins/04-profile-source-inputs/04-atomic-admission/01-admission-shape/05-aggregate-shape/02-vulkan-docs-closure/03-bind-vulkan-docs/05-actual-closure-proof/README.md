# F02.4.4.1.5.2.3.5 — Prove the actual Docs closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.5
Depends: F02.4.4.1.5.2.3.3, F02.4.4.1.5.2.3.4
Evidence: [blocker record](evidence.md)

Prerequisite lists: the [captured core closure](../03-capture-core-closure/README.md),
[staged witnesses](../04-stage-build-witnesses/README.md), and [Docs parent](../../README.md).

## Outcome

Prove a complete, scoped, cap-valid, unadmitted Vulkan Docs closure or retain the exact first remaining blocker
without reclassifying any root, output, inventory, or support claim.

## Starting points

- [actual Docs grammar](../02-actual-closure-identity/README.md)
- [staging receipt](../04-stage-build-witnesses/README.md)
- [aggregate admission parent](../../../../../README.md)

## Checklist

- [ ] [F02.4.4.1.5.2.3.5.1 — Authoritative member metadata](01-authoritative-member-metadata/README.md)
- [ ] [F02.4.4.1.5.2.3.5.2 — Proof-level lineage trace](02-proof-lineage-trace/README.md)
- [ ] [F02.4.4.1.5.2.3.5.3 — Capture fresh lineage](03-capture-fresh-lineage/README.md)
- [ ] [F02.4.4.1.5.2.3.5.4 — Reify the actual member closure](04-reify-actual-member-closure/README.md)
- [ ] [F02.4.4.1.5.2.3.5.5 — Restage and cross-prove](05-restage-and-cross-prove/README.md)

## Verification

- A PASS proves source-closure evidence only; later aggregate admission still also requires the VCTS core closure.
- This task cannot mark Vulkan API, guest, browser, CTS, conformance, or performance work complete.
- Status: **BLOCKED** because observed records alone cannot establish a complete actual Docs grammar. Reifiable
  raw metadata and proof-level derived lineage are separated below; no child changes this task's unadmitted state.
  Child `.5.1` records the missing derived license authority and child `.5.2` records a narrow hosted primitive
  witness but no trusted Docs syscall trace; neither blocker receipt is a PASS.

## Split rationale

The current observer is sealed and its C trace is 139/180 lines. Static member metadata and
output-to-producer lineage are separate proof obligations, so this five-child split keeps both contracts bounded.
Only the final child may turn the blocker record into a PASS, and only if it proves the closure without widening
scope, cache authority, or support claims.
