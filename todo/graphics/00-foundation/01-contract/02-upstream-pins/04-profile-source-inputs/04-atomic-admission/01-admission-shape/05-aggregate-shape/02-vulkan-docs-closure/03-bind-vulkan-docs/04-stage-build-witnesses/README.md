# F02.4.4.1.5.2.3.4 — Stage and verify build witnesses

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.4
Depends: F02.4.4.1.5.2.3.2, F02.4.4.1.5.2.3.3
Evidence: [aggregate receipt](evidence.md)

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

- [x] [F02.4.4.1.5.2.3.4.1 — Define the isolated staging contract](01-stage-contract/README.md)
- [x] [F02.4.4.1.5.2.3.4.2 — Stage and verify captured core inputs](02-stage-core-inputs/README.md)
- [x] [F02.4.4.1.5.2.3.4.3 — Stage both bounded output witnesses](03-stage-output-witnesses/README.md)
- [x] [F02.4.4.1.5.2.3.4.4 — Publish and reuse the staged witness cache](04-verify-staged-cache/README.md)

## Verification

- A cache receipt cannot replace either required fresh official build.
- No active F02 cache, inventory, V1 grammar, F03 state, guest, browser, CTS, or performance behavior changes here.

## Split rationale

The boundary contract, input payloads, full output payloads, and reusable cache marker have separate trust and
resource boundaries. The second child handles 1,760 cap-valid core inputs; the third separately handles both
clean-run output trees; the fourth can publish only after both prior payload receipts rehash. This parent remains open
until its aggregate receipt binds all four without changing an active consumer.
