# F02.4.4.1.5.2.3.4.1 — Define the isolated staging contract

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.4.1
Depends: F02.4.4.1.5.2.3.2, F02.4.4.1.5.2.3.3
Evidence: pending

Prerequisite lists: the [actual Docs grammar](../../02-actual-closure-identity/README.md),
[bound input scope](../../03-capture-core-closure/02-bind-core-input-scope/README.md), and
[independent comparison](../../03-capture-core-closure/03-compare-fresh-captures/README.md).

## Starting points

- [build-witness receipt](../../02-actual-closure-identity/evidence.md)
- [scope receipt](../../03-capture-core-closure/02-bind-core-input-scope/evidence.md)
- [comparison receipt](../../03-capture-core-closure/03-compare-fresh-captures/evidence.md)
- [fixture cache boundary patterns](../../../02-successor-cache-verifier/README.md#cache-boundary)

## Outcome

Define an actual-Docs-only, staging-only plan and marker grammar that binds the reviewed build witness, source scope,
and comparison while isolating all cache payloads outside WebBoxVM.

## Checklist

- [ ] Bind image, platform, argv, environment, mounts, recipe/toolchain, source identities, scope/comparison identities, and both run IDs.
- [ ] Require an absolute, non-root, non-symlinked external cache root outside this repository; reject aliases, traversal, and active-looking status.
- [ ] Define canonical relative selectors and descriptor-safe regular-file reads and writes using no-follow traversal and atomic no-overwrite publication.
- [ ] Define self-hashed plan and marker records whose status remains `staging-only-unadmitted`, `admitted=false`, and `cutover_ready=false`.
- [ ] Add positive and hostile filesystem, root, selector, witness-field, plan, and marker-shape regressions with a compact receipt.

## Verification

- Existing `successor_cache_*` helpers are fixture-only; this child may reuse their defensive patterns but not their public contract.
- It publishes no actual payload and changes no active F02/V1/guest/browser/CTS/conformance/performance state.
