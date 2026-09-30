# F03.3.2.2.3 object/resource aggregate receipt

Revision: `96f2d56a7924e723af2e76c73dff60719225af6b`
Validation: six completed bounded source groups and their checked-in raw artifacts
Result: PASS
Artifacts: 20 raw inventories and the linked child receipts
Profile: GLES 3.2 raw source vocabulary; no implemented API behavior

Date: 2026-09-30 Europe/Bucharest.

## Aggregate verification

The six children preserve separate generic/sync/query, buffer, program/pipeline,
texture/sampler, framebuffer/renderbuffer and vertex/transform-feedback routes.
Current enumeration of all checked-in `*inventory.json` artifacts found 20 files
with 252 raw entries. Each canonical body matches its `inventory_sha256`, and every
artifact has exactly `raw_only=true` and `promotion_allowed=false`.
Sorted relative artifact path / raw count / inventory-hash manifest digest:
`25eb47e625d9c412473306de4b91c5119d40cf50f13562df42b50c4ca2037255`.

The first five groups' implementation and artifacts are unchanged from `140c389c`;
their separate scoped receipts and hostile tests are retained. The final group has
a fresh [aggregate receipt](06-vertex-transform-feedback-commands/evidence.md),
36 raw entries, byte-identical batch regeneration and shared machinery checks.
All six group outcomes are bounded declaration inventories; none asserts complete
GLES command/state coverage or promotes runtime ownership.

The complete project source suite rechecked the children during integration. State
and execution slices, command/state closure and Matrix ownership remain separate
unfinished siblings. Only F03.3.2.2.5 can close the full classified domain.

[Complete local integration](../../../../../../../integration-evidence-2026-09-30.md): 455 Python, 1,161 Rust and 338 web tests passed.
