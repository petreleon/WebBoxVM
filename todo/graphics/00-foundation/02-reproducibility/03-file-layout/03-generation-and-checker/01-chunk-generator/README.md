# F06.3.1 — Build deterministic generated-file chunking

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F06.3.1
Depends: F06.2
Evidence: pending

Prerequisite lists: [F06.2](../../02-line-limit-coverage/README.md).

## Outcome

A small checked generator turns an ordered, provenance-bound record stream into stable named chunks
whose maintained outputs cannot exceed 180 physical lines.

## Starting points

- [line-limit policy](../../02-line-limit-coverage/README.md)
- [F02 input inventory](../../../../01-contract/02-upstream-pins/01-input-inventory/README.md)
- [roadmap checker](../../../../../../../scripts/check_graphics_roadmap.py)

## Checklist

- [ ] Define input ordering, chunk names, headers, source-manifest revision, and output metadata.
- [ ] Implement deterministic generation and a check mode that rejects stale or oversized chunks.
- [ ] Add hermetic boundary tests for stable output and an over-180-line input/chunk.
- [ ] Record command output, test count, and byte hashes without claiming generated protocol support.

## Verification

- Identical ordered inputs generate byte-identical chunks with stable names and provenance headers.
- A stale header, unordered input, or over-limit chunk fails before it is accepted.
