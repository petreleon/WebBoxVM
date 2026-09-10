# F02.4.4.1.5.2.3.3 — Capture the core input and scope closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.3
Depends: F02.4.4.1.5.2.3.2
Evidence: pending

Prerequisite lists: the [actual Docs grammar](../02-actual-closure-identity/README.md),
[Docs include audit](../../../../../../03-vulkan-input-audit/README.md), and [Vulkan boundary](../../../../03-vulkan-boundaries/README.md).

## Outcome

Capture every cap-valid raw/generated core closure input and the exact WSI, video, extension, configuration, and
promotion-metadata treatment from the reproducible official build without treating a root or observations as a closure.

## Starting points

- [build witness](../01-reproduce-pinned-html/evidence.md)
- [direct include transcript](../../../../../../03-vulkan-input-audit/spec_includes.json)
- [boundary requirements](../../../../03-vulkan-boundaries/boundaries.json)

## Checklist

- [x] [F02.4.4.1.5.2.3.3.1 — Observe pinned core build inputs](01-observe-pinned-build-inputs/README.md)
- [x] [F02.4.4.1.5.2.3.3.2 — Bind the core input and scope manifest](02-bind-core-input-scope/README.md)
- [ ] [F02.4.4.1.5.2.3.3.3 — Compare fresh captures and record the receipt](03-compare-fresh-captures/README.md)

## Verification

- A command-line core flag alone is not evidence that every included and excluded member has the required scope.
- No rendered output, active inventory, candidate decision, F03 state, or support claim changes in this child.

## Split rationale

A direct include replay cannot establish generator and copied-asset reads, while a raw read trace cannot by itself
classify conditional scope. The first child records the phase-specific observation, the second rejects an unsafe
normalized closure, and the third compares two independent captures. The parent remains open until their aggregate
receipt records nonzero counts without staging payloads or changing an active consumer.
