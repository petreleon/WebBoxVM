# F03.4.2.1 — Seal provenance channels and the raw-to-matrix boundary

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.4.2.1
Depends: F03.4.1, F02.5.4.2
Evidence: pending

## Outcome

One fail-closed boundary binds the Vulkan normative Docs root and full-suite root through the active
role-aware API. It proves that F03.4.1's registry facts remain auxiliary structural input, not matrix
rows or a substitute for either mandatory source role.

## Starting points

- [F03 role-aware bindings](../../../01-profile-scope/role_aware_bindings.py)
- [sealed source contract](../../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)
- [raw registry inventory](../../01-registry-inventory/README.md)

## Checklist

- [ ] Load `vulkan-14-spec` and `vulkan-cts-default` only through the fixed-path F03 role-aware binding.
- [ ] Reject an F02.1 alias, ambient module, stale or mixed lock, auxiliary registry, or filtered suite root.
- [ ] Record the root revision, digest, bytes, license/attribution, and permitted citation or diagnostic channel.
- [ ] Prove raw registry rows have null owners/test plans and cannot enter the v2 matrix as normative rows.
- [ ] Record that `vkspec.adoc` includes require a separately pinned citation map, not an inferred Docs closure.
- [ ] Add hostile boundary tests and a no-claim receipt that retains `matrix-incomplete`.

## Verification

This task establishes only channel identities. It cannot create a Docs locator, select a CTS case, or
claim guest support, CTS execution, conformance, certification, browser operation, or performance.
