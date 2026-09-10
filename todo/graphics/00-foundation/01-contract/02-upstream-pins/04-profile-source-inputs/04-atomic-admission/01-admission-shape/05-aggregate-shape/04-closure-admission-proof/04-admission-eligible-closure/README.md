# F02.4.4.1.5.4.4 — Establish an admission-eligible closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4
Depends: F02.4.4.1.5.4.3
Evidence: pending

Prerequisite lists: the [blocked-state receipt](../03-blocked-state-receipt/README.md),
[V2 source contract](../../05-vulkan-source-contract-v2/README.md), and [F02.2 policy](../../../../../../02-fetch-verifier/README.md).

## Outcome

One later active, complete, policy-valid successor closure may turn the six-role aggregate
admission-eligible without substituting V2 VCTS for Docs or weakening the immutable scope rules.
The successor separates a reproducible Vulkan-Docs core source closure from the canonical full CTS
diagnostic suite and from release-conformance claims. It still cannot perform inventory cutover,
prove cache freshness, or alter F03.

## Starting points

- [blocked-state receipt](../03-blocked-state-receipt/README.md)
- [Docs provenance boundary](../../05-vulkan-source-contract-v2/05-aggregate-handoff/README.md)
- [source requirements](../../../../../../../03-feature-matrix/01-profile-scope/source_requirements.json)

## Checklist

- [x] [F02.4.4.1.5.4.4.1 — Define the derived-Docs successor policy](01-derived-docs-policy/README.md)
- [ ] [F02.4.4.1.5.4.4.2 — Prove the derived-Docs source closure](02-derived-docs-closure/README.md)
- [ ] [F02.4.4.1.5.4.4.3 — Separate source sufficiency from release conformance](03-source-release-boundary/README.md)
- [ ] [F02.4.4.1.5.4.4.4 — Reconcile the final aggregate admission](04-final-aggregate-reconciliation/README.md)
- [ ] [F02.4.4.1.5.4.4.5 — Resolve the independent GLES/VCTS closure policies](05-independent-suite-closures/README.md)
- [ ] [F02.4.4.1.5.4.4.6 — Define the successor inventory transition](06-successor-inventory-transition/README.md)

## Verification

- Current state: **BLOCKED** — the first live blocker is the GLES CTS multi-file closure; Docs remains
  provenance rather than an admitted implementation source, and V2 preserves `admitted=false`,
  `cutover_ready=false`, and `satisfies_vulkan_14_core_manifest=false`. This task stays unchecked until
  all three independent source truths change through their own policies.
- `admission_closures.json` and every downstream cutover/F03 effect remain reserved for F02.4.4.2–.5.

## Split rationale

Khronos publishes a canonical VCTS suite, but not an immutable core-only selector that can replace it;
Vulkan-Docs can reproduce a core document, but its generated output is not a raw F02.2 source. The
children make those different evidence roles explicit. A source-sufficiency decision may enable only
traceable planning or implementation inputs; release conformance, Khronos certification, inventory
admission, and F03 transition remain separate and false until their independent proofs exist.
