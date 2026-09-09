# F02.4.4.1.5.2.1 — Specify successor raw/generated closure identity

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.1
Depends: F02.2, F02.4.3, F02.4.4.1.3, F02.4.4.1.4
Evidence: [receipt](evidence.md)

Prerequisite lists: the [Docs blocker record](../evidence.md),
[F02.2 source policy](../../../../../../02-fetch-verifier/01-fetch-contract/source_model.py), and
[V1 transition grammar](../../../04-post-cutover-rules/post_cutover_schema.py).

## Outcome

An isolated, opt-in successor fixture contract defines synthetic raw members, generated members, and logical
closures without changing the active F02.1/F02.2 manifest, inventory layout, candidate audit JSON, cache,
F03 gate, or V1 transition grammar. It is not a claim about an actual Vulkan Docs closure.

## Starting points

- [F02.2 source input](../../../../../../02-fetch-verifier/01-fetch-contract/source_model.py)
- [inventory layout](../../../../../../01-input-inventory/inventory_layout.py)
- [post-cutover grammar](../../../04-post-cutover-rules/post_cutover_schema.py)
- [Docs discovery blocker](../evidence.md)
- [successor contract](successor_identity_contract.py)
- [immutable cache plan model](successor_identity_model.py)
- [synthetic fixture](successor_identity.fixture.json)
- [hostile regressions](successor_identity_hostile_test.py)

## Checklist

- [x] Model a tagged raw-versus-generated member union and one ordered logical closure identity.
- [x] Keep F02.2's per-member 8 MiB rule and reject root-only, duplicate, unordered, and oversize members.
- [x] Require generated producer members, recipe, configuration, toolchain, output, and determinism identities.
- [x] Preserve V1/audit data only as immutable predecessor evidence through a successor adapter boundary.
- [x] Add hermetic hostile tests and a focused PASS receipt without changing active consumers.

## Verification

- A generated member must never use a fabricated raw URL or silently overwrite a V1 member shape.
- Consumers receive the frozen plan only through fresh fixture validation; a caller cannot provide an admitted plan.
- This is a successor-fixture design only; it cannot admit Docs, alter inventory/cache behavior, or advance F03.
