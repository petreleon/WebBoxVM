# F02.4.4.1.5.4.4.2.1 — Anchor the historical successor closure evidence

[Parent task](../README.md)

Task: F02.4.4.1.5.4.4.2.1
Depends: F02.4.4.1.5.4.4.1
Evidence: [receipt](evidence.md)

Prerequisite lists: the [derived-Docs policy](../../01-derived-docs-policy/README.md) and retained
[historical Docs witnesses](../../../../02-vulkan-docs-closure/03-bind-vulkan-docs/README.md).

## Outcome

One fail-closed adapter binds the policy to the existing witness, scope, and comparison records while
classifying them only as historical, unadmitted evidence. It explicitly rejects treating the old two
observations or rendered output as a fresh successor closure.

## Starting points

- [build witness](../../../../02-vulkan-docs-closure/03-bind-vulkan-docs/02-actual-closure-identity/vulkan_docs_build_witness.json)
- [captured scope](../../../../02-vulkan-docs-closure/03-bind-vulkan-docs/03-capture-core-closure/02-bind-core-input-scope/vulkan_docs_core_input_scope.json)
- [capture comparison](../../../../02-vulkan-docs-closure/03-bind-vulkan-docs/03-capture-core-closure/03-compare-fresh-captures/vulkan_docs_core_input_comparison.json)

## Checklist

- [x] Bind the self-hashed successor policy and each immutable historical-record byte identity.
- [x] Require exact Vulkan 1.4 core role, raw/derived counts, build configuration, and unadmitted statuses.
- [x] Keep the historical rendered output separate from every source-role claim.
- [x] Mark historical evidence, role proof, admission, cutover, and release effects false.
- [x] Reject stale anchors, altered counts/statuses, output-as-source, and false-ready records with focused tests.
- [x] Attach a bounded receipt without changing F02 inventory, F03, or aggregate admission.

## Verification

- PASS means only that the historical evidence is accurately anchored and cannot be misrepresented.
- It does not cure the missing per-derived license/attribution/role/provenance/producer authority or produce
  a fresh write-lineage trace.
