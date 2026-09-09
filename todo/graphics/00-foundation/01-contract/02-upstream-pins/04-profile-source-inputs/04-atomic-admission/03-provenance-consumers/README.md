# F02.4.4.3 — Renew provenance and reproducibility consumers

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.4.4.3
Depends: F02.3.1, F02.3.4, F06, F02.4.4.2
Evidence: pending

Prerequisite lists: [provenance validation](../../../03-provenance-contract/04-provenance-validation/README.md),
[F06](../../../../../02-reproducibility/03-file-layout/README.md), and the inventory cutover.

## Outcome

Every lock-bound provenance sidecar, generator fixture, deterministic chunk, and old-lock sentinel
is renewed atomically for the new inventory revision. The consumer proof remains provenance-only.

## Starting points

- [provenance closure validator](../../../03-provenance-contract/04-provenance-validation/validate_provenance_closure.py)
- [ABI records](../../../03-provenance-contract/02-abi-fixtures/records)
- [generator records](../../../03-provenance-contract/03-generator-outputs)
- [reproducibility fixture](../../../../../02-reproducibility/03-file-layout/03-generation-and-checker/01-chunk-generator/fixture)

## Checklist

- [ ] Enumerate the validator's complete closed sidecar set before changing any lock-bearing file.
- [ ] Renew every ABI and generator provenance record to the new lock in one reviewed change.
- [ ] Regenerate F06 deterministic fixture metadata, chunk output, and expected hashes from the same lock.
- [ ] Move renewal sentinels from the former lock to the immediate pre-cutover lock only.
- [ ] Add hostile stale-lock, missing-sidecar, and mixed-revision fixtures in bounded test modules.

## Verification

- The provenance closure rejects every old, missing, mixed, or unrecorded consumer.
- No generator output, feature support, guest route, or conformance claim follows from this renewal.
