# F02.4.4.1.5.2.3.4.4 — Publish and reuse the staged witness cache

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.4.4
Depends: F02.4.4.1.5.2.3.4.2, F02.4.4.1.5.2.3.4.3
Evidence: pending

Prerequisite lists: [staged core inputs](../02-stage-core-inputs/README.md) and
[staged output witnesses](../03-stage-output-witnesses/README.md).

## Starting points

- [input-staging task](../02-stage-core-inputs/README.md)
- [output-staging task](../03-stage-output-witnesses/README.md)
- [staging-contract task](../01-stage-contract/README.md)

## Outcome

Publish a self-hashed unadmitted marker only after complete staging verification, then prove a second reuse pass that
rehashes every cached input and both independently staged output witnesses.

## Checklist

- [ ] Bind scope, comparison, build-witness, canonical 1,760-input manifest, both 2,530-file output manifests, and both run identities in one marker.
- [ ] Before initial marker publication and every successful reuse, rehash all 1,760 staged input entries and both complete 2,530-file output-witness trees.
- [ ] Reject missing, stale, divergent, partial, reordered, cross-closure, self-digest-mutated, active-looking, or raced marker/cache state.
- [ ] Exercise an initial stage and successful reuse beneath a safe external root; record expected/actual nonzero counts, never payloads.
- [ ] Add positive and hostile cache-reuse, marker, output, cross-closure, stale, partial, and publication-race regressions; then aggregate this parent.

## Verification

- The receipt remains `staging-only-unadmitted`, `admitted=false`, and `cutover_ready=false`; it cannot replace the build observations.
- No active F02 cache, inventory, V1 grammar, F03 state, guest, browser, CTS, conformance, or performance behavior changes here.
