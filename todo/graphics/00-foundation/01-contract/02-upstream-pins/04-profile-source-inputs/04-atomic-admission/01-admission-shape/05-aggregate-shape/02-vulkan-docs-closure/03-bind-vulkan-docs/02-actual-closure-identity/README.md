# F02.4.4.1.5.2.3.2 — Define the actual Docs closure identity

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.2
Depends: F02.4.4.1.5.2.3.1
Evidence: [receipt](evidence.md)

Prerequisite lists: the [build witness](../01-reproduce-pinned-html/README.md),
[synthetic identity contract](../../01-successor-identity/README.md), and [F02 source policy](../../../../../../../02-fetch-verifier/01-fetch-contract/source_model.py).

## Outcome

An isolated, unadmitted actual-Docs grammar distinguishes cap-limited immutable closure inputs from separately
bounded rendered outputs, recipes, toolchain, and two-run identities without changing active F02/V1 consumers.

This child validates the future closure's shape and the actual two-run build witness only. It does not claim a complete
resolved selector set: child `.3` must capture the conditional and promotion-metadata closure before any scope proof.

## Starting points

- [official build receipt](../01-reproduce-pinned-html/evidence.md)
- [successor identity fixture](../../01-successor-identity/README.md)
- [post-cutover grammar](../../../../04-post-cutover-rules/post_cutover_schema.py)

## Checklist

- [x] Define raw, generated-input, rendered-output, recipe, configuration, scope, and predecessor identities.
- [x] Preserve F02.2's 8 MiB source-member policy while imposing an explicit bounded output-artifact policy.
- [x] Reject mutable, root-only, duplicate-path, malformed, structurally partial, or active-consumer-looking data.
- [x] Bind the pinned container context and two clean-run output identities without reusing fixture-only semantics.
- [x] Add focused positive and hostile tests that keep every result unadmitted and not cutover-ready.

## Verification

- Identical bytes at distinct safe selectors may not be confused with duplicate identities or cache paths.
- A rendered output has no fabricated raw URL or revision and cannot substitute for an F02.2 source member.
