# F02.3.3.4.1 — Lock and verify a WGSL grammar input

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.3.3.4.1
Depends: F02.1, F02.2, F02.3.1
Evidence: pending

Prerequisite lists: [F02.1](../../../../01-input-inventory/README.md),
[F02.2](../../../../02-fetch-verifier/README.md), and
[F02.3.1](../../../01-provenance-record/README.md).

## Outcome

The official GPUWeb `wgsl/syntax.bnf` candidate is either pinned as a grammar-generator input with a
resolved license, or rejected with a concrete reason; F02.1/F02.2 evidence is renewed accordingly.

## Starting points

- [current composite inventory root](../../../../01-input-inventory/manifest.toml)
- [current canonical inventory lock](../../../../01-input-inventory/inventory.lock)
- [WGSL source repository at the existing pin](https://github.com/gpuweb/gpuweb/tree/e0aff163a37eb3633ffd612e2a943ceb6196d6af/wgsl)
- [GPUWeb license at the existing pin](https://github.com/gpuweb/gpuweb/blob/e0aff163a37eb3633ffd612e2a943ceb6196d6af/LICENSE.md)

## Checklist

- [ ] [F02.3.3.4.1.1 — Migrate the inventory to a composite lock](01-inventory-lock-layout/README.md)
- [ ] [F02.3.3.4.1.2 — Lock and fetch the WGSL grammar](02-wgsl-grammar-lock/README.md)

## Verification

- An unavailable or mismatched grammar input leaves this child and F02.3.3.4 open.
- A successful fetch and offline rehash identify one exact external-cache payload only.

## Split rationale

The pre-cutover single-file manifest was already 175 lines, so a new readable entry would have
violated the 180-line limit. The canonical composite lock now preserves a fail-closed inventory
identity across small entry files; the grammar input, cache proof, and downstream record renewal can
therefore use that identity without compressing metadata or weakening provenance.
