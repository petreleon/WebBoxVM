# F02.4.4.1.5.2 — Resolve the Vulkan Docs core closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2
Depends: F02.2, F02.4.3, F02.4.4.1.3
Evidence: [blocker record](evidence.md)
Status: superseded
Superseded-by: F02.4.4.1.5.5

Prerequisite lists: the [Vulkan boundary](../../03-vulkan-boundaries/README.md),
[F02 source policy](../../../../../02-fetch-verifier/01-fetch-contract/source_model.py), and
[Vulkan Docs audit](../../../../03-vulkan-input-audit/README.md).

## Outcome

Either establish a complete, F02.2-valid Vulkan 1.4 core document closure or retain a precise blocker.
Generated/transitive files, macro/conditional configuration, and core-only scope must all have bounded
immutable identities; the root alone never qualifies.

This is retained as the V1 historical route. V2 does not claim that generated Docs members have a
uniform license or make their closure an implementation-source prerequisite.

## Starting points

- [direct include transcript](../../../../03-vulkan-input-audit/spec_includes.json)
- [boundary requirements](../../03-vulkan-boundaries/boundaries.json)
- [source fetch policy](../../../../../02-fetch-verifier/01-fetch-contract/source_fetch.py)

## Checklist

- [x] [F02.4.4.1.5.2.1 — Specify successor raw/generated closure identity](01-successor-identity/README.md)
- [x] [F02.4.4.1.5.2.2 — Verify successor closure cache staging](02-successor-cache-verifier/README.md)
- [ ] [F02.4.4.1.5.2.3 — Bind the actual Vulkan Docs closure](03-bind-vulkan-docs/README.md)

## Verification

- The existing 73 observations are insufficient; this task stays open unless every required Docs member and scope input is genuinely modelled.
- No inventory, candidate decision, F03 state, guest API, browser, CTS, conformance, or performance claim changes here.
- Status: **IN PROGRESS**: the fresh official core HTML route is now reproducible, but its closure grammar, complete
  scope/input capture, witness staging, and final proof remain deliberately separate children. The completed first
  children remain isolated successor contracts; no child may mutate active F02.2 or V1 state before an actual closure
  truthfully models every required member and output.
