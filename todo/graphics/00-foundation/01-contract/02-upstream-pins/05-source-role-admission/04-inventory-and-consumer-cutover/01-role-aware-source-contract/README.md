# F02.5.4.1 — Seal the role-aware source contract

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.5.4.1
Depends: F02.5.2, F02.5.3
Evidence: [receipt](evidence.md)

## Outcome

A separate, self-hashed F02.5 successor inventory admits the exact three normative roots and three
immutable full-suite roots as one ordered contract. It records local definitions, shards, and maps as
WebBoxVM auxiliary evidence only, without rewriting the historical F02.1 17-input inventory.

## Starting points

- [normative root catalog](../../02-normative-build-record/01-normative-root-pins/normative_roots.py)
- [full-suite root catalog](../../03-full-suite-record/01-canonical-full-suite-roots/full_suite_roots.py)
- [source-role policy](../../01-authority-and-transform-boundary/source-role-policy.md)

## Checklist

- [x] Build one ordered catalog from the producer-validated normative and full-suite records.
- [x] Seal its raw lock and receipt identity; reject a stale, mixed, reordered, or duplicate record.
- [x] Bind every mandatory record to exact kind, revision, digest, scope, and root selector identity.
- [x] Preserve full-suite ledger/cache receipt links; a root selector alone cannot prove closure.
- [x] Register the registry, local definition, maps, and bounded shards as false-claim auxiliary evidence.
- [x] Add fresh-empty-cache, positive, and hostile contract tests without broadening F02.1's input cap.

## Verification

The exact six mandatory profile roles are exposed atomically from the sealed contract. No auxiliary
record can discharge a role, no full-suite closure is squeezed through a generic 8 MiB source input,
and every support, conformance, certification, profile, and performance claim remains false.
