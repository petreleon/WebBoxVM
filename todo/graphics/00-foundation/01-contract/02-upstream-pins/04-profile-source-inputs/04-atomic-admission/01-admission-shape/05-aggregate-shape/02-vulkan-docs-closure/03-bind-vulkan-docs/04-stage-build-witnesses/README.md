# F02.4.4.1.5.2.3.4 — Stage and verify build witnesses

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.4
Depends: F02.4.4.1.5.2.3.2, F02.4.4.1.5.2.3.3
Evidence: pending

Prerequisite lists: the [actual Docs grammar](../02-actual-closure-identity/README.md),
[captured core closure](../03-capture-core-closure/README.md), and [successor cache verifier](../../02-successor-cache-verifier/README.md).

## Outcome

Stage and rehash the actual unadmitted Docs inputs and output witnesses under a separate safe external cache root,
then reject any stale, divergent, partial, or substituted evidence.

## Starting points

- [cache boundary](../../02-successor-cache-verifier/README.md#cache-boundary)
- [build witness](../01-reproduce-pinned-html/evidence.md)
- [actual Docs grammar](../02-actual-closure-identity/README.md)

## Checklist

- [ ] Stage cap-valid source members and separately bounded outputs with descriptor-safe paths.
- [ ] Bind image, platform, argv, environment, mounts, recipe, and both clean-run tree identities.
- [ ] Rehash every cached member and reject missing, stale, divergent, partial, or cross-closure markers.
- [ ] Add hostile filesystem, identity, scope, output, and cache-reuse regressions.
- [ ] Record a reproducible unadmitted staging receipt with nonzero cases.

## Verification

- A cache receipt cannot replace either required fresh official build.
- No active F02 cache, inventory, V1 grammar, F03 state, guest, browser, CTS, or performance behavior changes here.
