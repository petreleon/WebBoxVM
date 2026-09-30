# F03.3.2.2.3.6 aggregate source receipt

Revision: `96f2d56a7924e723af2e76c73dff60719225af6b`
Validation: five source slices, batch regeneration and raw artifact equality
Result: PASS
Artifacts: five checked-in raw inventories and shared extraction/validation tooling
Profile: GLES 3.2 source inventory only; no runtime or support promotion

Date: 2026-09-30 Europe/Bucharest.

## Outcome

All five assigned children are closed: current generic attribute templates,
binding/restart literals, vertex-array object lifecycle, transform-feedback object
lifecycle and capture-control declarations. The shared engine preserves source
authority, physical pages, section fences, domain routes, grammar, unavailable ledger
and artifact hashes. Catalogs retain sealed declarations and semantic negative tests.

## Current aggregate checks

`make graphics-inventory-automation-test`: 23 tests passed, exit 0.
`make graphics-gles-vertex-inventory-regenerate`: five slices, 36 raw entries, exit 0.
Each regenerated JSON is byte-identical to its checked-in artifact.
Batch digest: `b626deed998a15d41b7603671f481a38137ef595c687f0043ded2c89046beb8a`.

Source: 2,198,754 bytes, 601 physical pages, SHA-256
`5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`.
Four existing focused suites passed 28 tests; the new capture-control suite passed 9.
Standalone suites retain full upstream checks. Within a batch only, unchanged
domain/grammar/ledger proofs are reused after admission/bytes checks, with a complete
before/after fingerprint of contract inputs, helper code and cache files.

The first shared-engine check exposed a template adapter mismatch (missing
`PRIMARY_PAGE`, exit 1). The adapter now preserves `SOURCE_PAGE`/`SECTION` and the
original grammar-document argument. The failed suite was rerun and passed.
Review also added lock/manifest mutation and ambient package-alias regressions.

Actual sealed-PDF extraction produced unchanged rows with 55 -> 14 Poppler calls
across the four migrated catalogs. This measures inventory extraction only.

## Scope and integration

Facts remain `raw_only=true`, `promotion_allowed=false`; no Matrix owner,
reference-test plan, CTS execution, rendering, API behavior or performance claim is
created. State, limits, formats, shaders and primitive processing remain routed out.
Complete project gates passed as recorded in the linked integration receipt. The broader F03
inventory and graphics compatibility/performance objective remain open.

[Complete local integration](../../../../../../../../integration-evidence-2026-09-30.md): 455 Python, 1,161 Rust and 338 web tests passed.
