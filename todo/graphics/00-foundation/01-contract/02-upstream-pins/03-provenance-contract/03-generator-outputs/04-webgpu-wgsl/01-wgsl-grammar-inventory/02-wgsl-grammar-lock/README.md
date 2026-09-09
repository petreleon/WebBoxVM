# F02.3.3.4.1.2 — Lock and fetch the WGSL grammar

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.3.3.4.1.2
Depends: F02.2, F02.3.3.4.1.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.2](../../../../../02-fetch-verifier/README.md) and
[the composite-lock migration](../01-inventory-lock-layout/README.md).

## Outcome

The official GPUWeb `wgsl/syntax.bnf` grammar is a separately classified immutable generator input,
with its source identity, W3C document license, limited dialect, and fresh external-cache result
recorded truthfully.

## Starting points

- [grammar candidate at its reviewed commit](https://raw.githubusercontent.com/gpuweb/gpuweb/e0aff163a37eb3633ffd612e2a943ceb6196d6af/wgsl/syntax.bnf)
- [GPUWeb license at the same commit](https://github.com/gpuweb/gpuweb/blob/e0aff163a37eb3633ffd612e2a943ceb6196d6af/LICENSE.md)
- [F02.2 live-cache workflow](../../../../../02-fetch-verifier/03-live-inventory/README.md)

## Checklist

- [x] Add only the verified grammar input with immutable URL, commit, SHA-256, byte count, license,
  external cache path, and narrowly stated grammar-generator role.
- [x] Keep the candidate in its own approved grammar family; preserve the existing `wgsl-spec` role as
  semantic reference material.
- [x] Fetch all declared inputs into a new external cache, then offline-rehash them; record the
  16th grammar result and every unavailable input as a failure.
- [x] Regenerate the canonical lock and renew F02.1/F02.2 receipt facts without retaining payloads.

## Verification

- The grammar input is rejected if its exact raw bytes, license classification, or nonstandard BNF
  dialect cannot be established.
- Success proves source integrity only, not a WGSL compiler, WebGPU API, guest, or browser feature.
